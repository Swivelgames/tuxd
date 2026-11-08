//! Test fixture composing real Editing and Editor owners.
use crate::features::editor::Editor;
use crate::libraries::editing::{EditKind, Editing, Movement};
use std::time::Instant;
#[derive(Default)]
pub struct Harness {
	pub editing: Editing,
	pub panel: Editor,
}
impl std::ops::Deref for Harness {
	type Target = Editing;
	fn deref(&self) -> &Editing {
		&self.editing
	}
}
impl std::ops::DerefMut for Harness {
	fn deref_mut(&mut self) -> &mut Editing {
		&mut self.editing
	}
}
impl Harness {
	pub fn insert(&mut self, text: &str, kind: EditKind, now: Instant) {
		self.panel.insert(&mut self.editing, text, kind, now);
	}
	pub fn delete(&mut self, backwards: bool, now: Instant) {
		self.panel.delete(&mut self.editing, backwards, now);
	}
	pub fn delete_word(&mut self, now: Instant) {
		self.panel.delete_word(&mut self.editing, now);
	}
	pub fn undo(&mut self, redo: bool) {
		self.panel.undo(&mut self.editing, redo);
	}
	pub fn navigate(&mut self, movement: Movement, selecting: bool) {
		self.panel.navigate(&mut self.editing, movement, selecting);
	}
	pub fn move_cursor(&mut self, movement: Movement) {
		self.panel.move_cursor(&mut self.editing, movement);
	}
	pub fn click(&mut self, column: usize, row: usize) {
		self.panel.click(&mut self.editing, column, row);
	}
	pub fn scroll(&mut self, down: bool) {
		self.panel.scroll(&mut self.editing, down);
	}
	pub fn resize(&mut self, width: usize, height: usize) {
		self.panel.resize(&self.editing, width, height);
	}
	pub fn toggle_wrap(&mut self) {
		self.panel.toggle_wrap(&mut self.editing);
	}
	pub fn layout(&self) -> crate::libraries::text::Layout {
		self.panel.layout(&self.editing)
	}
}
