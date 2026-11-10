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
				self.workspace.active.editor.navigate(
					&mut self.workspace.active.session.editing,
					Movement::Start,
					false,
				);
				self.workspace.active.editor.navigate(
					&mut self.workspace.active.session.editing,
					Movement::Finish,
					true,
				);
			}
			_ => (),
		}
	}

	pub fn copy_selection(&mut self, cut: bool) {
		let effect = self.workspace.active.clipboard.copy(
			&self.workspace.active.session,
			&self.clipboard,
			cut,
		);
		self.editor_clipboard_effect(effect);
	}

	pub fn poll_editor_copy(&mut self, output: &mut impl std::io::Write) -> bool {
		let available = self.workspace_available();
		let effects: Vec<_> = self
			.workspace
			.buffers_mut()
			.filter_map(|buffer| {
				buffer.clipboard.poll_copy(
					&mut buffer.session,
					&mut buffer.editor,
					available,
					output,
				)
			})
			.collect();
		let updated = !effects.is_empty();
		for effect in effects {
			self.editor_clipboard_effect(effect);
		}
		updated
	}

	pub fn request_editor_paste(&mut self, mode: PasteMode) {
		self.workspace.active.clipboard.request_paste(
			&self.workspace.active.session,
			&self.clipboard,
			mode,
		);
	}

	pub fn poll_editor_paste(&mut self) -> bool {
		let available = self.workspace_available();
		let effect = self.workspace.buffers_mut().find_map(|buffer| {
			buffer
				.clipboard
				.poll_paste(&mut buffer.session, &mut buffer.editor, available)
		});
		if let Some(effect) = effect {
			self.editor_clipboard_effect(effect);
			true
		} else {
			false
		}
	}

	pub fn paste_editor_text(&mut self, text: &str, mode: PasteMode) {
		let effect = crate::features::editor::clipboard::paste_text(
			&mut self.workspace.active.session,
			&mut self.workspace.active.editor,
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
		let Some(buffer) = self
			.workspace
			.buffers_mut()
			.find(|buffer| buffer.session.id == review.session())
		else {
			return;
		};
		let effect = review.apply(&mut buffer.session, &mut buffer.editor, document);
		self.editor_clipboard_effect(effect);
	}

	fn editor_clipboard_effect(&mut self, effect: crate::features::editor::clipboard::Effect) {
		use crate::features::editor::clipboard::Effect;
		match effect {
			Effect::None => (),
			Effect::Notice(message) => self.notice = Some(message),
			Effect::Review { import, target } => {
				self.workspace.select_session(target.session());
				self.begin_review(
					import,
					crate::workflows::Source::Clipboard,
					crate::workflows::Destination::Paste(target),
				);
			}
		}
	}
}
