use super::{Control, Panel};
use crate::libraries::ansi::Action;
use crate::libraries::controls::{self, Direction, Range};

pub enum Effect {
	None,
	Style(Action),
	Glyph(char),
	Focus,
	Copy(String),
	Paste(usize),
	Notice(String),
}

impl Panel {
	pub fn insert_text(&mut self, text: &str) {
		if let Some(item) = self.items().get(self.selected)
			&& let Control::Field(field) = item.control
		{
			self.color.rgb.edit(field, text, false);
		}
	}

	pub fn reveal(&mut self) {
		if let Some(item) = self.items().get(self.selected) {
			if let Some(crate::libraries::ansi::Ink::Indexed(n)) = item.ink {
				let row = n as usize / 7;
				if row < self.color.indexed_top {
					self.color.indexed_top = row;
				}
				if row >= self.color.indexed_top + 5 {
					self.color.indexed_top = row - 4;
				}
			}
			if let Some(row) = self.display_row(item) {
				if row < self.top {
					self.top = row;
				}
				if row >= self.top + self.height.max(1) {
					self.top = row + 1 - self.height.max(1);
				}
			}
		}
	}

	pub fn activate(&mut self) -> Effect {
		let Some(control) = self.items().get(self.selected).map(|item| item.control) else {
			return Effect::None;
		};
		let effect = match control {
			Control::Section(section) => {
				self.toggle_section(section);
				Effect::None
			}
			Control::Style(action) => Effect::Style(action),
			Control::Target(background) => {
				self.color.background = background;
				Effect::None
			}
			Control::Palette(tab) => {
				self.color.palette = tab;
				Effect::None
			}
			Control::GlyphTab(tab) => {
				self.characters.tab = tab;
				Effect::None
			}
			Control::Glyph(ch) => Effect::Glyph(ch),
			Control::Field(_) => {
				self.color.rgb.replace_field = true;
				Effect::Focus
			}
			Control::Apply => self.apply_rgb(),
		};
		self.reveal();
		effect
	}
	pub fn apply_rgb(&mut self) -> Effect {
		match self.color.rgb.rgb() {
			Ok(ink) => {
				self.color.rgb.error = None;
				Effect::Style(self.color.color_action(ink))
			}
			Err(error) => {
				self.color.rgb.error = Some(error.clone());
				Effect::Notice(error)
			}
		}
	}
	pub fn handle(&mut self, action: crate::shortcuts::ActionId) -> Effect {
		use crate::shortcuts::ActionId as A;
		if action == A::Copy {
			if let Some(item) = self.items().get(self.selected)
				&& let Control::Field(field) = item.control
				&& self.color.rgb.replace_field
			{
				return Effect::Copy(self.color.rgb.fields[field].text().to_owned());
			}
			return Effect::None;
		}
		let items = self.items();
		if items.is_empty() {
			return Effect::None;
		}
		self.selected = self.selected.min(items.len() - 1);
		let current = &items[self.selected];
		if matches!(action, A::NextField | A::PreviousField) {
			self.selected = self.next_field(self.selected, action == A::PreviousField);
		} else if let Control::Field(field) = current.control {
			let value = self.color.rgb.fields[field]
				.text()
				.parse::<u8>()
				.unwrap_or(0);
			let adjusted = match action {
				A::RangeDecrease => Some(Range { min: 0, max: 255 }.step(value, -1)),
				A::RangeIncrease => Some(Range { min: 0, max: 255 }.step(value, 1)),
				A::RangeDecrease10 => Some(Range { min: 0, max: 255 }.step(value, -10)),
				A::RangeIncrease10 => Some(Range { min: 0, max: 255 }.step(value, 10)),
				A::RangeMinimum => Some(0),
				A::RangeMaximum => Some(255),
				_ => None,
			};
			if let Some(value) = adjusted {
				self.color.rgb.set_component(field, value);
				return Effect::None;
			}
			match action {
				A::RgbApply | A::Submit => {
					let effect = self.apply_rgb();
					self.reveal();
					return effect;
				}
				A::SelectAll => self.color.rgb.replace_field = true,
				A::FieldDeleteBefore => self.color.rgb.edit(field, "", true),
				A::FieldLeft | A::FieldRight | A::FieldHome | A::FieldEnd => {
					self.color.rgb.move_caret(field, action)
				}
				A::FieldUp | A::FieldDown => {
					self.selected = self.next_field(self.selected, action == A::FieldUp)
				}
				A::Paste => {
					self.reveal();
					return Effect::Paste(field);
				}
				_ => (),
			}
			self.reveal();
			return Effect::None;
		} else if action == A::Activate {
			return self.activate();
		} else {
			let group = current.group();
			let members: Vec<_> = items
				.iter()
				.enumerate()
				.filter(|(_, item)| item.group() == group)
				.collect();
			let at = members
				.iter()
				.position(|(i, _)| *i == self.selected)
				.unwrap_or(0);
			let columns = current.columns.max(1);
			let grid = matches!(
				group,
				crate::panel::Group::Colors(_) | crate::panel::Group::Glyphs(_)
			);
			let rows = if matches!(current.ink, Some(crate::libraries::ansi::Ink::Indexed(_))) {
				5
			} else {
				(self.height / 2).max(1)
			};
			let direction = match action {
				A::FieldHome => Direction::Beginning,
				A::FieldEnd => Direction::End,
				A::FieldLeft => Direction::Previous,
				A::FieldRight => Direction::Next,
				A::FieldUp if grid => Direction::Up,
				A::FieldDown if grid => Direction::Down,
				A::FieldPageUp if grid => Direction::PageUp,
				A::FieldPageDown if grid => Direction::PageDown,
				A::FieldUp | A::FieldDown | A::FieldPageUp | A::FieldPageDown => {
					self.selected = self
						.next_field(self.selected, matches!(action, A::FieldUp | A::FieldPageUp));
					self.reveal();
					return Effect::None;
				}
				_ => return Effect::None,
			};
			let next = controls::navigate(at, members.len(), columns, rows, direction);
			self.selected = members[next].0;
			if next != at
				&& matches!(
					current.control,
					Control::Target(_) | Control::Palette(_) | Control::GlyphTab(_)
				) {
				return self.activate();
			}
		}
		self.color.rgb.replace_field = true;
		self.reveal();
		Effect::None
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::shortcuts::ActionId as A;
	use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
	use ratatui::layout::Rect;

	#[test]
	fn gauge_pointer_and_keyboard_update_the_same_draft_without_applying() {
		let mut panel = Panel::default();
		panel.color.palette = 2;
		panel.selected = panel
			.items()
			.iter()
			.position(|item| item.control == Control::Field(1))
			.unwrap();
		panel.color.gauge_areas.push((Rect::new(10, 10, 11, 1), 1));
		assert!(matches!(panel.handle(A::RangeMaximum), Effect::None));
		assert_eq!(panel.color.rgb.fields[1].text(), "255");
		assert!(matches!(
			panel.mouse(MouseEvent {
				kind: MouseEventKind::Down(MouseButton::Left),
				column: 10,
				row: 10,
				modifiers: KeyModifiers::NONE,
			}),
			Some(Effect::Focus)
		));
		assert_eq!(panel.color.rgb.fields[1].text(), "0");
		assert_eq!(panel.color.rgb.fields[0].text(), "#00c0ff");
		assert!(matches!(
			panel.handle(A::Submit),
			Effect::Style(Action::Foreground(crate::libraries::ansi::Ink::Rgb(
				0, 192, 255
			)))
		));
		panel.release_pointer();
		assert!(panel.color.gauge_drag.is_none());
	}

	#[test]
	fn arrangements_reuse_drafts_and_invalid_text_cannot_emit_a_color_command() {
		let mut panel = Panel::default();
		panel.color.palette = 2;
		panel.color.rgb.edit(0, "#12", false);
		assert!(matches!(panel.apply_rgb(), Effect::Notice(_)));
		panel.active_section = Some(1);
		assert_eq!(panel.color.rgb.fields[0].text(), "#12");
		assert!(
			panel
				.items()
				.iter()
				.any(|item| item.control == Control::Field(0))
		);
		panel.color.rgb.replace_field = true;
		panel.color.rgb.edit(0, "#123456", false);
		panel.active_section = None;
		assert_eq!(panel.color.rgb.fields[2].text(), "52");
		panel.color.background = true;
		assert!(matches!(
			panel.apply_rgb(),
			Effect::Style(Action::Background(crate::libraries::ansi::Ink::Rgb(
				18, 52, 86
			)))
		));
	}
}
