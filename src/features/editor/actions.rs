//! Typed shortcut handler for editing and navigation.
use super::Editor;
use crate::{
	libraries::editing::{Editing, Movement},
	shortcuts::ActionId as A,
};

fn movement(action: A) -> Option<(Movement, bool)> {
	Some(match action {
		A::MoveLeft => (Movement::Left, false),
		A::SelectLeft => (Movement::Left, true),
		A::MoveRight => (Movement::Right, false),
		A::SelectRight => (Movement::Right, true),
		A::MoveUp => (Movement::Up, false),
		A::SelectUp => (Movement::Up, true),
		A::MoveDown => (Movement::Down, false),
		A::SelectDown => (Movement::Down, true),
		A::MoveHome => (Movement::Home, false),
		A::SelectHome => (Movement::Home, true),
		A::MoveEnd => (Movement::End, false),
		A::SelectEnd => (Movement::End, true),
		A::MovePageUp => (Movement::PageUp, false),
		A::SelectPageUp => (Movement::PageUp, true),
		A::MovePageDown => (Movement::PageDown, false),
		A::SelectPageDown => (Movement::PageDown, true),
		A::MoveStart => (Movement::Start, false),
		A::SelectStart => (Movement::Start, true),
		A::MoveFinish => (Movement::Finish, false),
		A::SelectFinish => (Movement::Finish, true),
		A::MoveWordLeft => (Movement::WordLeft, false),
		A::SelectWordLeft => (Movement::WordLeft, true),
		A::MoveWordRight => (Movement::WordRight, false),
		A::SelectWordRight => (Movement::WordRight, true),
		_ => return None,
	})
}
impl Editor {
	pub fn handle_action(&mut self, editing: &mut Editing, action: A) -> bool {
		if let Some((movement, selecting)) = movement(action) {
			self.navigate(editing, movement, selecting);
			return true;
		}
		let now = std::time::Instant::now();
		use crate::libraries::{ansi::Action as StyleAction, editing::EditKind};
		match action {
			A::Bold => editing.apply_tool(StyleAction::Bold),
			A::Italic => editing.apply_tool(StyleAction::Italic),
			A::Underline => editing.apply_tool(StyleAction::Underline),
			A::Undo => self.undo(editing, false),
			A::Redo => self.undo(editing, true),
			A::DeleteBefore => self.delete(editing, true, now),
			A::DeleteAfter => self.delete(editing, false, now),
			A::DeleteWord => self.delete_word(editing, now),
			A::InsertNewline => self.insert(editing, "\n", EditKind::Atomic, now),
			A::InsertTab => self.insert(editing, "\t", EditKind::Atomic, now),
			_ => return false,
		}
		true
	}
}
