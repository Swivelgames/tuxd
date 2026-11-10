//! Cross-buffer workflows: identity, editing state, persistence and delayed work.
use super::App;
use crate::{
	features::editor::clipboard::PasteMode,
	files::{Action, Dialog},
	libraries::editing::Movement,
	shortcuts::ActionId,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

fn key(app: &mut App, code: KeyCode) -> bool {
	app.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)))
}

fn paste(app: &mut App, text: &str) {
	app.paste_editor_text(text, PasteMode::Literal);
}

#[test]
fn new_and_switch_preserve_order_selection_history_and_views() {
	let mut app = App::default();
	paste(&mut app, "first");
	app.workspace.active.editor.navigate(
		&mut app.workspace.active.session.editing,
		Movement::Start,
		true,
	);
	app.workspace.active.editor.surface.viewport.wrap = false;
	app.workspace.active.code.surface.viewport.left = 7;
	let first = app.workspace.active.session.id;
	for text in ["second", "third"] {
		app.file_action(Action::New);
		paste(&mut app, text);
	}
	assert!(app.files.dialog.is_none());
	let names: Vec<_> = app
		.workspace
		.buffers()
		.map(|b| b.session.editing.document.lines()[0].clone())
		.collect();
	assert_eq!(names, ["first", "second", "third"]);
	for index in [0, 2, 1, 0] {
		app.workspace.select_buffer(index);
	}
	assert_eq!(app.workspace.active.session.id, first);
	assert!(app.workspace.active.session.editing.anchor.is_some());
	assert!(!app.workspace.active.editor.surface.viewport.wrap);
	assert_eq!(app.workspace.active.code.surface.viewport.left, 7);
	app.execute_action(ActionId::Undo);
	assert!(app.workspace.active.session.editing.document.is_empty());
	app.workspace.select_buffer(1);
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["second"]
	);
}

#[test]
fn duplicate_open_focuses_existing_document_and_save_cannot_overwrite_another_buffer() {
	let folder = tempfile::tempdir().unwrap();
	let path = folder.path().join("one.tuxd.json");
	let mut app = App::default();
	paste(&mut app, "first");
	app.write_file(Action::SaveAs, path.clone(), false);
	let first = app.workspace.active.session.id;
	paste(&mut app, " changed");
	app.file_action(Action::New);
	paste(&mut app, "second");
	app.write_file(Action::SaveAs, path.clone(), true);
	assert!(app.files.error.as_ref().unwrap().contains("another buffer"));
	assert_eq!(crate::persistence::load(&path).unwrap().lines(), ["first"]);
	key(&mut app, KeyCode::Esc);
	app.submit_path(Action::Open, folder.path().join(".").join("one.tuxd.json"));
	assert_eq!(app.workspace.active.session.id, first);
	assert_eq!(app.workspace.buffers().count(), 2);
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["first changed"]
	);
}

#[test]
fn save_all_prompts_for_untitled_and_restores_original_tab() {
	let folder = tempfile::tempdir().unwrap();
	let mut app = App::default();
	paste(&mut app, "first");
	app.file_action(Action::New);
	paste(&mut app, "second");
	let original = app.workspace.active.session.id;
	app.file_action(Action::SaveAll);
	for (name, text) in [("one", "first"), ("two", "second")] {
		assert!(matches!(app.files.dialog, Some(Dialog::Path { .. })));
		let path = folder.path().join(format!("{name}.tuxd.json"));
		app.submit_path(Action::SaveAs, path.clone());
		assert_eq!(crate::persistence::load(&path).unwrap().lines(), [text]);
	}
	assert_eq!(app.workspace.active.session.id, original);
	assert!(app.workspace.buffers().all(|b| !b.session.dirty()));
	assert!(app.files.dialog.is_none());
	for index in 0..2 {
		app.workspace.select_buffer(index);
		paste(&mut app, " updated");
	}
	app.file_action(Action::SaveAll);
	assert!(app.files.dialog.is_none());
	assert_eq!(app.workspace.active.session.id, original);
	for (name, expected) in [("one", "first updated"), ("two", "second updated")] {
		assert_eq!(
			crate::persistence::load(&folder.path().join(format!("{name}.tuxd.json")))
				.unwrap()
				.lines(),
			[expected]
		);
	}
}

#[test]
fn quit_cancel_preserves_every_buffer_and_close_last_creates_empty_buffer() {
	let mut app = App::default();
	paste(&mut app, "first");
	app.file_action(Action::New);
	paste(&mut app, "second");
	app.file_action(Action::Quit);
	assert!(!key(&mut app, KeyCode::Char('d')));
	assert!(matches!(app.files.dialog, Some(Dialog::Dirty)));
	assert!(!key(&mut app, KeyCode::Esc));
	assert!(app.workspace.buffers().all(|b| b.session.dirty()));
	assert_eq!(app.workspace.buffers().count(), 2);
	for _ in 0..2 {
		app.file_action(Action::CloseBuffer);
		key(&mut app, KeyCode::Char('d'));
	}
	assert_eq!(app.workspace.buffers().count(), 1);
	assert!(app.workspace.active.session.editing.document.is_empty());
	assert!(!app.dirty());
}

#[test]
fn delayed_paste_applies_only_to_its_original_buffer() {
	let (service, control) = crate::test_clipboard::controlled();
	let mut app = App {
		clipboard: service,
		..Default::default()
	};
	paste(&mut app, "first");
	let first = app.workspace.active.session.id;
	app.request_editor_paste(PasteMode::Literal);
	app.file_action(Action::New);
	paste(&mut app, "second");
	control.reads.send(Ok(" pasted".into())).unwrap();
	crate::test_clipboard::until(|| app.poll_editor_paste());
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["second"]
	);
	app.workspace.select_session(first);
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["first pasted"]
	);
}

#[test]
fn new_shortcuts_dispatch_and_tabs_keep_active_visible_at_narrow_widths() {
	let mut app = App::default();
	for _ in 0..12 {
		app.file_action(Action::New);
	}
	let last = app.workspace.active.session.id;
	for (binding, same) in [("Ctrl+PageUp", false), ("Ctrl+PageDown", true)] {
		let key = crate::shortcuts::key::defaults(binding).remove(0);
		app.handle(Event::Key(KeyEvent::new(key.code, key.modifiers)));
		assert_eq!(app.workspace.active.session.id == last, same);
	}
	for width in [20, 40, 64, 100] {
		let mut terminal =
			ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, 24)).unwrap();
		terminal.draw(|frame| app.draw(frame)).unwrap();
		let buffer = terminal.backend().buffer();
		let tabs: String = (0..width).map(|x| buffer[(x, 0)].symbol()).collect();
		assert!(tabs.starts_with(" tuxd"), "{tabs}");
		assert!(
			tabs.contains(if width == 20 { "untitl" } else { "untitled 13" }),
			"{tabs}"
		);
	}
}

#[test]
fn delayed_cut_removes_only_the_original_selection() {
	use crate::libraries::clipboard::{Receipt, WriteOutput};
	let (service, control) = crate::test_clipboard::controlled();
	let mut app = App {
		clipboard: service,
		..Default::default()
	};
	paste(&mut app, "cut me");
	app.execute_action(ActionId::SelectAll);
	let first = app.workspace.active.session.id;
	app.copy_selection(true);
	app.file_action(Action::New);
	paste(&mut app, "keep me");
	control
		.writes
		.send(Ok(WriteOutput::Delivered(Receipt::Desktop)))
		.unwrap();
	crate::test_clipboard::until(|| app.poll_editor_copy(&mut Vec::new()));
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["keep me"]
	);
	app.workspace.select_session(first);
	assert!(app.workspace.active.session.editing.document.is_empty());
	app.execute_action(ActionId::Undo);
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["cut me"]
	);
}

#[test]
fn failed_save_all_can_cancel_without_touching_later_buffers() {
	let folder = tempfile::tempdir().unwrap();
	let mut app = App::default();
	paste(&mut app, "first");
	app.file_action(Action::New);
	paste(&mut app, "second");
	app.file_action(Action::SaveAll);
	app.submit_path(Action::SaveAs, folder.path().join("missing/doc.tuxd.json"));
	assert!(app.files.error.is_some());
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["first"]
	);
	key(&mut app, KeyCode::Esc);
	assert!(app.pending_file.is_none());
	assert!(
		app.workspace
			.buffers()
			.all(|b| b.session.dirty() && b.session.path.is_none())
	);
}

#[test]
fn mouse_tabs_switch_documents_and_long_names_retain_dirty_marker() {
	use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
	let mut app = App::default();
	paste(&mut app, "one");
	app.file_action(Action::New);
	paste(&mut app, "two");
	let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(80, 24)).unwrap();
	terminal.draw(|frame| app.draw(frame)).unwrap();
	app.handle(Event::Mouse(MouseEvent {
		kind: MouseEventKind::Down(MouseButton::Left),
		column: 10,
		row: 0,
		modifiers: KeyModifiers::NONE,
	}));
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["one"]
	);
	app.workspace.active.session.origin =
		Some("very-long-document-name-with-more-text-than-fits.ansi".into());
	let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(20, 24)).unwrap();
	terminal.draw(|frame| app.draw(frame)).unwrap();
	let row: String = (0..20)
		.map(|x| terminal.backend().buffer()[(x, 0)].symbol())
		.collect();
	assert!(row.contains('*'), "{row}");
}

fn click_first_tab_close(app: &mut App) {
	use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
	let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(100, 24)).unwrap();
	terminal.draw(|frame| app.draw(frame)).unwrap();
	let column = (8..100)
		.find(|x| terminal.backend().buffer()[(*x, 0)].symbol() == "x")
		.expect("tab close button");
	app.handle(Event::Mouse(MouseEvent {
		kind: MouseEventKind::Down(MouseButton::Left),
		column,
		row: 0,
		modifiers: KeyModifiers::NONE,
	}));
}

#[test]
fn background_tab_close_can_cancel_then_save_the_correct_document() {
	let folder = tempfile::tempdir().unwrap();
	let path = folder.path().join("closed.tuxd.json");
	let mut app = App::default();
	paste(&mut app, "first");
	let first = app.workspace.active.session.id;
	app.file_action(Action::New);
	paste(&mut app, "second");
	let second = app.workspace.active.session.id;
	click_first_tab_close(&mut app);
	assert_eq!(app.workspace.active.session.id, first);
	assert!(matches!(app.files.dialog, Some(Dialog::Dirty)));
	key(&mut app, KeyCode::Enter); // Cancel is initially focused.
	assert_eq!(app.workspace.buffers().count(), 2);
	app.workspace.select_session(second);
	click_first_tab_close(&mut app);
	key(&mut app, KeyCode::Char('s'));
	app.submit_path(Action::SaveAs, path.clone());
	assert_eq!(crate::persistence::load(&path).unwrap().lines(), ["first"]);
	assert_eq!(app.workspace.buffers().count(), 1);
	assert_eq!(app.workspace.active.session.id, second);
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["second"]
	);
}

#[test]
fn closing_clean_last_tab_creates_a_fresh_untitled_document() {
	let mut app = App::default();
	let original = app.workspace.active.session.id;
	click_first_tab_close(&mut app);
	assert!(app.files.dialog.is_none());
	assert_eq!(app.workspace.buffers().count(), 1);
	assert_ne!(app.workspace.active.session.id, original);
	assert!(app.workspace.active.session.editing.document.is_empty());
}

#[test]
fn open_reuses_only_the_unchanged_initial_document_and_only_after_success() {
	let folder = tempfile::tempdir().unwrap();
	let path = folder.path().join("existing.tuxd.json");
	let mut saved = App::default();
	paste(&mut saved, "saved");
	saved.write_file(Action::SaveAs, path.clone(), false);
	let mut app = App::default();
	let initial = app.workspace.active.session.id;
	app.submit_path(Action::Open, folder.path().join("missing.tuxd.json"));
	assert_eq!(app.workspace.active.session.id, initial);
	key(&mut app, KeyCode::Esc);
	app.submit_path(Action::Open, path.clone());
	assert_eq!(app.workspace.buffers().count(), 1);
	assert_ne!(app.workspace.active.session.id, initial);
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["saved"]
	);
	let mut multiple = App::default();
	multiple.file_action(Action::New);
	multiple.workspace.select_buffer(0);
	multiple.submit_path(Action::Open, path.clone());
	assert_eq!(multiple.workspace.buffers().count(), 2);
	assert_eq!(
		multiple.workspace.active.session.editing.document.lines(),
		["saved"]
	);
	for explicit_new in [false, true] {
		let mut app = App::default();
		if explicit_new {
			app.file_action(Action::New);
		} else {
			paste(&mut app, "keep");
		}
		let before = app.workspace.buffers().count();
		app.submit_path(Action::Open, path.clone());
		assert_eq!(app.workspace.buffers().count(), before + 1);
	}
}
