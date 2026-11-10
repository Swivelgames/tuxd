use super::*;
use crate::{
	app::App,
	libraries::ansi::{Document, Mode},
	persistence,
};
use crate::{
	libraries::ansi::Position,
	libraries::ansi::{Action as StyleAction, Ink},
	libraries::editing::Movement,
};
use crossterm::event::Event;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn type_text(app: &mut App, text: &str) {
	app.handle(Event::Paste(text.into()));
}
fn key(app: &mut App, code: KeyCode) -> bool {
	app.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)))
}
fn save_as(app: &mut App, path: &std::path::Path) {
	app.file_action(Action::SaveAs);
	app.submit_path(Action::SaveAs, path.to_path_buf());
}

#[test]
fn imported_document_dialogs_keep_source_directory_and_cwd_shortcut() {
	let folder = tempfile::tempdir().unwrap();
	let source = folder.path().join("source.txt");
	std::fs::write(&source, "Imported").unwrap();
	let mut app = App::default();
	app.submit_path(Action::ImportEscaped, source.clone());
	assert!(app.workspace.active.session.path.is_none());
	for action in [
		Action::SaveAs,
		Action::Open,
		Action::ImportEscaped,
		Action::ExportEscaped,
	] {
		app.file_action(action);
		type_text(&mut app, "destination");
		let Some(Dialog::Path { picker, .. }) = &app.files.dialog else {
			panic!("expected picker")
		};
		assert_eq!(picker.destination(), folder.path().join("destination"));
		app.handle(Event::Key(KeyEvent::new(
			KeyCode::Char('g'),
			KeyModifiers::CONTROL,
		)));
		let Some(Dialog::Path { picker, .. }) = &app.files.dialog else {
			panic!("expected picker")
		};
		// Navigation preserves an export/save filename but clears open/import names.
		if matches!(action, Action::SaveAs | Action::ExportEscaped) {
			assert_eq!(
				picker.destination(),
				std::env::current_dir().unwrap().join("destination")
			);
		}
		key(&mut app, KeyCode::Esc);
	}
}

#[test]
fn open_non_document_offers_import_and_uses_selected_format() {
	let folder = tempfile::tempdir().unwrap();
	let path = folder.path().join("styled.txt");
	std::fs::write(&path, r"\x1b[31mRed\x1b[0m").unwrap();
	let mut app = App::default();
	type_text(&mut app, "keep");
	let before = app.workspace.active.session.editing.document.clone();
	app.submit_path(Action::Open, path.clone());
	assert!(matches!(
		&app.files.dialog,
		Some(Dialog::Path { picker }) if picker.action() == Action::ImportEscaped
	));
	key(&mut app, KeyCode::Esc);
	assert_eq!(app.workspace.active.session.editing.document, before);
	app.submit_path(Action::Open, path.clone());
	key(&mut app, KeyCode::Right); // Raw -> Escaped, still no import.
	assert_eq!(app.workspace.active.session.editing.document, before);
	key(&mut app, KeyCode::Enter); // Import directly from the selected format.
	assert!(app.files.dialog.is_none());
	assert_eq!(app.workspace.buffers().count(), 2);
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["Red"]
	);
	assert_eq!(
		app.workspace
			.active
			.session
			.editing
			.document
			.style_at_offset(0)
			.foreground,
		Ink::Ansi(1)
	);
	assert!(app.workspace.active.session.path.is_none());
	assert!(app.dirty());
}

#[test]
fn save_dirty_undo_redo_and_export_are_independent() {
	let folder = tempfile::tempdir().unwrap();
	let path = folder.path().join("styled.tuxd.json");
	let mut app = App::default();
	type_text(&mut app, "Hello\n界");
	app.workspace.active.editor.navigate(
		&mut app.workspace.active.session.editing,
		Movement::Start,
		true,
	);
	app.workspace
		.active
		.session
		.editing
		.apply_tool(StyleAction::Background(Ink::Rgb(1, 2, 3)));
	app.workspace.active.session.editing.anchor = None;
	assert!(app.dirty());
	save_as(&mut app, &path);
	assert!(!app.dirty());
	assert!(app.files.dialog.is_none());
	assert_eq!(
		persistence::load(&path).unwrap(),
		app.workspace.active.session.editing.document
	);
	type_text(&mut app, "X");
	assert!(app.dirty());
	app.workspace
		.active
		.editor
		.undo(&mut app.workspace.active.session.editing, false);
	assert!(!app.dirty());
	app.workspace
		.active
		.editor
		.undo(&mut app.workspace.active.session.editing, true);
	assert!(app.dirty());
	let raw = folder.path().join("output.ansi");
	app.write_file(Action::ExportRaw, raw.clone(), false);
	assert_eq!(
		std::fs::read(raw).unwrap(),
		app.workspace
			.active
			.session
			.editing
			.document
			.to_raw_ansi()
			.as_bytes()
	);
	let escaped = folder.path().join("output.txt");
	app.write_file(Action::ExportEscaped, escaped.clone(), false);
	assert_eq!(
		std::fs::read(escaped).unwrap(),
		app.workspace
			.active
			.session
			.editing
			.document
			.to_escaped_ansi()
			.as_bytes()
	);
	assert!(app.dirty());
	assert_eq!(app.workspace.active.session.path, Some(path));
}

#[test]
fn closing_buffer_can_cancel_or_save_without_losing_history() {
	let folder = tempfile::tempdir().unwrap();
	let mut app = App::default();
	type_text(&mut app, "keep");
	let before = app.workspace.active.session.editing.document.clone();
	app.file_action(Action::CloseBuffer);
	assert!(matches!(app.files.dialog, Some(Dialog::Dirty)));
	assert_eq!(app.files.choice, 2);
	key(&mut app, KeyCode::Enter); // Default Cancel leaves the document unchanged.
	assert!(app.files.dialog.is_none());
	assert_eq!(app.workspace.active.session.editing.document, before);
	app.file_action(Action::CloseBuffer);
	key(&mut app, KeyCode::Char('s'));
	assert!(matches!(app.files.dialog, Some(Dialog::Path { .. })));
	let path = folder.path().join("saved.tuxd.json");
	type_text(&mut app, path.to_str().unwrap());
	key(&mut app, KeyCode::Enter);
	assert_eq!(persistence::load(&path).unwrap(), before);
	assert_eq!(
		app.workspace.active.session.editing.document,
		Document::default()
	);
	assert!(!app.dirty());
	app.workspace
		.active
		.editor
		.undo(&mut app.workspace.active.session.editing, false);
	assert_eq!(
		app.workspace.active.session.editing.document,
		Document::default()
	);
}

#[test]
fn failures_and_overwrite_cancel_preserve_document_path_and_dirty_state() {
	let folder = tempfile::tempdir().unwrap();
	let existing = folder.path().join("existing.tuxd.json");
	std::fs::write(&existing, b"keep bytes").unwrap();
	let mut app = App::default();
	type_text(&mut app, "unsaved");
	let before = app.workspace.active.session.editing.document.clone();
	save_as(&mut app, &existing);
	assert!(matches!(app.files.dialog, Some(Dialog::Overwrite { .. })));
	key(&mut app, KeyCode::Char('n'));
	assert_eq!(std::fs::read(&existing).unwrap(), b"keep bytes");
	app.file_action(Action::Open);
	app.submit_path(Action::Open, existing.to_str().unwrap().into());
	assert!(app.files.error.is_some());
	key(&mut app, KeyCode::Esc);
	app.file_action(Action::Quit);
	key(&mut app, KeyCode::Char('s'));
	app.submit_path(
		Action::SaveAs,
		folder
			.path()
			.join("missing/sub/doc")
			.to_str()
			.unwrap()
			.into(),
	);
	assert!(app.files.error.is_some());
	assert!(!app.handle(Event::Resize(80, 24)));
	assert!(app.workspace.active.session.path.is_none());
	assert!(app.dirty());
	assert_eq!(app.workspace.active.session.editing.document, before);
	key(&mut app, KeyCode::Esc);
	app.workspace
		.active
		.editor
		.undo(&mut app.workspace.active.session.editing, false);
	assert!(app.workspace.active.session.editing.document.is_empty());
}

#[test]
fn import_review_abort_strip_and_dirty_protection() {
	let folder = tempfile::tempdir().unwrap();
	let source = folder.path().join("import.txt");
	std::fs::write(&source, r"\x1b[31mRed\x1b[2J\n界\q").unwrap();
	let mut app = App::default();
	type_text(&mut app, "original");
	let original = app.workspace.active.session.editing.document.clone();
	for abort in [true, false] {
		app.file_action(Action::ImportEscaped);
		app.submit_path(Action::ImportEscaped, source.to_str().unwrap().into());
		if let Some(review) = &app.import_review {
			assert_eq!(review.import.issues.len(), 2);
		} else {
			panic!("expected review");
		}
		// Pasting a response cannot answer a confirmation.
		type_text(&mut app, "s");
		assert_eq!(app.workspace.active.session.editing.document, original);
		key(&mut app, KeyCode::Char(if abort { 'a' } else { 's' }));
		if abort {
			assert_eq!(app.workspace.active.session.editing.document, original);
		}
	}
	assert!(app.files.dialog.is_none());
	assert_eq!(
		app.workspace
			.buffers()
			.next()
			.unwrap()
			.session
			.editing
			.document,
		original
	);
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["Red", "界"]
	);
	assert_eq!(
		app.workspace
			.active
			.session
			.editing
			.document
			.style_at_offset(0)
			.foreground,
		Ink::Ansi(1)
	);
	assert!(app.dirty());
	assert!(app.workspace.active.session.path.is_none());
	app.workspace
		.active
		.editor
		.undo(&mut app.workspace.active.session.editing, false);
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["Red", "界"]
	);
}

#[test]
fn clean_document_quit_and_save_before_quit() {
	let folder = tempfile::tempdir().unwrap();
	let path = folder.path().join("doc.tuxd.json");
	let mut app = App::default();
	let mut doc = Document::default();
	doc.insert(Position::default(), "saved").unwrap();
	app.load_document(doc, Some(path.clone()), false);
	assert!(!app.dirty());
	type_text(&mut app, "new");
	app.file_action(Action::Quit);
	assert!(key(&mut app, KeyCode::Char('s')));
	assert_eq!(
		persistence::load(&path).unwrap(),
		app.workspace.active.session.editing.document
	);
	assert!(!app.dirty());
}

#[test]
fn export_refuses_to_replace_active_document() {
	let folder = tempfile::tempdir().unwrap();
	let path = folder.path().join("doc.tuxd.json");
	let mut app = App::default();
	type_text(&mut app, "content");
	save_as(&mut app, &path);
	app.write_file(Action::ExportRaw, path.clone(), true);
	assert!(app.files.error.is_some());
	assert_eq!(
		persistence::load(&path).unwrap(),
		app.workspace.active.session.editing.document
	);
}

#[test]
fn long_review_and_narrow_menus_remain_navigable() {
	use ratatui::{Terminal, backend::TestBackend};
	let mut app = App::default();
	app.import_document(
		crate::libraries::ansi::parse(&"\x1b[2J".repeat(80), Mode::Raw).unwrap(),
		crate::workflows::Source::Stdin,
	);
	let mut terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
	terminal
		.draw(|frame| app.import_review.as_mut().unwrap().draw(frame))
		.unwrap();
	assert!(app.import_review.as_ref().unwrap().view.has_scrollbar());
	key(&mut app, KeyCode::End);
	terminal
		.draw(|frame| app.import_review.as_mut().unwrap().draw(frame))
		.unwrap();
	assert!(app.import_review.as_ref().unwrap().view.scrolled());
	key(&mut app, KeyCode::Char('a'));
	app.file_dialog(Dialog::Menu { index: 0 });
	for _ in 0..8 {
		terminal
			.draw(|frame| draw(frame, &mut app.files, &app.shortcuts.config))
			.unwrap();
		key(&mut app, KeyCode::Down);
	}
	assert!(matches!(app.files.dialog, Some(Dialog::Menu { index: 8 })));
}

#[test]
fn selected_native_path_retains_non_utf8_bytes() {
	use std::os::unix::ffi::OsStringExt;
	let directory = tempfile::tempdir().unwrap();
	let name = std::ffi::OsString::from_vec(b"document-\xff.tuxd.json".to_vec());
	let path = directory.path().join(name);
	let bytes = crate::adapters::native_document::encode(&Document::default()).unwrap();
	std::fs::write(&path, bytes).unwrap();
	let mut app = App::default();
	app.submit_path(Action::Open, path.clone());
	assert_eq!(app.workspace.active.session.path, Some(path));
	assert!(app.files.dialog.is_none());
}
