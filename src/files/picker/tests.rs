use super::*;
use crate::shortcuts::ActionId as A;
use crossterm::event::{Event, MouseButton, MouseEventKind};
use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend};
fn key(picker: &mut Picker, code: KeyCode, modifiers: KeyModifiers) -> Outcome {
	{
		let scope = picker.input_scope();
		let key = crate::shortcuts::ShortcutKey::from(KeyEvent::new(code, modifiers));
		match picker
			.test_shortcuts
			.dispatch(key.clone(), scope, Instant::now())
		{
			crate::shortcuts::Dispatch::Action(action) => picker.shortcut(action),
			crate::shortcuts::Dispatch::Unbound => key
				.printable()
				.map(|ch| picker.event(&Event::Paste(ch.to_string())))
				.unwrap_or(Outcome::None),
			_ => Outcome::None,
		}
	}
}

fn selected(picker: &mut Picker, name: &str) {
	let explorer = picker.explorer.as_mut().unwrap();
	let index = explorer
		.files()
		.iter()
		.position(|file| file.name == name)
		.unwrap();
	explorer.set_selected_idx(index);
	picker.focus = Focus::List;
}

#[test]
fn shift_m_selects_middle_of_visible_entries_without_scrolling() {
	let folder = tempfile::tempdir().unwrap();
	for i in 0..30 {
		std::fs::write(folder.path().join(format!("{i:02}.tuxd.json")), "").unwrap();
	}
	let mut picker = Picker::new(Action::Open, "", Some(&folder.path().join("doc.tuxd.json")));
	let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
	terminal
		.draw(|frame| picker.draw(frame, "Open", None))
		.unwrap();
	picker.top = 5;
	let expected = 5 + (picker.list_area.height as usize - 1) / 2;
	key(&mut picker, KeyCode::Char('M'), KeyModifiers::SHIFT);
	assert_eq!(picker.explorer.as_ref().unwrap().selected_idx(), expected);
	assert_eq!(picker.top, 5);
	picker.focus = Focus::Name;
	key(&mut picker, KeyCode::Char('a'), KeyModifiers::CONTROL);
	key(&mut picker, KeyCode::Char('M'), KeyModifiers::SHIFT);
	assert_eq!(picker.name.text(), "M");
}

#[test]
fn explorer_focus_vim_navigation_and_shortcut_aliases() {
	let folder = tempfile::tempdir().unwrap();
	std::fs::create_dir(folder.path().join("child")).unwrap();
	std::fs::write(folder.path().join("one.tuxd.json"), "").unwrap();
	let mut picker = Picker::new(Action::Open, "", Some(&folder.path().join("doc.tuxd.json")));
	assert!(picker.focus == Focus::List);
	key(&mut picker, KeyCode::Char('j'), KeyModifiers::NONE);
	assert_eq!(picker.selected().unwrap().0, folder.path().join("child"));
	key(&mut picker, KeyCode::Char('l'), KeyModifiers::NONE);
	assert_eq!(picker.cwd, folder.path().join("child"));
	key(&mut picker, KeyCode::Char('h'), KeyModifiers::NONE);
	assert_eq!(picker.cwd, folder.path());
	key(&mut picker, KeyCode::Char('G'), KeyModifiers::SHIFT);
	assert_eq!(
		picker.selected().unwrap().0,
		folder.path().join("one.tuxd.json")
	);
	key(&mut picker, KeyCode::Char('g'), KeyModifiers::NONE);
	assert_eq!(
		picker.selected().unwrap().0,
		folder.path().join("one.tuxd.json")
	);
	key(&mut picker, KeyCode::Char('g'), KeyModifiers::NONE);
	assert_eq!(picker.explorer.as_ref().unwrap().selected_idx(), 0);
	key(&mut picker, KeyCode::Char('.'), KeyModifiers::CONTROL);
	assert!(picker.explorer.as_ref().unwrap().show_hidden());
	picker.focus = Focus::Name;
	key(&mut picker, KeyCode::Char('a'), KeyModifiers::CONTROL);
	for ch in "hjklggG".chars() {
		key(&mut picker, KeyCode::Char(ch), KeyModifiers::NONE);
	}
	assert_eq!(picker.name.text(), "hjklggG");
}

#[test]
fn ansi_radios_are_keyboard_and_mouse_accessible() {
	use crossterm::event::MouseEvent;
	let mut picker = Picker::new(Action::ExportRaw, "", None);
	key(&mut picker, KeyCode::Tab, KeyModifiers::NONE);
	assert!(picker.focus == Focus::Format);
	key(&mut picker, KeyCode::Right, KeyModifiers::NONE);
	assert_eq!(picker.action(), Action::ExportEscaped);
	let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
	terminal
		.draw(|frame| picker.draw(frame, "Export ANSI", None))
		.unwrap();
	let rect = picker.format_buttons[0].0;
	picker.event(&Event::Mouse(MouseEvent {
		kind: MouseEventKind::Down(MouseButton::Left),
		column: rect.x,
		row: rect.y,
		modifiers: KeyModifiers::NONE,
	}));
	assert_eq!(picker.action(), Action::ExportRaw);
}

#[test]
fn browse_filter_and_save_in_selected_directory() {
	let folder = tempfile::tempdir().unwrap();
	std::fs::create_dir(folder.path().join("nested")).unwrap();
	for name in ["doc.tuxd.json", "other.txt", ".hidden.tuxd.json"] {
		std::fs::write(folder.path().join(name), "").unwrap();
	}
	let mut picker = Picker::new(
		Action::SaveAs,
		"",
		Some(&folder.path().join("doc.tuxd.json")),
	);
	assert_eq!(picker.explorer.as_ref().unwrap().files().len(), 3); // parent, nested, document
	key(&mut picker, KeyCode::Char('f'), KeyModifiers::CONTROL);
	assert_eq!(picker.explorer.as_ref().unwrap().files().len(), 4);
	key(&mut picker, KeyCode::Char('h'), KeyModifiers::CONTROL);
	assert_eq!(picker.explorer.as_ref().unwrap().files().len(), 5);
	selected(&mut picker, "nested/");
	key(&mut picker, KeyCode::Enter, KeyModifiers::NONE);
	assert_eq!(picker.cwd, folder.path().join("nested"));
	picker.focus = Focus::Name;
	picker.event(&Event::Paste("new 界 document".into()));
	match key(&mut picker, KeyCode::Enter, KeyModifiers::NONE) {
		Outcome::Submit(path) => {
			assert_eq!(path.path, folder.path().join("nested/new 界 document"))
		}
		_ => panic!("expected a destination"),
	}
}

#[test]
fn location_paste_failure_and_unicode_filename_editing() {
	let folder = tempfile::tempdir().unwrap();
	let mut picker = Picker::new(Action::Open, "", None);
	key(&mut picker, KeyCode::Char('l'), KeyModifiers::CONTROL);
	picker.event(&Event::Paste(folder.path().to_string_lossy().into_owned()));
	key(&mut picker, KeyCode::Enter, KeyModifiers::NONE);
	assert_eq!(picker.cwd, folder.path());
	picker.navigate(folder.path().join("missing"));
	assert_eq!(picker.cwd, folder.path());
	assert!(picker.error.is_some());
	picker.focus = Focus::Name;
	picker.event(&Event::Paste("café界\n".into()));
	key(&mut picker, KeyCode::Backspace, KeyModifiers::NONE);
	assert_eq!(picker.name.text(), "café");
	key(&mut picker, KeyCode::Char('a'), KeyModifiers::CONTROL);
	picker.event(&Event::Paste("replacement.tuxd.json".into()));
	assert_eq!(picker.name.text(), "replacement.tuxd.json");
	assert_eq!(
		picker.destination(),
		folder.path().join("replacement.tuxd.json")
	);
}

#[test]
fn clipboard_delivery_is_scoped_to_the_picker_and_target_field() {
	let mut picker = Picker::new(Action::Open, "", None);
	let (service, control) = crate::test_clipboard::controlled();
	picker.clipboard = service;
	picker.focus = Focus::Name;
	picker.shortcut(A::Paste);
	assert!(!picker.poll_paste());
	picker.focus = Focus::Location;
	control
		.reads
		.send(Ok("/tmp/pasted path.tuxd.json".into()))
		.unwrap();
	crate::test_clipboard::until(|| picker.poll_paste());
	assert_eq!(
		picker.destination(),
		PathBuf::from("/tmp/pasted path.tuxd.json")
	);
	picker.shortcut(A::Paste);
	drop(picker);
	control.reads.send(Ok("stale".into())).unwrap();
	// A new picker owns a different request channel and cannot consume that result.
	let mut next = Picker::new(Action::Open, "", None);
	assert!(!next.poll_paste());
	assert!(next.name.text().is_empty());
}

#[test]
fn mouse_scrollbar_and_small_terminal_remain_usable() {
	use crossterm::event::MouseEvent;
	let folder = tempfile::tempdir().unwrap();
	for i in 0..60 {
		std::fs::write(folder.path().join(format!("{i:02}.tuxd.json")), "").unwrap();
	}
	let mut picker = Picker::new(Action::Open, "", Some(&folder.path().join("doc.tuxd.json")));
	let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
	terminal
		.draw(|frame| picker.draw(frame, "Open", None))
		.unwrap();
	let bar = picker.bar.unwrap();
	picker.event(&Event::Mouse(MouseEvent {
		kind: MouseEventKind::Down(MouseButton::Left),
		column: bar.area.x,
		row: bar.area.bottom() - 1,
		modifiers: KeyModifiers::NONE,
	}));
	terminal
		.draw(|frame| picker.draw(frame, "Open", None))
		.unwrap();
	assert!(picker.top > 0);
	let index = picker.top;
	let path = picker.explorer.as_ref().unwrap().files()[index]
		.path
		.clone();
	let click = Event::Mouse(MouseEvent {
		kind: MouseEventKind::Down(MouseButton::Left),
		column: picker.list_area.x,
		row: picker.list_area.y,
		modifiers: KeyModifiers::NONE,
	});
	picker.event(&click);
	assert_eq!(picker.destination(), path);
	assert!(matches!(picker.event(&click), Outcome::Submit(p) if p.path == path));
	for (width, height) in [(1, 1), (12, 5), (30, 12), (80, 24)] {
		let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
		terminal
			.draw(|frame| picker.draw(frame, "Open", None))
			.unwrap();
	}
	assert!(matches!(
		key(&mut picker, KeyCode::Esc, KeyModifiers::NONE),
		Outcome::Cancel
	));
}

#[test]
fn directory_picker_chooses_current_or_typed_directory_without_loading_a_file() {
	let folder = tempfile::tempdir().unwrap();
	std::fs::create_dir(folder.path().join("nested")).unwrap();
	std::fs::write(folder.path().join("document.tuxd.json"), "").unwrap();
	let hint = folder.path().join("document.tuxd.json");
	let mut picker = Picker::new(Action::OpenDirectory, "", Some(&hint));
	assert_eq!(picker.destination(), folder.path());
	assert!(
		picker
			.explorer
			.as_ref()
			.unwrap()
			.files()
			.iter()
			.all(|file| file.is_dir)
	);
	assert!(
		matches!(picker.submit(), Outcome::Submit(selection) if selection.path == folder.path())
	);
	picker.name.set("nested".into());
	assert!(
		matches!(picker.submit(), Outcome::Submit(selection) if selection.path == folder.path().join("nested"))
	);
}
