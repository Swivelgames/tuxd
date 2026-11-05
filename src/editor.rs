use std::time::{Duration, Instant};

use crate::document::{Document, Position, byte_at_column, column_at};

const HISTORY_LIMIT: usize = 200;
const GROUP_PAUSE: Duration = Duration::from_millis(750);

#[derive(Clone)]
struct Snapshot {
	document: Document,
	cursor: Position,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EditKind {
	Typing,
	Backspace,
	Delete,
	Atomic,
}

pub enum Movement {
	Left,
	Right,
	Up,
	Down,
	Home,
	End,
	Start,
	Finish,
	PageUp,
	PageDown,
}

#[derive(Default)]
pub struct Editor {
	pub document: Document,
	pub cursor: Position,
	pub top: usize,
	pub left: usize,
	pub width: usize,
	pub height: usize,
	preferred_column: Option<usize>,
	undo: Vec<Snapshot>,
	redo: Vec<Snapshot>,
	group: Option<(EditKind, Instant)>,
}

impl Editor {
	pub fn break_group(&mut self) {
		self.group = None;
	}

	fn snapshot(&self) -> Snapshot {
		Snapshot {
			document: self.document.clone(),
			cursor: self.cursor,
		}
	}

	fn begin_edit(&mut self, kind: EditKind, now: Instant) {
		let grouped = kind != EditKind::Atomic
			&& self.group.is_some_and(|(previous, at)| {
				previous == kind && now.saturating_duration_since(at) < GROUP_PAUSE
			});
		if !grouped {
			if self.undo.len() == HISTORY_LIMIT {
				self.undo.remove(0);
			}
			self.undo.push(self.snapshot());
		}
		self.redo.clear();
		self.group = if kind == EditKind::Atomic {
			None
		} else {
			Some((kind, now))
		};
		self.preferred_column = None;
	}

	pub fn insert(&mut self, text: &str, kind: EditKind, now: Instant) {
		if text.is_empty() {
			return;
		}
		self.begin_edit(kind, now);
		self.cursor = self.document.insert(self.cursor, text);
		self.ensure_visible();
	}

	pub fn delete(&mut self, backwards: bool, now: Instant) {
		let (start, end) = if backwards {
			(self.document.previous(self.cursor), self.cursor)
		} else {
			(self.cursor, self.document.next(self.cursor))
		};
		if start == end {
			self.break_group();
			return;
		}
		self.begin_edit(
			if backwards {
				EditKind::Backspace
			} else {
				EditKind::Delete
			},
			now,
		);
		self.cursor = self.document.remove(start, end);
		self.ensure_visible();
	}

	pub fn undo(&mut self, redo: bool) {
		self.break_group();
		let snapshot = if redo {
			self.redo.pop()
		} else {
			self.undo.pop()
		};
		if let Some(snapshot) = snapshot {
			let current = self.snapshot();
			if redo {
				self.undo.push(current);
			} else {
				self.redo.push(current);
			}
			self.document = snapshot.document;
			self.cursor = snapshot.cursor;
			self.preferred_column = None;
			self.ensure_visible();
		}
	}

	pub fn move_cursor(&mut self, movement: Movement) {
		self.break_group();
		let row = self.cursor.row;
		match movement {
			Movement::Up | Movement::Down | Movement::PageUp | Movement::PageDown => {
				let column = *self
					.preferred_column
					.get_or_insert_with(|| column_at(&self.document.lines[row], self.cursor.byte));
				let distance = if matches!(movement, Movement::PageUp | Movement::PageDown) {
					self.height.max(1)
				} else {
					1
				};
				let row = if matches!(movement, Movement::Up | Movement::PageUp) {
					row.saturating_sub(distance)
				} else {
					row.saturating_add(distance)
						.min(self.document.lines.len() - 1)
				};
				self.cursor = Position {
					row,
					byte: byte_at_column(&self.document.lines[row], column),
				};
			}
			_ => {
				self.preferred_column = None;
				self.cursor = match movement {
					Movement::Left => self.document.previous(self.cursor),
					Movement::Right => self.document.next(self.cursor),
					Movement::Home => Position { row, byte: 0 },
					Movement::End => Position {
						row,
						byte: self.document.lines[row].len(),
					},
					Movement::Start => Position::default(),
					Movement::Finish => {
						let row = self.document.lines.len() - 1;
						Position {
							row,
							byte: self.document.lines[row].len(),
						}
					}
					_ => unreachable!(),
				};
			}
		}
		self.ensure_visible();
	}

	pub fn resize(&mut self, width: usize, height: usize) {
		if (self.width, self.height) != (width, height) {
			self.width = width;
			self.height = height;
			self.ensure_visible();
		}
	}

	pub fn ensure_visible(&mut self) {
		let last_column = self
			.document
			.lines
			.iter()
			.map(|line| column_at(line, line.len()))
			.max()
			.unwrap_or(0);
		self.left = self
			.left
			.min((last_column + 1).saturating_sub(self.width.max(1)));
		self.top = self
			.top
			.min(self.document.lines.len().saturating_sub(self.height.max(1)));
		if self.cursor.row < self.top {
			self.top = self.cursor.row;
		}
		if self.cursor.row >= self.top + self.height.max(1) {
			self.top = self.cursor.row + 1 - self.height.max(1);
		}
		let column = column_at(&self.document.lines[self.cursor.row], self.cursor.byte);
		if column < self.left {
			self.left = column;
		}
		if column >= self.left + self.width.max(1) {
			self.left = column + 1 - self.width.max(1);
		}
	}

	pub fn click(&mut self, column: usize, row: usize) {
		self.break_group();
		self.preferred_column = None;
		let row = (self.top + row).min(self.document.lines.len() - 1);
		self.cursor = Position {
			row,
			byte: byte_at_column(&self.document.lines[row], self.left + column),
		};
		self.ensure_visible();
	}

	pub fn scroll(&mut self, down: bool) {
		self.break_group();
		let max = self.document.lines.len().saturating_sub(self.height.max(1));
		self.top = if down {
			self.top.saturating_add(3).min(max)
		} else {
			self.top.saturating_sub(3)
		};
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use unicode_segmentation::UnicodeSegmentation;

	#[test]
	fn typing_pause_navigation_and_paste_define_undo_groups() {
		let mut editor = Editor::default();
		let now = Instant::now();
		editor.insert("a", EditKind::Typing, now);
		editor.insert("b", EditKind::Typing, now + Duration::from_millis(100));
		editor.insert("c", EditKind::Typing, now + Duration::from_secs(1));
		editor.undo(false);
		assert_eq!(editor.document.lines, ["ab"]);
		editor.undo(false);
		assert!(editor.document.is_empty());
		editor.undo(true);
		assert_eq!(editor.document.lines, ["ab"]);
		editor.move_cursor(Movement::Left);
		editor.insert("X\nY", EditKind::Atomic, now);
		assert_eq!(editor.document.lines, ["aX", "Yb"]);
		editor.undo(false);
		assert_eq!(editor.document.lines, ["ab"]);
		assert_eq!(editor.cursor.byte, 1);
		editor.insert("z", EditKind::Typing, now);
		editor.undo(true);
		assert_eq!(editor.document.lines, ["azb"]);
	}

	#[test]
	fn deletes_join_lines_and_restore_whole_graphemes() {
		let mut editor = Editor::default();
		let now = Instant::now();
		editor.insert("e\u{301}\n界👩‍💻", EditKind::Atomic, now);
		editor.delete(true, now);
		assert_eq!(editor.document.lines, ["e\u{301}", "界"]);
		editor.undo(false);
		assert_eq!(editor.document.lines, ["e\u{301}", "界👩‍💻"]);
		editor.move_cursor(Movement::Home);
		editor.delete(true, now);
		assert_eq!(editor.document.lines, ["e\u{301}界👩‍💻"]);
		editor.move_cursor(Movement::Home);
		editor.delete(false, now);
		assert_eq!(editor.document.lines, ["界👩‍💻"]);
	}

	#[test]
	fn vertical_navigation_remembers_column_across_short_lines() {
		let mut editor = Editor::default();
		editor.resize(10, 3);
		editor.insert("abcdef\nx\nabcdef", EditKind::Atomic, Instant::now());
		editor.move_cursor(Movement::Up);
		assert_eq!(editor.cursor, Position { row: 1, byte: 1 });
		editor.move_cursor(Movement::Up);
		assert_eq!(editor.cursor, Position { row: 0, byte: 6 });
	}

	#[test]
	fn resizing_scroll_and_clicks_keep_valid_coordinates() {
		let mut editor = Editor::default();
		editor.resize(4, 2);
		editor.insert("0123456789\nx\ny\nz", EditKind::Atomic, Instant::now());
		assert_eq!(editor.top, 2);
		editor.scroll(false);
		assert_eq!(editor.top, 0);
		editor.click(3, 0);
		assert_eq!(editor.cursor.byte, 3);
		editor.move_cursor(Movement::End);
		assert_eq!(editor.left, 7);
		editor.resize(100, 100);
		assert_eq!(editor.top, 0);
		assert_eq!(editor.left, 0);
		// Expand the viewport without moving the document position.
		assert_eq!(editor.cursor.byte, 10);
		editor.resize(0, 0);
		editor.move_cursor(Movement::Finish);
		editor.resize(4, 2);
		assert_eq!(editor.top, 2);
		editor.move_cursor(Movement::Start);
		editor.delete(false, Instant::now());
		assert_eq!(editor.left, 0);
	}

	#[test]
	fn deterministic_edit_sequences_preserve_invariants_and_roundtrip_history() {
		let mut editor = Editor::default();
		editor.resize(8, 3);
		let mut state = 12345u32;
		for _ in 0..180 {
			state = state.wrapping_mul(1664525).wrapping_add(1013904223);
			match state % 8 {
				0..=2 => editor.insert(
					["e\u{301}", "界", "\n", "\t", "👩‍💻"][(state as usize / 8) % 5],
					EditKind::Atomic,
					Instant::now(),
				),
				3 => {
					editor.break_group();
					editor.delete(true, Instant::now());
				}
				4 => {
					editor.break_group();
					editor.delete(false, Instant::now());
				}
				5 => editor.move_cursor(Movement::Left),
				6 => editor.move_cursor(Movement::Up),
				_ => editor.move_cursor(Movement::Right),
			}
			let line = &editor.document.lines[editor.cursor.row];
			assert!(
				line.grapheme_indices(true)
					.map(|(byte, _)| byte)
					.chain([line.len()])
					.any(|byte| byte == editor.cursor.byte)
			);
		}
		let final_document = editor.document.clone();
		while !editor.undo.is_empty() {
			editor.undo(false);
		}
		assert!(editor.document.is_empty());
		while !editor.redo.is_empty() {
			editor.undo(true);
		}
		assert_eq!(editor.document, final_document);
	}
}
