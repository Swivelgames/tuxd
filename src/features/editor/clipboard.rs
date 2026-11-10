//! Clipboard transactions retain their original editor target until completion.
use crate::features::clipboard_notice;
use crate::libraries::clipboard::{ReadRequest, WriteRequest};
use crate::{
	libraries::ansi::TextStyle,
	libraries::ansi::{Document, Position},
};
use std::io::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasteMode {
	Styled,
	Plain,
	Literal,
}

struct Target {
	session: crate::session::SessionId,
	revision: u64,
	document: Document,
	cursor: Position,
	anchor: Option<Position>,
	pending: Option<TextStyle>,
}
impl Target {
	fn capture(session: &crate::session::DocumentSession) -> Self {
		let editor = &session.editing;
		Self {
			session: session.id,
			revision: editor.revision,
			document: editor.document.clone(),
			cursor: editor.cursor,
			anchor: editor.anchor,
			pending: editor.pending,
		}
	}
	fn matches(&self, session: &crate::session::DocumentSession) -> bool {
		let editor = &session.editing;
		self.session == session.id
			&& self.revision == editor.revision
			&& self.document == editor.document
			&& self.cursor == editor.cursor
			&& self.anchor == editor.anchor
			&& self.pending == editor.pending
	}
}

struct PendingCopy {
	request: WriteRequest,
	target: Option<Target>,
}
impl PendingCopy {
	pub fn cancel_cut(&mut self) {
		self.target = None;
	}
}

struct PendingPaste {
	target: Target,
	mode: PasteMode,
	request: ReadRequest,
}
pub struct PasteReview {
	target: Target,
	formatted: bool,
}

/// Only unescaped ESC notation triggers decoding; ordinary backslashes stay literal.
pub(crate) fn import_mode(text: &str) -> Option<crate::libraries::ansi::Mode> {
	if text
		.chars()
		.any(|ch| ch.is_control() && !matches!(ch, '\n' | '\r' | '\t'))
	{
		return Some(crate::libraries::ansi::Mode::Raw);
	}
	let bytes = text.as_bytes();
	let mut at = 0;
	while at < bytes.len() {
		if bytes[at..].starts_with(b"\\\\") {
			at += 2;
		} else if bytes[at..].starts_with(b"\\x1b[") || bytes[at..].starts_with(b"\\x1B[") {
			return Some(crate::libraries::ansi::Mode::Escaped);
		} else {
			at += 1;
		}
	}
	None
}

pub enum Effect {
	None,
	Notice(String),
	Review {
		import: crate::libraries::ansi::Import,
		target: PasteReview,
	},
}
#[derive(Default)]
pub struct Clipboard {
	copy: Option<PendingCopy>,
	paste: Option<PendingPaste>,
}
impl Clipboard {
	pub fn cancel_edits(&mut self) {
		if let Some(copy) = &mut self.copy {
			copy.cancel_cut();
		}
		self.paste = None;
	}
	#[cfg(test)]
	pub fn copying(&self) -> bool {
		self.copy.is_some()
	}
	#[cfg(test)]
	pub fn pasting(&self) -> bool {
		self.paste.is_some()
	}
	pub fn copy(
		&mut self,
		session: &crate::session::DocumentSession,
		service: &crate::libraries::clipboard::Service,
		cut: bool,
	) -> Effect {
		if self.copy.is_some() {
			return Effect::None;
		}
		let Some(document) = session.editing.selected_document() else {
			return Effect::None;
		};
		let Some(request) = service.write(document.to_escaped_ansi()) else {
			return Effect::None;
		};
		self.copy = Some(PendingCopy {
			request,
			target: cut.then(|| Target::capture(session)),
		});
		Effect::Notice("Copying escaped ANSI…".into())
	}
	pub fn poll_copy(
		&mut self,
		session: &mut crate::session::DocumentSession,
		editor: &mut super::Editor,
		available: bool,
		output: &mut impl Write,
	) -> Option<Effect> {
		let completion = self.copy.as_mut()?.request.poll(output)?;
		let result = completion.result;
		if let Some(target) = self.copy.take().and_then(|pending| pending.target)
			&& result.is_ok()
		{
			if target.matches(session) && available {
				editor.delete(&mut session.editing, true, std::time::Instant::now());
			} else {
				return Some(Effect::Notice(
					"Copied selection; cut canceled because the editor changed".into(),
				));
			}
		}
		Some(Effect::Notice(clipboard_notice::copied(
			result,
			"escaped ANSI",
		)))
	}
	pub fn request_paste(
		&mut self,
		session: &crate::session::DocumentSession,
		service: &crate::libraries::clipboard::Service,
		mode: PasteMode,
	) {
		if self.paste.is_none() {
			self.paste = Some(PendingPaste {
				target: Target::capture(session),
				mode,
				request: service.read(),
			});
		}
	}
	pub fn poll_paste(
		&mut self,
		session: &mut crate::session::DocumentSession,
		editor: &mut super::Editor,
		available: bool,
	) -> Option<Effect> {
		let completion = self.paste.as_mut()?.request.poll()?;
		let pending = self.paste.take().unwrap();
		Some(match completion.result {
			Ok(text) if pending.target.matches(session) && available => {
				paste_text(session, editor, &text, pending.mode)
			}
			Ok(_) => Effect::Notice("Paste canceled because the editor changed".into()),
			Err(error) => Effect::Notice(clipboard_notice::paste_error(error)),
		})
	}
}
pub fn paste_text(
	session: &mut crate::session::DocumentSession,
	editor: &mut super::Editor,
	text: &str,
	mode: PasteMode,
) -> Effect {
	let detected = if mode == PasteMode::Literal {
		None
	} else {
		import_mode(text)
	};
	if let Some(detected) = detected {
		match crate::libraries::ansi::parse(text, detected) {
			Ok(import) if import.issues.is_empty() => editor.paste_document(
				&mut session.editing,
				&import.document,
				mode == PasteMode::Styled,
			),
			Ok(import) => {
				let target = PasteReview {
					target: Target::capture(session),
					formatted: mode == PasteMode::Styled,
				};
				return Effect::Review { import, target };
			}
			Err(error) => return Effect::Notice(format!("Paste failed: {error}")),
		}
	} else {
		let text = crate::libraries::text::display::sanitize(text);
		let mut document = Document::default();
		document
			.insert(Position::default(), &text)
			.expect("sanitized paste");
		editor.paste_document(&mut session.editing, &document, false);
	}
	Effect::None
}

impl PasteReview {
	pub fn session(&self) -> crate::session::SessionId {
		self.target.session
	}

	pub fn apply(
		self,
		session: &mut crate::session::DocumentSession,
		editor: &mut super::Editor,
		document: &Document,
	) -> Effect {
		if self.target.matches(session) {
			editor.paste_document(&mut session.editing, document, self.formatted);
			Effect::None
		} else {
			Effect::Notice("Paste canceled because the editor changed".into())
		}
	}
}
#[cfg(test)]
mod session_target_tests {
	use super::*;
	use crate::session::DocumentSession;

	#[test]
	fn equal_content_and_revision_cannot_make_a_replacement_session_match() {
		let first = DocumentSession::default();
		let target = Target::capture(&first);
		let replacement = DocumentSession::default();
		assert_eq!(first.editing.revision, replacement.editing.revision);
		assert_eq!(first.editing.document, replacement.editing.document);
		assert!(target.matches(&first));
		assert!(!target.matches(&replacement));
	}
}
