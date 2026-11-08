use super::{Dialog, Files, MENU};
use crossterm::event::{Event, MouseButton, MouseEventKind};

pub enum Effect {
	None,
	Cancel,
	Activate(usize),
	Submit(super::picker::Selection),
	Copy(String),
}
impl From<super::picker::Outcome> for Effect {
	fn from(value: super::picker::Outcome) -> Self {
		match value {
			super::picker::Outcome::None => Self::None,
			super::picker::Outcome::Cancel => Self::Cancel,
			super::picker::Outcome::Submit(selection) => Self::Submit(selection),
		}
	}
}
impl Files {
	pub fn shortcut(&mut self, action: crate::shortcuts::ActionId) -> Effect {
		use crate::shortcuts::ActionId as A;
		if action == A::Copy {
			return match &self.dialog {
				Some(Dialog::Path { picker }) => picker
					.selected_text()
					.map(Effect::Copy)
					.unwrap_or(Effect::None),
				_ => Effect::None,
			};
		}

		if let Some(Dialog::Path { picker }) = &mut self.dialog {
			return picker.shortcut(action).into();
		}

		if action == A::Close {
			return Effect::Cancel;
		}
		let mut activate = None;
		match &mut self.dialog {
			Some(Dialog::Menu { index }) => match action {
				A::NextField | A::FieldRight | A::FieldDown => {
					*index = (*index + 1).min(MENU.len() - 1)
				}
				A::PreviousField | A::FieldLeft | A::FieldUp => *index = index.saturating_sub(1),
				A::Activate => activate = Some(*index),
				A::FieldHome => *index = 0,
				A::FieldEnd => *index = MENU.len() - 1,
				_ => (),
			},
			Some(Dialog::Dirty) => match action {
				A::NextField | A::FieldRight | A::FieldDown => self.choice = (self.choice + 1) % 3,
				A::PreviousField | A::FieldLeft | A::FieldUp => self.choice = (self.choice + 2) % 3,
				A::Activate => activate = Some(self.choice),
				A::Positive => activate = Some(0),
				A::Negative => activate = Some(1),
				A::Abort => activate = Some(2),
				_ => (),
			},
			Some(Dialog::Overwrite { .. }) => match action {
				A::NextField | A::PreviousField | A::FieldLeft | A::FieldUp => {
					self.choice = 1 - self.choice
				}
				A::Activate => activate = Some(self.choice),
				A::Positive => activate = Some(0),
				A::Negative | A::Abort => activate = Some(1),
				_ => (),
			},
			_ => (),
		}
		activate.map(Effect::Activate).unwrap_or(Effect::None)
	}
	/// Non-key events. All shortcuts are resolved by the central registry first.
	pub fn event(&mut self, event: &Event) -> Effect {
		if let Some(Dialog::Path { picker }) = &mut self.dialog {
			return picker.event(event).into();
		}

		if let Event::Mouse(mouse) = event {
			match mouse.kind {
				MouseEventKind::ScrollUp => self.scroll = self.scroll.saturating_sub(3),
				MouseEventKind::ScrollDown => self.scroll = self.scroll.saturating_add(3),
				MouseEventKind::Up(MouseButton::Left) => self.drag = None,
				MouseEventKind::Down(MouseButton::Left) => {
					if let Some(bar) = self
						.bar
						.filter(|bar| bar.area.contains((mouse.column, mouse.row).into()))
					{
						let at = bar.coordinate(mouse.column, mouse.row);
						let grab = bar.grab_at(at);
						self.drag = Some(grab);
						self.scroll = bar.offset_at(at, grab);
					} else if let Some((_, choice)) = self
						.buttons
						.iter()
						.find(|(rect, _)| rect.contains((mouse.column, mouse.row).into()))
						.copied()
					{
						return Effect::Activate(choice);
					}
				}
				MouseEventKind::Drag(MouseButton::Left) => {
					if let (Some(bar), Some(grab)) = (self.bar, self.drag) {
						self.scroll = bar.offset_at(bar.coordinate(mouse.column, mouse.row), grab);
					}
				}
				_ => (),
			}
		}
		Effect::None
	}
}
