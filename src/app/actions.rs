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
		if self.workspace.focus == Pane::Tools && self.workspace.panel.overlay_action(action) {
			return;
		}
		// The sidebar closes its highlighted buffer; the File menu closes the active one.
		if self.workspace.focus == Pane::Sidebar
			&& action == crate::shortcuts::ActionId::CloseBuffer
		{
			let effect = self.workspace.sidebar_action(action);
			self.workspace_effect(effect);
			return;
		}
		if self.file_command(action) || self.workspace.pane_action(action) {
			return;
		}
		// These commands deliberately remain eligible across workspace scopes.
		if self
			.workspace
			.active
			.editor
			.handle_action(&mut self.workspace.active.session.editing, action)
		{
			return;
		}
		match self.workspace.focus {
			Pane::Sidebar => {
				let effect = self.workspace.sidebar_action(action);
				self.workspace_effect(effect);
			}
			Pane::Editor => self.editor_clipboard_action(action),
			Pane::Code => {
				if let Some(notice) = self.workspace.active.code.handle_action(
					action,
					&self.workspace.active.session.editing.document,
					&self.clipboard,
				) {
					self.notice = Some(notice);
				}
			}
			Pane::Tools => {
				let effect = self.workspace.panel_action(action);
				self.workspace_effect(effect);
			}
		}
	}
}
