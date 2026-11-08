use super::App;
use crate::features::editor::clipboard::PasteMode;

impl App {
	pub(crate) fn editor_clipboard_action(&mut self, action: crate::shortcuts::ActionId) {
		use crate::shortcuts::ActionId as A;
		match action {
			A::Copy => self.copy_selection(false),
			A::Cut => self.copy_selection(true),
			A::Paste => self.request_editor_paste(PasteMode::Styled),
			A::PastePlain => self.request_editor_paste(PasteMode::Plain),
			A::PasteLiteral => self.request_editor_paste(PasteMode::Literal),
			A::SelectAll => {
				use crate::libraries::editing::Movement;
				self.workspace.editor.navigate(
					&mut self.workspace.session.editing,
					Movement::Start,
					false,
				);
				self.workspace.editor.navigate(
					&mut self.workspace.session.editing,
					Movement::Finish,
					true,
				);
			}
			_ => (),
		}
	}

	pub fn copy_selection(&mut self, cut: bool) {
		let effect = self
			.editor_clipboard
			.copy(&self.workspace.session, &self.clipboard, cut);
		self.editor_clipboard_effect(effect);
	}

	pub fn poll_editor_copy(&mut self, output: &mut impl std::io::Write) -> bool {
		let available = self.workspace_available();
		let effect = self.editor_clipboard.poll_copy(
			&mut self.workspace.session,
			&mut self.workspace.editor,
			available,
			output,
		);
		if let Some(effect) = effect {
			self.editor_clipboard_effect(effect);
			true
		} else {
			false
		}
	}

	pub fn request_editor_paste(&mut self, mode: PasteMode) {
		self.editor_clipboard
			.request_paste(&self.workspace.session, &self.clipboard, mode);
	}

	pub fn poll_editor_paste(&mut self) -> bool {
		let available = self.workspace_available();
		let effect = self.editor_clipboard.poll_paste(
			&mut self.workspace.session,
			&mut self.workspace.editor,
			available,
		);
		if let Some(effect) = effect {
			self.editor_clipboard_effect(effect);
			true
		} else {
			false
		}
	}

	pub fn paste_editor_text(&mut self, text: &str, mode: PasteMode) {
		let effect = crate::features::editor::clipboard::paste_text(
			&mut self.workspace.session,
			&mut self.workspace.editor,
			text,
			mode,
		);
		self.editor_clipboard_effect(effect);
	}

	pub fn finish_paste_review(
		&mut self,
		review: crate::features::editor::clipboard::PasteReview,
		document: &crate::libraries::ansi::Document,
	) {
		let effect = review.apply(
			&mut self.workspace.session,
			&mut self.workspace.editor,
			document,
		);
		self.editor_clipboard_effect(effect);
	}

	fn editor_clipboard_effect(&mut self, effect: crate::features::editor::clipboard::Effect) {
		use crate::features::editor::clipboard::Effect;
		match effect {
			Effect::None => (),
			Effect::Notice(message) => self.notice = Some(message),
			Effect::Review { import, target } => self.begin_review(
				import,
				crate::workflows::Source::Clipboard,
				crate::workflows::Destination::Paste(target),
			),
		}
	}
}
