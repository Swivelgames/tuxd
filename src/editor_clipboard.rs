//! Clipboard transactions retain their original editor target until completion.
use crate::{
	app::App,
	document::{Document, Position},
	editor::Editor,
	styling::TextStyle,
};
use std::sync::mpsc::{self, Receiver};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasteMode {
	Styled,
	Plain,
	Literal,
}

pub struct Target {
	revision: u64,
	document: Document,
	cursor: Position,
	anchor: Option<Position>,
	pending: Option<TextStyle>,
}
impl Target {
	fn capture(editor: &Editor) -> Self {
		Self {
			revision: editor.revision,
			document: editor.document.clone(),
			cursor: editor.cursor,
			anchor: editor.anchor,
			pending: editor.pending,
		}
	}
	fn matches(&self, editor: &Editor) -> bool {
		self.revision == editor.revision
			&& self.document == editor.document
			&& self.cursor == editor.cursor
			&& self.anchor == editor.anchor
			&& self.pending == editor.pending
	}
}

pub struct PendingPaste {
	target: Target,
	mode: PasteMode,
	receiver: Receiver<Result<String, String>>,
}
pub struct PasteReview {
	target: Target,
	formatted: bool,
}

/// Only unescaped ESC notation triggers decoding; ordinary backslashes stay literal.
fn import_mode(text: &str) -> Option<crate::ansi_import::Mode> {
	if text
		.chars()
		.any(|ch| ch.is_control() && !matches!(ch, '\n' | '\r' | '\t'))
	{
		return Some(crate::ansi_import::Mode::Raw);
	}
	let bytes = text.as_bytes();
	let mut at = 0;
	while at < bytes.len() {
		if bytes[at..].starts_with(b"\\\\") {
			at += 2;
		} else if bytes[at..].starts_with(b"\\x1b[") || bytes[at..].starts_with(b"\\x1B[") {
			return Some(crate::ansi_import::Mode::Escaped);
		} else {
			at += 1;
		}
	}
	None
}

impl App {
	pub fn copy_selection(&mut self, cut: bool) {
		if self.clipboard_result.is_some() {
			return;
		}
		let Some(document) = self.editor.selected_document() else {
			return;
		};
		self.cut_target = cut.then(|| Target::capture(&self.editor));
		self.copy_text(crate::styling::clipboard_text(&document), true);
	}
	pub fn finish_editor_copy(&mut self, result: Result<String, String>) {
		if let Some(target) = self.cut_target.take()
			&& result.is_ok()
		{
			if target.matches(&self.editor)
				&& self.files.dialog.is_none()
				&& self.settings.is_none()
			{
				self.editor.delete(true, std::time::Instant::now());
			} else {
				self.notice =
					Some("Copied selection; cut canceled because the editor changed".into());
				return;
			}
		}
		self.notice = Some(result.unwrap_or_else(|error| error));
	}
	pub fn request_editor_paste(&mut self, mode: PasteMode) {
		if self.editor_paste.is_some() {
			return;
		}
		let (sender, receiver) = mpsc::channel();
		self.editor_paste = Some(PendingPaste {
			target: Target::capture(&self.editor),
			mode,
			receiver,
		});
		std::thread::spawn(move || {
			let _ = sender.send(crate::clipboard::paste());
		});
	}
	pub fn poll_editor_paste(&mut self) -> bool {
		let Some(pending) = self.editor_paste.take() else {
			return false;
		};
		match pending.receiver.try_recv() {
			Ok(Ok(text))
				if pending.target.matches(&self.editor)
					&& self.files.dialog.is_none()
					&& self.settings.is_none() =>
			{
				self.paste_editor_text(&text, pending.mode)
			}
			Ok(Ok(_)) => self.notice = Some("Paste canceled because the editor changed".into()),
			Ok(Err(error)) => self.notice = Some(error),
			Err(mpsc::TryRecvError::Empty) => {
				self.editor_paste = Some(pending);
				return false;
			}
			Err(mpsc::TryRecvError::Disconnected) => {
				self.notice = Some("Clipboard paste failed".into())
			}
		}
		true
	}
	pub fn paste_editor_text(&mut self, text: &str, mode: PasteMode) {
		let detected = if mode == PasteMode::Literal {
			None
		} else {
			import_mode(text)
		};
		if let Some(detected) = detected {
			match crate::ansi_import::parse(text, detected) {
				Ok(import) if import.issues.is_empty() => self
					.editor
					.paste_document(&import.document, mode == PasteMode::Styled),
				Ok(import) => {
					self.paste_review = Some(PasteReview {
						target: Target::capture(&self.editor),
						formatted: mode == PasteMode::Styled,
					});
					self.files.review = crate::import_review::Review::clipboard();
					self.file_dialog(crate::files::Dialog::Review(import));
				}
				Err(error) => self.notice = Some(format!("Paste failed: {error}")),
			}
		} else {
			let text = crate::document::sanitize(text);
			let mut document = Document::default();
			document.insert(Position::default(), &text);
			self.editor.paste_document(&document, false);
		}
	}
	pub fn finish_paste_review(&mut self, document: &Document, accepted: bool) -> bool {
		let Some(review) = self.paste_review.take() else {
			return false;
		};
		if accepted {
			if review.target.matches(&self.editor) {
				self.editor.paste_document(document, review.formatted);
			} else {
				self.notice = Some("Paste canceled because the editor changed".into());
			}
		}
		true
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		editor::{EditKind, Movement},
		styling::{Action, Ink},
	};
	use std::time::Instant;

	fn select_all(app: &mut App) {
		app.editor.navigate(Movement::Start, false);
		app.editor.navigate(Movement::Finish, true);
	}

	#[test]
	fn styled_selection_roundtrips_and_paste_is_one_undo_step() {
		let mut source = App::default();
		source.paste_editor_text("one\n界é", PasteMode::Styled);
		select_all(&mut source);
		source.editor.apply_tool(Action::Bold);
		source
			.editor
			.apply_tool(Action::Foreground(Ink::Rgb(0, 128, 255)));
		let selection = source.editor.selected_document().unwrap();
		let copied = crate::styling::clipboard_text(&selection);
		let mut target = App::default();
		target.paste_editor_text("replace", PasteMode::Styled);
		select_all(&mut target);
		target.editor.pending = Some(TextStyle {
			italic: true,
			..Default::default()
		});
		let before = target.editor.document.clone();
		let pending = target.editor.pending;
		target.paste_editor_text(&copied, PasteMode::Styled);
		assert_eq!(target.editor.document, selection);
		assert_eq!(target.editor.pending, pending);
		target.editor.undo(false);
		assert_eq!(target.editor.document, before);
		target.editor.undo(true);
		assert_eq!(target.editor.document, selection);
	}

	#[test]
	fn plain_literal_and_automatic_detection_are_distinct() {
		let text = r"\x1b[31mred\nnext\x1b[0m";
		let mut app = App::default();
		app.editor.pending = Some(TextStyle {
			italic: true,
			..Default::default()
		});
		app.paste_editor_text(text, PasteMode::Plain);
		assert_eq!(app.editor.document.lines, ["red", "next"]);
		assert!(
			app.editor
				.document
				.styles
				.runs
				.iter()
				.all(|r| r.style.italic && r.style.foreground == Ink::Default)
		);
		let mut app = App::default();
		app.paste_editor_text(text, PasteMode::Literal);
		assert_eq!(app.editor.document.lines, [text]);
		for literal in [r"C:\new\text", r"\n\t", r"\\x1b[31m"] {
			assert!(import_mode(literal).is_none());
		}
		let mut app = App::default();
		app.paste_editor_text("\x1b[1mraw\x1b[0m", PasteMode::Styled);
		assert_eq!(app.editor.document.lines, ["raw"]);
		assert!(app.editor.document.styles.runs[0].style.bold);
	}

	#[test]
	fn review_does_not_change_document_until_accepted() {
		for accepted in [false, true] {
			let mut app = App::default();
			app.paste_editor_text("old", PasteMode::Styled);
			select_all(&mut app);
			let before = app.editor.document.clone();
			app.paste_editor_text(r"\x1b[31mnew\x1b[2J", PasteMode::Styled);
			assert_eq!(app.editor.document, before);
			let Some(crate::files::Dialog::Review(import)) = app.files.dialog.take() else {
				panic!("expected review")
			};
			assert_eq!(import.issues.len(), 1);
			assert!(app.finish_paste_review(&import.document, accepted));
			assert_eq!(
				app.editor.document.lines,
				[if accepted { "new" } else { "old" }]
			);
			if accepted {
				app.editor.undo(false);
				assert_eq!(app.editor.document, before);
			}
		}
	}

	#[test]
	fn cut_requires_success_and_an_unchanged_target_and_is_undoable() {
		for scenario in 0..4 {
			let mut app = App::default();
			app.paste_editor_text("keep", PasteMode::Styled);
			select_all(&mut app);
			app.cut_target = Some(Target::capture(&app.editor));
			if scenario == 2 {
				app.editor.navigate(Movement::Left, false);
			}
			if scenario == 3 {
				app.editor
					.insert("changed", EditKind::Atomic, Instant::now());
				app.editor.undo(false);
			}
			app.finish_editor_copy(if scenario == 1 {
				Err("failed".into())
			} else {
				Ok("forwarded".into())
			});
			assert_eq!(
				app.editor.document.lines,
				[if scenario == 0 { "" } else { "keep" }]
			);
			if scenario == 0 {
				app.editor.undo(false);
				assert_eq!(app.editor.document.lines, ["keep"]);
			}
		}
	}

	#[test]
	fn delayed_paste_cannot_overwrite_new_editing() {
		let mut app = App::default();
		let (sender, receiver) = mpsc::channel();
		app.editor_paste = Some(PendingPaste {
			target: Target::capture(&app.editor),
			mode: PasteMode::Styled,
			receiver,
		});
		app.paste_editor_text("new", PasteMode::Styled);
		sender.send(Ok("stale".into())).unwrap();
		assert!(app.poll_editor_paste());
		assert_eq!(app.editor.document.lines, ["new"]);
	}
}
