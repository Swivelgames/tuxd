//! Route resolved IDs to their owner; catalog eligibility is checked before execution.
use super::App;
use crate::workspace::Pane;

impl App {
	pub(super) fn execute_action(&mut self, action: crate::shortcuts::ActionId) {
		if let Some(review) = &mut self.import_review {
			if let Some(decision) = review.view.shortcut(action) {
				self.decide_review(decision);
			}
			return;
		}
		if self.files.dialog.is_some() {
			self.file_shortcut(action);
			return;
		}
		if self.file_command(action) || self.workspace.pane_action(action) {
			return;
		}
		// These commands deliberately remain eligible across workspace scopes.
		if self
			.workspace
			.editor
			.handle_action(&mut self.workspace.session.editing, action)
		{
			return;
		}
		match self.workspace.focus {
			Pane::Editor => self.editor_clipboard_action(action),
			Pane::Code => {
				if let Some(notice) = self.workspace.code.handle_action(
					action,
					&self.workspace.session.editing.document,
					&self.clipboard,
				) {
					self.notice = Some(notice);
				}
			}
			Pane::Tools => {
				let effect = self.workspace.panel_action(action);
				self.tool_effect(effect);
			}
		}
	}
}
