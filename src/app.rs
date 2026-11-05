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
	pub code_wrap: bool,
	pub editor_wrap_toggle: Rect,
	pub code_wrap_toggle: Rect,
	pub notice: Option<String>,
	clipboard_result: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
	pub focus: usize,
	pub code_open: bool,
	pub code_top: usize,
	pub code_left: usize,
	pub code_area: Rect,
	pub code_pane: Rect,
	pub editor_pane: Rect,
	pub code_toggle: Rect,
	pub tool_areas: Vec<(Rect, usize)>,
	pub tool_index: usize,
	pub dragging: bool,

	pub editor: Editor,
	pub overlay: Overlay,
	pub editor_area: Rect,
}

impl App {
	pub fn poll_clipboard(&mut self) -> bool {
		let was_pending = self.clipboard_result.is_some();
		if let Some(receiver) = &self.clipboard_result {
			match receiver.try_recv() {
				Ok(result) => {
					self.notice = Some(match result {
						Ok(()) => "Copied escaped ANSI to clipboard".into(),
						Err(message) => message,
					});
					self.clipboard_result = None;
				}
				Err(std::sync::mpsc::TryRecvError::Disconnected) => {
					self.notice = Some("Clipboard copy failed".into());
					self.clipboard_result = None;
				}
				Err(std::sync::mpsc::TryRecvError::Empty) => (),
			}
		}
		was_pending && self.clipboard_result.is_none()
	}

	fn copy_code(&mut self) {
		if self.clipboard_result.is_some() {
			return;
		}
		let text = crate::styling::clipboard_text(&self.editor.document);
		let (sender, receiver) = std::sync::mpsc::channel();
		self.clipboard_result = Some(receiver);
		self.notice = Some("Copying escaped ANSI…".into());
		std::thread::spawn(move || {
			let _ = sender.send(crate::clipboard::copy(&text));
		});
	}

	fn toggle_wrap(&mut self, code: bool) {
		if code {
			self.code_wrap = !self.code_wrap;
			self.code_left = 0;
			self.code_top = 0;
		} else {
			self.editor.toggle_wrap();
		}
	}

	fn workspace_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
		let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
		match key.code {
			KeyCode::F(3) => {
				if self.focus != 1 {
					self.toggle_wrap(self.focus == 2);
				}
				return true;
			}
			KeyCode::Char('c' | 'C') if ctrl && self.focus == 2 => {
				self.copy_code();
				return true;
			}
			KeyCode::F(2) => {
				self.code_open = !self.code_open;
				if !self.code_open && self.focus == 2 {
					self.focus = 0;
				}
				self.editor.break_group();
				return true;
			}
			KeyCode::F(6) => {
				self.focus = (self.focus + 1) % if self.code_open { 3 } else { 2 };
				self.editor.break_group();
				return true;
			}
			KeyCode::Esc => {
				self.editor.anchor = None;
				self.focus = 0;
				self.dragging = false;
				self.editor.break_group();
				return true;
			}
			KeyCode::Char(ch) if ctrl && "bBiIuU".contains(ch) => {
				self.editor.apply_tool(match ch.to_ascii_lowercase() {
					'b' => 0,
					'i' => 1,
					_ => 2,
				});
				return true;
			}
			KeyCode::Char('z' | 'Z' | 'y' | 'Y') if ctrl => return false,
			_ => (),
		}
		if self.focus == 1 {
			match key.code {
				KeyCode::Up | KeyCode::BackTab => self.tool_index = (self.tool_index + 11) % 12,
				KeyCode::Down | KeyCode::Tab => self.tool_index = (self.tool_index + 1) % 12,
				KeyCode::Enter | KeyCode::Char(' ') => self.editor.apply_tool(self.tool_index),
				_ => (),
			}
			return true;
		}
		if self.focus == 2 {
			match key.code {
				KeyCode::Up => self.code_top = self.code_top.saturating_sub(1),
				KeyCode::Down => self.code_top = self.code_top.saturating_add(1),
				KeyCode::Left => self.code_left = self.code_left.saturating_sub(4),
				KeyCode::Right => self.code_left = self.code_left.saturating_add(4),
				KeyCode::PageUp => {
					self.code_top = self.code_top.saturating_sub(self.code_area.height as usize)
				}
				KeyCode::PageDown => {
					self.code_top = self.code_top.saturating_add(self.code_area.height as usize)
				}
				KeyCode::Home => {
					self.code_left = 0;
					if ctrl {
						self.code_top = 0;
					}
				}
				KeyCode::End => {
					self.code_left = usize::MAX;
					if ctrl {
						self.code_top = usize::MAX;
					}
				}
				_ => (),
			}
			return true;
		}
		false
	}

	fn mouse(&mut self, mouse: crossterm::event::MouseEvent) {
		let point = (mouse.column, mouse.row).into();
		if mouse.kind == MouseEventKind::Up(MouseButton::Left) {
			self.dragging = false;
			return;
		}
		if self.dragging
			&& mouse.kind == MouseEventKind::Drag(MouseButton::Left)
			&& self.editor_area.width > 0
			&& self.editor_area.height > 0
		{
			let area = self.editor_area;
			self.editor.click(
				mouse
					.column
					.clamp(area.x, area.right() - 1)
					.saturating_sub(area.x) as usize,
				mouse
					.row
					.clamp(area.y, area.bottom() - 1)
					.saturating_sub(area.y) as usize,
			);
			return;
		}
		if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
			self.dragging = false;
			if self.editor_wrap_toggle.contains(point) {
				self.toggle_wrap(false);
				return;
			}
			if self.code_wrap_toggle.contains(point) {
				self.toggle_wrap(true);
				return;
			}
			if self.code_toggle.contains(point) {
				self.code_open = !self.code_open;
				if !self.code_open && self.focus == 2 {
					self.focus = 0;
				}
				return;
			}
			if let Some((_, tool)) = self
				.tool_areas
				.iter()
				.find(|(area, _)| area.contains(point))
			{
				self.tool_index = *tool;
				self.editor.apply_tool(*tool);
				return;
			}
			if self.code_pane.contains(point) {
				self.focus = 2;
				self.editor.break_group();
				return;
			}
			if self.editor_pane.contains(point) {
				self.focus = 0;
				self.editor.click(
					mouse.column.saturating_sub(self.editor_area.x) as usize,
					(mouse.row - self.editor_area.y) as usize,
				);
				self.editor.anchor = Some(self.editor.cursor);
				self.dragging = true;
			}
		}
		if matches!(
			mouse.kind,
			MouseEventKind::ScrollDown | MouseEventKind::ScrollUp
		) {
			let down = mouse.kind == MouseEventKind::ScrollDown;
			if self.editor_pane.contains(point) {
				self.editor.scroll(down);
			} else if self.code_pane.contains(point) {
				self.code_top = if down {
					self.code_top.saturating_add(3)
				} else {
					self.code_top.saturating_sub(3)
				};
			}
		}
	}

	/// Returns true only when the user has chosen to exit.
	pub fn handle(&mut self, event: Event) -> bool {
		match event {
			Event::Key(key) if key.kind != KeyEventKind::Release => {
				self.notice = None;
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
				if self.workspace_key(key) {
					return false;
				}
				let selecting = key.modifiers.contains(KeyModifiers::SHIFT);
				let now = Instant::now();
				let word = key
					.modifiers
					.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
				if word {
					match key.code {
						KeyCode::Left => {
							self.editor.navigate(Movement::WordLeft, selecting);
							return false;
						}
						KeyCode::Right => {
							self.editor.navigate(Movement::WordRight, selecting);
							return false;
						}
						KeyCode::Backspace => {
							self.editor.delete_word(now);
							return false;
						}
						_ => (),
					}
				}
				match (key.code, control) {
					(KeyCode::Char('z' | 'Z'), true) => {
						self.editor
							.undo(key.modifiers.contains(KeyModifiers::SHIFT));
					}
					(KeyCode::Char('y' | 'Y'), true) => self.editor.undo(true),
					(KeyCode::Home, true) => self.editor.navigate(Movement::Start, selecting),
					(KeyCode::End, true) => self.editor.navigate(Movement::Finish, selecting),
					(KeyCode::Left, false) => self.editor.navigate(Movement::Left, selecting),
					(KeyCode::Right, false) => self.editor.navigate(Movement::Right, selecting),
					(KeyCode::Up, false) => self.editor.navigate(Movement::Up, selecting),
					(KeyCode::Down, false) => self.editor.navigate(Movement::Down, selecting),
					(KeyCode::Home, false) => self.editor.navigate(Movement::Home, selecting),
					(KeyCode::End, false) => self.editor.navigate(Movement::End, selecting),
					(KeyCode::PageUp, false) => self.editor.navigate(Movement::PageUp, selecting),
					(KeyCode::PageDown, false) => {
						self.editor.navigate(Movement::PageDown, selecting)
					}
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
			Event::Paste(text) if self.overlay == Overlay::None && self.focus == 0 => self
				.editor
				.insert(&sanitize(&text), EditKind::Atomic, Instant::now()),
			Event::Mouse(mouse) if self.overlay == Overlay::None => self.mouse(mouse),
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
	fn control_and_alt_word_bindings_and_focus_specific_wrap() {
		for modifier in [KeyModifiers::CONTROL, KeyModifiers::ALT] {
			let mut app = App::default();
			app.handle(Event::Paste("one two".into()));
			app.handle(key(KeyCode::Left, modifier));
			assert_eq!(app.editor.cursor.byte, 4);
			app.handle(key(KeyCode::Right, modifier | KeyModifiers::SHIFT));
			assert!(app.editor.selection().is_some());
			app.handle(key(KeyCode::Backspace, modifier));
			assert_eq!(app.editor.document.lines, ["one "]);
			app.handle(key(KeyCode::F(3), KeyModifiers::NONE));
			assert!(app.editor.wrap);
			assert!(!app.code_wrap);
			app.code_open = true;
			app.focus = 2;
			app.handle(key(KeyCode::F(3), KeyModifiers::NONE));
			assert!(app.editor.wrap && app.code_wrap);
		}
	}

	#[test]
	fn keyboard_selection_applies_styles_to_text() {
		let mut app = App::default();
		app.handle(Event::Paste("Hello\nworld".into()));
		app.handle(key(KeyCode::Home, KeyModifiers::CONTROL));
		for _ in 0..5 {
			app.handle(key(KeyCode::Right, KeyModifiers::SHIFT));
		}
		assert!(app.editor.selection().is_some());
		app.handle(key(KeyCode::Char('b'), KeyModifiers::CONTROL));
		assert!(app.editor.document.styles.at(0).bold);
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
