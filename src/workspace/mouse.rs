use super::Workspace;
use crate::workspace::{Pane, TextPane};
use crossterm::event::{MouseButton, MouseEventKind};

impl Workspace {
	pub(crate) fn mouse(&mut self, mouse: crossterm::event::MouseEvent) -> super::Effect {
		if self.panel.overlay_mouse(mouse) {
			return super::Effect::None;
		}
		let point = (mouse.column, mouse.row).into();
		let editor_bar = self.editor.surface.scrollbar_event(mouse);
		let code_bar = self.code.surface.scrollbar_event(mouse);
		if (editor_bar || code_bar) && mouse.kind != MouseEventKind::Up(MouseButton::Left) {
			self.dragging = false;
			return super::Effect::None;
		}
		if mouse.kind == MouseEventKind::Up(MouseButton::Left) {
			self.dragging = false;
			self.panel.release_pointer();
			return super::Effect::None;
		}
		if let Some(effect) = self.panel.mouse(mouse) {
			self.dragging = false;
			return self.tool_effect(effect);
		}

		if self.dragging
			&& mouse.kind == MouseEventKind::Drag(MouseButton::Left)
			&& self.editor.surface.text.width > 0
			&& self.editor.surface.text.height > 0
		{
			let area = self.editor.surface.text;
			self.editor.click(
				&mut self.session.editing,
				mouse
					.column
					.clamp(area.x, area.right() - 1)
					.saturating_sub(area.x) as usize,
				mouse
					.row
					.clamp(area.y, area.bottom() - 1)
					.saturating_sub(area.y) as usize,
			);
			return super::Effect::None;
		}
		if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
			self.dragging = false;
			if self.editor_wrap_toggle.contains(point) {
				self.toggle_wrap(TextPane::Editor);
				return super::Effect::None;
			}
			if self.code_wrap_toggle.contains(point) {
				self.toggle_wrap(TextPane::Code);
				return super::Effect::None;
			}
			if self.pane_toggles[0].contains(point) {
				self.toggle_pane(TextPane::Editor);
				return super::Effect::None;
			}
			if self.code_toggle.contains(point) || self.pane_toggles[1].contains(point) {
				self.toggle_pane(TextPane::Code);
				return super::Effect::None;
			}
			if let Some((_, tab)) = self
				.slim_tabs
				.iter()
				.find(|(rect, _)| rect.contains(point))
				.copied()
			{
				self.select_slim_tab(tab);
				return super::Effect::None;
			}

			if self.code.surface.inner.contains(point) {
				self.focus = Pane::Code;
				self.session.editing.break_group();
				return super::Effect::None;
			}
			if self.editor.surface.inner.contains(point) {
				self.focus = Pane::Editor;
				self.editor.click(
					&mut self.session.editing,
					mouse.column.saturating_sub(self.editor.surface.text.x) as usize,
					(mouse.row - self.editor.surface.text.y) as usize,
				);
				self.session.editing.anchor = Some(self.session.editing.cursor);
				self.dragging = true;
			}
		}
		if matches!(
			mouse.kind,
			MouseEventKind::ScrollDown | MouseEventKind::ScrollUp
		) {
			let down = mouse.kind == MouseEventKind::ScrollDown;
			if self.editor.surface.inner.contains(point) {
				self.editor.scroll(&mut self.session.editing, down);
			} else if self.code.surface.inner.contains(point) {
				self.code.surface.viewport.top = if down {
					self.code.surface.viewport.top.saturating_add(3)
				} else {
					self.code.surface.viewport.top.saturating_sub(3)
				};
			}
		}
		super::Effect::None
	}
}
