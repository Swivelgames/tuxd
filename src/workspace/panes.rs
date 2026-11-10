use super::Workspace;
use crate::workspace::{Pane, TextPane};

impl Workspace {
	pub(crate) fn toggle_wrap(&mut self, pane: TextPane) {
		if pane == TextPane::Code {
			self.active.code.surface.viewport.toggle_wrap();
		} else {
			self.active
				.editor
				.toggle_wrap(&mut self.active.session.editing);
		}
	}

	pub fn toggle_pane(&mut self, pane: TextPane) {
		if pane == TextPane::Code {
			self.code_open = !self.code_open;
			if !self.code_open {
				self.editor_open = true;
				if self.focus == Pane::Code || self.slim {
					self.focus = Pane::Editor;
				}
			}
		} else {
			self.editor_open = !self.editor_open;
			if !self.editor_open {
				self.code_open = true;
				if self.focus == Pane::Editor {
					self.focus = if self.slim && self.slim_tab > 0 {
						Pane::Tools
					} else {
						Pane::Code
					};
				}
			}
		}
		self.dragging = false;
		self.panel.scrollbar_drag = None;
		self.active.editor.surface.cancel_drag();
		self.active.code.surface.cancel_drag();
		self.active.session.editing.break_group();
	}

	pub fn select_slim_tab(&mut self, tab: usize) {
		self.slim_tab = tab;
		self.code_open = true;
		self.focus = if tab == 0 { Pane::Code } else { Pane::Tools };
		self.panel.active_section = tab.checked_sub(1);
		self.panel.top = 0;
		self.panel.selected = 0;
		self.panel.color.gauge_drag = None;
		self.panel.scrollbar_drag = None;
		self.active.editor.surface.cancel_drag();
		self.active.code.surface.cancel_drag();
		self.active.session.editing.break_group();
	}

	pub(crate) fn pane_action(&mut self, action: crate::shortcuts::ActionId) -> bool {
		use crate::shortcuts::ActionId as A;
		match action {
			A::ToggleCode => self.toggle_pane(TextPane::Code),
			A::ToggleEditor => self.toggle_pane(TextPane::Editor),
			A::ClearSelection => {
				self.active.session.editing.anchor = None;
				self.focus = Pane::Editor;
				self.editor_open = true;
				self.dragging = false;
				self.active.session.editing.break_group();
			}
			A::NextTab | A::PreviousTab => {
				if self.slim {
					self.select_slim_tab(
						(self.slim_tab + if action == A::PreviousTab { 3 } else { 1 }) % 4,
					);
				}
			}
			A::CyclePanes => {
				if self.slim {
					if self.focus == Pane::Editor {
						self.select_slim_tab(self.slim_tab);
					} else {
						self.focus = Pane::Editor;
						self.editor_open = true;
					}
				} else {
					self.focus = self.focus.next();
					if self.focus == Pane::Editor {
						self.editor_open = true;
					}
					if self.focus == Pane::Code {
						self.code_open = true;
					}
					if self.focus == Pane::Tools {
						self.reveal_tool();
					}
				}
				self.active.session.editing.break_group();
			}
			A::FocusEditor => {
				self.focus = Pane::Editor;
				self.editor_open = true;
				self.active.session.editing.break_group();
			}
			A::FocusCode => {
				if self.slim {
					self.select_slim_tab(0);
				} else {
					self.focus = Pane::Code;
					self.code_open = true;
				}
				self.active.session.editing.break_group();
			}
			A::FocusTools => {
				if self.slim {
					self.select_slim_tab(self.slim_tab.max(1));
				} else {
					self.focus = Pane::Tools;
					self.reveal_tool();
				}
				self.active.session.editing.break_group();
			}
			_ => return false,
		}
		true
	}
}
