use std::time::Instant;

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

use crate::{
	document::sanitize,
	editor::{EditKind, Editor, Movement},
};

#[derive(Default, PartialEq, Eq)]
pub enum Overlay {
	#[default]
	None,
	Help,
	Quit,
}

#[derive(Default)]
pub struct App {
	pub editor: Editor,
	pub overlay: Overlay,
	pub editor_area: Rect,
}

impl App {
	/// Returns true only when the user has chosen to exit.
	pub fn handle(&mut self, event: Event) -> bool {
		match event {
			Event::Key(key) if key.kind != KeyEventKind::Release => {
				let control = key.modifiers.contains(KeyModifiers::CONTROL);
				if self.overlay == Overlay::Quit {
					if matches!(key.code, KeyCode::Char('y' | 'Y')) {
						return true;
					}
					if matches!(key.code, KeyCode::Esc | KeyCode::Char('n' | 'N')) {
						self.overlay = Overlay::None;
					}
					return false;
				}
				if self.overlay == Overlay::Help {
					if matches!(key.code, KeyCode::Esc | KeyCode::F(1)) {
						self.overlay = Overlay::None;
					}
					return false;
				}
				if control && matches!(key.code, KeyCode::Char('q' | 'Q')) {
					self.editor.break_group();
					if self.editor.document.is_empty() {
						return true;
					}
					self.overlay = Overlay::Quit;
					return false;
				}
				if key.code == KeyCode::F(1) {
					self.editor.break_group();
					self.overlay = Overlay::Help;
					return false;
				}
				let now = Instant::now();
				match (key.code, control) {
					(KeyCode::Char('z' | 'Z'), true) => {
						self.editor
							.undo(key.modifiers.contains(KeyModifiers::SHIFT));
					}
					(KeyCode::Char('y' | 'Y'), true) => self.editor.undo(true),
					(KeyCode::Home, true) => self.editor.move_cursor(Movement::Start),
					(KeyCode::End, true) => self.editor.move_cursor(Movement::Finish),
					(KeyCode::Left, false) => self.editor.move_cursor(Movement::Left),
					(KeyCode::Right, false) => self.editor.move_cursor(Movement::Right),
					(KeyCode::Up, false) => self.editor.move_cursor(Movement::Up),
					(KeyCode::Down, false) => self.editor.move_cursor(Movement::Down),
					(KeyCode::Home, false) => self.editor.move_cursor(Movement::Home),
					(KeyCode::End, false) => self.editor.move_cursor(Movement::End),
					(KeyCode::PageUp, false) => self.editor.move_cursor(Movement::PageUp),
					(KeyCode::PageDown, false) => self.editor.move_cursor(Movement::PageDown),
					(KeyCode::Backspace, false) => self.editor.delete(true, now),
					(KeyCode::Delete, false) => self.editor.delete(false, now),
					(KeyCode::Enter, false) => self.editor.insert("\n", EditKind::Atomic, now),
					(KeyCode::Tab, false) => self.editor.insert("\t", EditKind::Atomic, now),
					(KeyCode::Char(ch), false)
						if !key.modifiers.contains(KeyModifiers::ALT) && !ch.is_control() =>
					{
						self.editor.insert(&ch.to_string(), EditKind::Typing, now)
					}
					_ => self.editor.break_group(),
				}
			}
			Event::Paste(text) if self.overlay == Overlay::None => {
				self.editor
					.insert(&sanitize(&text), EditKind::Atomic, Instant::now())
			}
			Event::Mouse(mouse)
				if self.overlay == Overlay::None
					&& self.editor_area.contains((mouse.column, mouse.row).into()) =>
			{
				match mouse.kind {
					MouseEventKind::Down(MouseButton::Left) => self.editor.click(
						(mouse.column - self.editor_area.x) as usize,
						(mouse.row - self.editor_area.y) as usize,
					),
					MouseEventKind::ScrollDown => self.editor.scroll(true),
					MouseEventKind::ScrollUp => self.editor.scroll(false),
					_ => (),
				}
			}
			Event::Resize(_, _) | Event::FocusLost => self.editor.break_group(),
			_ => (),
		}
		false
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crossterm::event::KeyEvent;

	fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
		Event::Key(KeyEvent::new(code, modifiers))
	}

	#[test]
	fn quit_is_cancelable_and_paste_cannot_answer_a_dialog() {
		let mut app = App::default();
		app.handle(Event::Paste("hello".into()));
		assert!(!app.handle(key(KeyCode::Char('q'), KeyModifiers::CONTROL)));
		assert!(app.overlay == Overlay::Quit);
		assert!(!app.handle(Event::Paste("y".into())));
		assert_eq!(app.editor.document.lines, ["hello"]);
		app.handle(key(KeyCode::Esc, KeyModifiers::NONE));
		assert!(app.overlay == Overlay::None);
		app.handle(key(KeyCode::Char('q'), KeyModifiers::CONTROL));
		assert!(app.handle(key(KeyCode::Char('y'), KeyModifiers::NONE)));
	}

	#[test]
	fn help_blocks_edits_and_empty_editor_exits_directly() {
		let mut app = App::default();
		app.handle(key(KeyCode::F(1), KeyModifiers::NONE));
		app.handle(Event::Paste("ignored".into()));
		app.handle(key(KeyCode::Char('x'), KeyModifiers::NONE));
		assert!(app.editor.document.is_empty());
		app.handle(key(KeyCode::Esc, KeyModifiers::NONE));
		assert!(app.handle(key(KeyCode::Char('q'), KeyModifiers::CONTROL)));
	}
}
