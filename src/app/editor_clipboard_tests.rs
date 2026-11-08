use crate::{
	app::App,
	features::editor::clipboard::{PasteMode, import_mode},
	libraries::ansi::TextStyle,
	libraries::clipboard::{Error, Receipt},
};
use crate::{
	libraries::ansi::{Action, Ink},
	libraries::editing::{EditKind, Movement},
};
use std::time::Instant;

fn select_all(app: &mut App) {
	app.workspace
		.editor
		.navigate(&mut app.workspace.session.editing, Movement::Start, false);
	app.workspace
		.editor
		.navigate(&mut app.workspace.session.editing, Movement::Finish, true);
}

#[test]
fn styled_selection_roundtrips_and_paste_is_one_undo_step() {
	let mut source = App::default();
	source.paste_editor_text("one\n界é", PasteMode::Styled);
	select_all(&mut source);
	source.workspace.session.editing.apply_tool(Action::Bold);
	source
		.workspace
		.session
		.editing
		.apply_tool(Action::Foreground(Ink::Rgb(0, 128, 255)));
	let selection = source
		.workspace
		.session
		.editing
		.selected_document()
		.unwrap();
	let copied = selection.to_escaped_ansi();
	let mut target = App::default();
	target.paste_editor_text("replace", PasteMode::Styled);
	select_all(&mut target);
	target.workspace.session.editing.pending = Some(TextStyle {
		italic: true,
		..Default::default()
	});
	let before = target.workspace.session.editing.document.clone();
	let pending = target.workspace.session.editing.pending;
	target.paste_editor_text(&copied, PasteMode::Styled);
	assert_eq!(target.workspace.session.editing.document, selection);
	assert_eq!(target.workspace.session.editing.pending, pending);
	target
		.workspace
		.editor
		.undo(&mut target.workspace.session.editing, false);
	assert_eq!(target.workspace.session.editing.document, before);
	target
		.workspace
		.editor
		.undo(&mut target.workspace.session.editing, true);
	assert_eq!(target.workspace.session.editing.document, selection);
}

#[test]
fn plain_literal_and_automatic_detection_are_distinct() {
	let text = r"\x1b[31mred\nnext\x1b[0m";
	let mut app = App::default();
	app.workspace.session.editing.pending = Some(TextStyle {
		italic: true,
		..Default::default()
	});
	app.paste_editor_text(text, PasteMode::Plain);
	assert_eq!(
		app.workspace.session.editing.document.lines(),
		["red", "next"]
	);
	assert!(
		app.workspace
			.session
			.editing
			.document
			.style_runs()
			.iter()
			.all(|r| r.style.italic && r.style.foreground == Ink::Default)
	);
	let mut app = App::default();
	app.paste_editor_text(text, PasteMode::Literal);
	assert_eq!(app.workspace.session.editing.document.lines(), [text]);
	for literal in [r"C:\new\text", r"\n\t", r"\\x1b[31m"] {
		assert!(import_mode(literal).is_none());
	}
	let mut app = App::default();
	app.paste_editor_text("\x1b[1mraw\x1b[0m", PasteMode::Styled);
	assert_eq!(app.workspace.session.editing.document.lines(), ["raw"]);
	assert!(
		app.workspace.session.editing.document.style_runs()[0]
			.style
			.bold
	);
}

#[test]
fn review_does_not_change_document_until_accepted() {
	for accepted in [false, true] {
		let mut app = App::default();
		app.paste_editor_text("old", PasteMode::Styled);
		select_all(&mut app);
		let before = app.workspace.session.editing.document.clone();
		app.paste_editor_text(r"\x1b[31mnew\x1b[2J", PasteMode::Styled);
		assert_eq!(app.workspace.session.editing.document, before);
		assert_eq!(app.import_review.as_ref().unwrap().import.issues.len(), 1);
		app.decide_review(if accepted {
			crate::components::import_review::Decision::Strip
		} else {
			crate::components::import_review::Decision::Abort
		});
		assert_eq!(
			app.workspace.session.editing.document.lines(),
			[if accepted { "new" } else { "old" }]
		);
		if accepted {
			app.workspace
				.editor
				.undo(&mut app.workspace.session.editing, false);
			assert_eq!(app.workspace.session.editing.document, before);
		}
	}
}

#[test]
fn cut_requires_success_and_an_unchanged_target_and_is_undoable() {
	for scenario in 0..4 {
		let mut app = App::default();
		app.paste_editor_text("keep", PasteMode::Styled);
		select_all(&mut app);
		let (service, control) = crate::test_clipboard::controlled();
		app.clipboard = service;
		app.copy_selection(true);
		if scenario == 2 {
			app.workspace.editor.navigate(
				&mut app.workspace.session.editing,
				Movement::Left,
				false,
			);
		}
		if scenario == 3 {
			app.workspace.editor.insert(
				&mut app.workspace.session.editing,
				"changed",
				EditKind::Atomic,
				Instant::now(),
			);
			app.workspace
				.editor
				.undo(&mut app.workspace.session.editing, false);
		}
		control
			.writes
			.send(if scenario == 1 {
				Err(Error::TmuxWrite)
			} else {
				Ok(crate::libraries::clipboard::WriteOutput::Delivered(
					Receipt::Tmux,
				))
			})
			.unwrap();
		crate::test_clipboard::until(|| app.poll_editor_copy(&mut Vec::new()));
		assert_eq!(
			app.workspace.session.editing.document.lines(),
			[if scenario == 0 { "" } else { "keep" }]
		);
		if scenario == 0 {
			app.workspace
				.editor
				.undo(&mut app.workspace.session.editing, false);
			assert_eq!(app.workspace.session.editing.document.lines(), ["keep"]);
		}
	}
}

#[test]
fn delayed_paste_cannot_overwrite_new_editing() {
	let mut app = App::default();
	let (service, control) = crate::test_clipboard::controlled();
	app.clipboard = service;
	app.request_editor_paste(PasteMode::Styled);
	app.paste_editor_text("new", PasteMode::Styled);
	control.reads.send(Ok("stale".into())).unwrap();
	crate::test_clipboard::until(|| app.poll_editor_paste());
	assert_eq!(app.workspace.session.editing.document.lines(), ["new"]);
}
