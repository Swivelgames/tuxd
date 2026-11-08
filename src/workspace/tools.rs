//! Workspace applies typed tool outcomes; Tools owns input, selection and drafts.
use super::Workspace;
use crate::workspace::Pane;
use crate::{libraries::editing::EditKind, panel::Effect as ToolEffect};
use std::time::Instant;

/// Effects requiring application-owned services; local tool effects never escape.
pub enum Effect {
	None,
	Copy(String),
	Paste(usize),
	Notice(String),
}

impl Workspace {
	pub fn reveal_tool(&mut self) {
		self.panel.reveal();
	}

	#[cfg(test)]
	pub(crate) fn activate_tool(&mut self) -> Effect {
		let effect = self.panel.activate();
		self.tool_effect(effect)
	}

	pub(crate) fn panel_action(&mut self, action: crate::shortcuts::ActionId) -> Effect {
		let effect = self.panel.handle(action);
		self.tool_effect(effect)
	}

	pub(crate) fn tool_effect(&mut self, effect: ToolEffect) -> Effect {
		match effect {
			ToolEffect::None => (),
			ToolEffect::Style(action) => self.session.editing.apply_tool(action),
			ToolEffect::Glyph(ch) => {
				self.focus = Pane::Tools;
				self.editor.insert(
					&mut self.session.editing,
					&ch.to_string(),
					EditKind::Atomic,
					Instant::now(),
				);
			}
			ToolEffect::Focus => self.focus = Pane::Tools,
			ToolEffect::Copy(text) => return Effect::Copy(text),
			ToolEffect::Paste(field) => return Effect::Paste(field),
			ToolEffect::Notice(message) => return Effect::Notice(message),
		}
		Effect::None
	}
}
