use crate::{
	libraries::controls::Direction,
	panel::{Control, Effect, Panel},
};

impl Panel {
	pub(in crate::panel) fn focus_character_control(&mut self, control: Control) {
		if let Some(index) = self.items().iter().position(|item| item.control == control) {
			self.selected = index;
		}
	}

	pub(in crate::panel) fn handle_characters(
		&mut self,
		action: crate::shortcuts::ActionId,
	) -> Option<Effect> {
		use crate::shortcuts::ActionId as A;
		let control = self.items().get(self.selected)?.control;
		if matches!(action, A::NextField | A::PreviousField) {
			return None;
		}
		if control == Control::CharacterSearch {
			match action {
				A::Submit => {
					if let Some(control) = self.characters.selected_control() {
						self.characters.reveal_selection();
						self.focus_character_control(control);
					}
				}
				A::Copy if self.characters.query.selected() => {
					return Some(Effect::Copy(self.characters.query.text().into()));
				}
				A::Paste => return Some(Effect::PasteCharacters),
				A::FieldUp | A::FieldDown => {
					self.selected = self.next_field(self.selected, action == A::FieldUp)
				}
				_ => {
					if let Some(edit) = crate::components::field::action(action) {
						let before = self.characters.query.text().to_owned();
						self.characters.query.apply(edit);
						if before != self.characters.query.text() {
							self.characters.refresh();
						}
					}
				}
			}
			return Some(Effect::None);
		}
		let direction = match action {
			A::FieldLeft => Direction::Previous,
			A::FieldRight => Direction::Next,
			A::FieldUp => Direction::Up,
			A::FieldDown => Direction::Down,
			A::FieldHome => Direction::Beginning,
			A::FieldEnd => Direction::End,
			A::FieldPageUp => Direction::PageUp,
			A::FieldPageDown => Direction::PageDown,
			_ => return None,
		};
		match control {
			Control::Glyph(_) | Control::CharacterType(_, _) => {
				self.characters.select(control);
				self.characters.navigate(direction);
				if let Some(control) = self.characters.selected_control() {
					self.focus_character_control(control);
				}
			}
			Control::CharacterGroup
				if matches!(
					action,
					A::FieldLeft | A::FieldRight | A::FieldHome | A::FieldEnd
				) =>
			{
				self.characters.choose_group(direction);
			}
			_ => return None,
		}
		Some(Effect::None)
	}
}

impl Panel {
	pub(in crate::panel) fn scroll_characters(&mut self, top: usize) {
		let focused = self.items().get(self.selected).is_some_and(|item| {
			matches!(
				item.control,
				Control::Glyph(_) | Control::CharacterType(_, _)
			)
		});
		self.characters.scroll(top);
		if focused {
			self.characters.select_visible();
			if let Some(control) = self.characters.selected_control() {
				self.focus_character_control(control);
			}
		}
	}
}
