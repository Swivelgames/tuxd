use super::Workspace;
use crate::workspace::{Pane, TextPane};
use crossterm::event::{MouseButton, MouseEventKind};

impl Workspace {
	pub(crate) fn mouse(&mut self, mouse: crossterm::event::MouseEvent) -> super::Effect {
		if mouse.kind == MouseEventKind::Down(MouseButton::Left)
			&& self
				.sidebar_toggle
				.contains((mouse.column, mouse.row).into())
		{
			self.pane_action(crate::shortcuts::ActionId::ToggleSidebar);
			return super::Effect::None;
		}
		if let Some(effect) = self.sidebar_mouse(mouse) {
			return effect;
		}
		if self.panel.overlay_mouse(mouse) {
			return super::Effect::None;
		}
		let point = (mouse.column, mouse.row).into();
		if mouse.kind == MouseEventKind::Down(MouseButton::Left)
			&& let Some((_, id)) = self
				.tab_close_hits
				.iter()
				.find(|(rect, _)| rect.contains(point))
				.copied()
		{
			return super::Effect::CloseBuffer(id);
		}
		if mouse.kind == MouseEventKind::Down(MouseButton::Left)
			&& let Some((_, index)) = self
				.tab_hits
				.iter()
				.find(|(rect, _)| rect.contains(point))
				.copied()
		{
			self.select_buffer(index);
			return super::Effect::None;
		}
		let editor_bar = self.active.editor.surface.scrollbar_event(mouse);
		let code_bar = self.active.code.surface.scrollbar_event(mouse);
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
			&& self.active.editor.surface.text.width > 0
			&& self.active.editor.surface.text.height > 0
		{
			let area = self.active.editor.surface.text;
			self.active.editor.click(
				&mut self.active.session.editing,
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
			if self.code_wrap_toggle.contains(point) {
				self.toggle_wrap(TextPane::Code);
				return super::Effect::None;
			}
			if self.pane_toggles[0].contains(point) {
				self.toggle_pane(TextPane::Editor);
				return super::Effect::None;
			}
			if self.pane_toggles[1].contains(point) {
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

			if self.active.code.surface.inner.contains(point) {
				self.focus = Pane::Code;
				self.active.session.editing.break_group();
				return super::Effect::None;
			}
			if self.active.editor.surface.inner.contains(point) {
				self.focus = Pane::Editor;
				self.active.editor.click(
					&mut self.active.session.editing,
					mouse
						.column
						.saturating_sub(self.active.editor.surface.text.x) as usize,
					(mouse.row - self.active.editor.surface.text.y) as usize,
				);
				self.active.session.editing.anchor = Some(self.active.session.editing.cursor);
				self.dragging = true;
			}
		}
		if matches!(
			mouse.kind,
			MouseEventKind::ScrollDown | MouseEventKind::ScrollUp
		) {
			let down = mouse.kind == MouseEventKind::ScrollDown;
			if self.active.editor.surface.inner.contains(point) {
				self.active
					.editor
					.scroll(&mut self.active.session.editing, down);
			} else if self.active.code.surface.inner.contains(point) {
				self.active.code.surface.viewport.top = if down {
					self.active.code.surface.viewport.top.saturating_add(3)
				} else {
					self.active.code.surface.viewport.top.saturating_sub(3)
				};
			}
		}
		super::Effect::None
	}
}
