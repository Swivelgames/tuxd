use super::*;
use crate::workspace::Pane;
use crate::{libraries::ansi::Action, libraries::editing::Movement};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
	Event::Key(KeyEvent::new(code, modifiers))
}

fn choose(app: &mut App, control: Control) {
	app.workspace.panel.selected = app
		.workspace
		.panel
		.items()
		.iter()
		.position(|item| item.control == control)
		.unwrap();
	app.workspace.activate_tool();
}

#[test]
fn collapsing_sections_preserves_selection_and_document_history() {
	let mut app = App::default();
	app.handle(Event::Paste("selected".into()));
	app.workspace.active.editor.navigate(
		&mut app.workspace.active.session.editing,
		Movement::Home,
		true,
	);
	let selection = app.workspace.active.session.editing.selection();
	let original = app.workspace.active.session.editing.document.clone();
	for section in 0..3 {
		choose(&mut app, Control::Section(section));
		assert!(app.workspace.panel.collapsed(section));
	}
	assert_eq!(app.workspace.panel.items().len(), 3);
	assert_eq!(app.workspace.active.session.editing.selection(), selection);
	assert_eq!(app.workspace.active.session.editing.document, original);
	for section in 0..3 {
		choose(&mut app, Control::Section(section));
	}
	assert!(
		app.workspace
			.panel
			.items()
			.iter()
			.any(|item| item.control == Control::Glyph('┌'))
	);
	app.workspace
		.active
		.editor
		.undo(&mut app.workspace.active.session.editing, false);
	assert!(app.workspace.active.session.editing.document.is_empty());
}

#[test]
fn palettes_preserve_selection_and_rgb_is_staged_until_valid_apply() {
	use crate::libraries::ansi::Ink;
	let mut app = App::default();
	assert!(
		app.workspace.code_open
			&& app.workspace.active.code.surface.viewport.wrap
			&& app.workspace.active.editor.surface.viewport.wrap
	);
	app.handle(Event::Paste("hello".into()));
	app.handle(key(KeyCode::Home, KeyModifiers::SHIFT));
	let selection = app.workspace.active.session.editing.selection();
	let original = app.workspace.active.session.editing.document.clone();
	choose(&mut app, Control::Target(true));
	choose(&mut app, Control::Palette(2));
	choose(&mut app, Control::Field(0));
	app.handle(Event::Paste("#00ff80".into()));
	assert_eq!(app.workspace.active.session.editing.document, original);
	app.handle(key(KeyCode::Enter, KeyModifiers::NONE));
	assert_eq!(
		app.workspace
			.active
			.session
			.editing
			.document
			.style_at_offset(0)
			.background,
		Ink::Rgb(0, 255, 128)
	);
	assert_eq!(app.workspace.active.session.editing.selection(), selection);
	let colored = app.workspace.active.session.editing.document.clone();
	choose(&mut app, Control::Field(2));
	app.handle(Event::Paste("256".into()));
	app.handle(key(KeyCode::Enter, KeyModifiers::NONE));
	assert!(app.workspace.panel.color.rgb.error.is_some());
	assert_eq!(app.workspace.active.session.editing.document, colored);
	// Invalid apply did not add an undo transaction.
	app.handle(key(KeyCode::Char('z'), KeyModifiers::CONTROL));
	assert_eq!(app.workspace.active.session.editing.document, original);
	app.handle(key(KeyCode::Char('y'), KeyModifiers::CONTROL));
	assert_eq!(app.workspace.active.session.editing.document, colored);
	choose(&mut app, Control::Style(Action::Clear));
	assert_eq!(app.workspace.active.session.editing.document, original);
	app.workspace
		.active
		.editor
		.undo(&mut app.workspace.active.session.editing, false);
	assert_eq!(app.workspace.active.session.editing.document, colored);
}

#[test]
fn glyph_replacement_and_repeated_insertion_are_independent_undo_steps() {
	let mut app = App::default();
	app.handle(Event::Paste("界x".into()));
	app.workspace.active.editor.navigate(
		&mut app.workspace.active.session.editing,
		Movement::Start,
		false,
	);
	app.workspace.active.editor.navigate(
		&mut app.workspace.active.session.editing,
		Movement::Right,
		true,
	);
	app.workspace.active.session.editing.apply_tool(Action::Dim);
	let before = app.workspace.active.session.editing.document.clone();
	choose(&mut app, Control::Glyph('┌'));
	assert_eq!(app.workspace.focus, Pane::Tools);
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["┌x"]
	);
	assert!(
		app.workspace
			.active
			.session
			.editing
			.document
			.style_at_offset(0)
			.dim
	);
	choose(&mut app, Control::Glyph('─'));
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["┌─x"]
	);
	app.workspace
		.active
		.editor
		.undo(&mut app.workspace.active.session.editing, false);
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["┌x"]
	);
	app.workspace
		.active
		.editor
		.undo(&mut app.workspace.active.session.editing, false);
	assert_eq!(app.workspace.active.session.editing.document, before);
	app.workspace
		.active
		.editor
		.undo(&mut app.workspace.active.session.editing, true);
	app.workspace.panel.characters.query.set("U+F015".into());
	app.workspace.panel.characters.refresh();
	choose(&mut app, Control::Glyph('\u{f015}'));
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["┌\u{f015}x"]
	);
	app.handle(key(KeyCode::Esc, KeyModifiers::NONE));
	assert_eq!(app.workspace.focus, Pane::Editor);
}

#[test]
fn control_and_alt_word_bindings_and_focus_specific_wrap() {
	for modifier in [KeyModifiers::CONTROL, KeyModifiers::ALT] {
		let mut app = App::default();
		app.handle(Event::Paste("one two".into()));
		app.handle(key(KeyCode::Left, modifier));
		assert_eq!(app.workspace.active.session.editing.cursor.byte, 4);
		app.handle(key(KeyCode::Right, modifier | KeyModifiers::SHIFT));
		assert!(app.workspace.active.session.editing.selection().is_some());
		app.handle(key(KeyCode::Backspace, modifier));
		assert_eq!(
			app.workspace.active.session.editing.document.lines(),
			["one "]
		);
		app.handle(key(KeyCode::F(3), KeyModifiers::NONE));
		assert!(!app.workspace.active.editor.surface.viewport.wrap);
		assert!(app.workspace.active.code.surface.viewport.wrap);
		app.workspace.code_open = true;
		app.workspace.focus = Pane::Code;
		app.handle(key(KeyCode::F(3), KeyModifiers::NONE));
		assert!(
			!app.workspace.active.editor.surface.viewport.wrap
				&& !app.workspace.active.code.surface.viewport.wrap
		);
	}
}

#[test]
fn keyboard_selection_applies_styles_to_text() {
	let mut app = App::default();
	app.handle(Event::Paste("Hello\nworld".into()));
	app.handle(key(KeyCode::Home, KeyModifiers::CONTROL));
	for _ in 0..5 {
		app.handle(key(KeyCode::Right, KeyModifiers::SHIFT));
	}
	assert!(app.workspace.active.session.editing.selection().is_some());
	app.handle(key(KeyCode::Char('b'), KeyModifiers::CONTROL));
	assert!(
		app.workspace
			.active
			.session
			.editing
			.document
			.style_at_offset(0)
			.bold
	);
}

#[test]
fn quit_is_cancelable_and_paste_cannot_answer_a_dialog() {
	let mut app = App::default();
	app.handle(Event::Paste("hello".into()));
	assert!(!app.handle(key(KeyCode::Char('q'), KeyModifiers::CONTROL)));
	assert!(matches!(
		app.files.dialog,
		Some(crate::files::Dialog::Dirty)
	));
	assert!(!app.handle(Event::Paste("y".into())));
	assert_eq!(
		app.workspace.active.session.editing.document.lines(),
		["hello"]
	);
	app.handle(key(KeyCode::Esc, KeyModifiers::NONE));
	assert!(app.settings.is_none());
	app.handle(key(KeyCode::Char('q'), KeyModifiers::CONTROL));
	assert!(app.handle(key(KeyCode::Char('d'), KeyModifiers::NONE)));
}

#[test]
fn help_blocks_edits_and_empty_editor_exits_directly() {
	let mut app = App::default();
	app.handle(key(KeyCode::F(1), KeyModifiers::NONE));
	app.handle(Event::Paste("ignored".into()));
	app.handle(key(KeyCode::Char('x'), KeyModifiers::NONE));
	assert!(app.workspace.active.session.editing.document.is_empty());
	app.handle(key(KeyCode::Esc, KeyModifiers::NONE));
	assert!(app.handle(key(KeyCode::Char('q'), KeyModifiers::CONTROL)));
}
