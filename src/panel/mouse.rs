use super::{Control, Effect, Panel};
use crate::libraries::controls::Range;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
impl Panel {
	fn set_scroll(&mut self, target: super::scroll::Target, offset: usize) {
		use super::scroll::Target;
		match target {
			Target::Tools => self.top = offset,
			Target::Indexed => self.color.indexed_top = offset,
		}
	}

	pub fn mouse(&mut self, mouse: MouseEvent) -> Option<Effect> {
		let point = (mouse.column, mouse.row).into();
		let gauge = if mouse.kind == MouseEventKind::Drag(MouseButton::Left) {
			self.color.gauge_drag.and_then(|field| {
				self.color
					.gauge_areas
					.iter()
					.find(|(_, f)| *f == field)
					.copied()
			})
		} else {
			self.color
				.gauge_areas
				.iter()
				.find(|(area, _)| area.contains(point))
				.copied()
		};
		if let Some((area, field)) = gauge {
			let value = match mouse.kind {
				MouseEventKind::Down(MouseButton::Left)
				| MouseEventKind::Drag(MouseButton::Left) => {
					self.color.gauge_drag = Some(field);
					Some(Range { min: 0, max: 255 }.at_cell(
						mouse.column.saturating_sub(area.x) as usize,
						area.width as usize,
					))
				}
				MouseEventKind::ScrollUp => Some(
					self.color.rgb.fields[field]
						.text()
						.parse::<u8>()
						.unwrap_or(0)
						.saturating_add(1),
				),
				MouseEventKind::ScrollDown => Some(
					self.color.rgb.fields[field]
						.text()
						.parse::<u8>()
						.unwrap_or(0)
						.saturating_sub(1),
				),
				_ => None,
			};
			if let Some(value) = value {
				if let Some(index) = self
					.items()
					.iter()
					.position(|item| item.control == Control::Field(field))
				{
					self.selected = index;
				}
				self.color.rgb.set_component(field, value);
				return Some(Effect::Focus);
			}
		}

		if mouse.kind == MouseEventKind::Drag(MouseButton::Left)
			&& let Some((target, grab)) = self.scrollbar_drag
		{
			if let Some(bar) = self
				.scrollbars
				.iter()
				.find(|bar| bar.target == target)
				.copied()
			{
				self.set_scroll(
					target,
					bar.offset_at(bar.coordinate(mouse.column, mouse.row), grab),
				);
			}
			return Some(Effect::None);
		}
		if let Some(bar) = self
			.scrollbars
			.iter()
			.find(|bar| bar.area.contains(point))
			.copied()
		{
			match mouse.kind {
				MouseEventKind::Down(MouseButton::Left) => {
					let at = bar.coordinate(mouse.column, mouse.row);
					let (start, size) = bar.thumb();
					let grab = bar.grab_at(at);
					self.scrollbar_drag = Some((bar.target, grab));
					if !(start..start + size).contains(&at) {
						self.set_scroll(bar.target, bar.offset_at(at, grab));
					}
					return Some(Effect::None);
				}
				MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
					let offset = if mouse.kind == MouseEventKind::ScrollDown {
						bar.offset
							.saturating_add(3)
							.min(bar.content.saturating_sub(bar.viewport))
					} else {
						bar.offset.saturating_sub(3)
					};
					self.set_scroll(bar.target, offset);
					return Some(Effect::None);
				}
				_ => (),
			}
		}

		if mouse.kind == MouseEventKind::Down(MouseButton::Left)
			&& let Some((_, tool)) = self
				.tool_areas
				.iter()
				.find(|(area, _)| area.contains(point))
		{
			self.selected = *tool;
			return Some(self.activate());
		}
		if matches!(
			mouse.kind,
			MouseEventKind::ScrollDown | MouseEventKind::ScrollUp
		) {
			let down = mouse.kind == MouseEventKind::ScrollDown;
			if self.color.indexed_area.contains(point) {
				self.color.indexed_top = if down {
					(self.color.indexed_top + 1).min(32)
				} else {
					self.color.indexed_top.saturating_sub(1)
				};
			} else if self.area.contains(point) {
				self.top = if down {
					self.top.saturating_add(3)
				} else {
					self.top.saturating_sub(3)
				};
			} else {
				return None;
			}
			return Some(Effect::None);
		}
		None
	}
}
