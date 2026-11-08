use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy)]
pub enum FieldAction {
	Left,
	Right,
	Beginning,
	End,
	Backspace,
	Delete,
	SelectAll,
}

/// Single-line editing state. Filtering and size limits are supplied by consumers.
#[derive(Clone, Debug, Default)]
pub struct Field {
	text: String,
	cursor: usize,
	selected: bool,
}

impl Field {
	pub fn new(text: String) -> Self {
		let mut field = Self::default();
		field.set(text);
		field
	}

	pub fn selecting(text: String) -> Self {
		let mut field = Self::new(text);
		field.select_all();
		field
	}

	pub fn text(&self) -> &str {
		&self.text
	}

	pub fn cursor(&self) -> usize {
		self.cursor
	}

	pub fn selected(&self) -> bool {
		self.selected
	}

	pub fn select_all(&mut self) {
		self.selected = true;
	}

	pub fn set(&mut self, text: String) {
		self.cursor = text.len();
		self.text = text;
		self.selected = false;
	}

	pub fn insert(&mut self, text: &str, limit: usize) {
		if self.selected {
			self.set(String::new());
		}
		if self.text.len().saturating_add(text.len()) > limit {
			return;
		}
		self.text.insert_str(self.cursor, text);
		self.cursor += text.len();
		self.align_cursor();
	}

	fn align_cursor(&mut self) {
		// Edits can join graphemes on either side. Keep the caret on a boundary.
		self.cursor = self
			.text
			.grapheme_indices(true)
			.map(|(i, _)| i)
			.find(|i| *i >= self.cursor)
			.unwrap_or(self.text.len());
	}

	pub fn apply(&mut self, action: FieldAction) {
		use FieldAction::*;
		if matches!(action, SelectAll) {
			self.select_all();
			return;
		}
		if matches!(action, Backspace | Delete) && self.selected {
			self.set(String::new());
			return;
		}
		let previous = self.text[..self.cursor]
			.grapheme_indices(true)
			.next_back()
			.map_or(0, |(i, _)| i);
		let next = self.cursor
			+ self.text[self.cursor..]
				.graphemes(true)
				.next()
				.map_or(0, str::len);
		match action {
			Beginning => self.cursor = 0,
			End => self.cursor = self.text.len(),
			Left => self.cursor = previous,
			Right => self.cursor = next,
			Backspace => {
				self.text.replace_range(previous..self.cursor, "");
				self.cursor = previous;
			}
			Delete => {
				self.text.replace_range(self.cursor..next, "");
			}
			SelectAll => unreachable!(),
		}
		self.align_cursor();
		self.selected = false;
	}
}
