use std::time::{Duration, Instant};

use crate::document::{Document, Position, byte_at_column, column_at};

const HISTORY_LIMIT: usize = 200;
const GROUP_PAUSE: Duration = Duration::from_millis(750);

#[derive(Clone)]
struct Snapshot {
	anchor: Option<Position>,
	pending: Option<crate::styling::TextStyle>,
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
	WordLeft,
	WordRight,
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
	pub wrap: bool,
	pub anchor: Option<Position>,
	pub pending: Option<crate::styling::TextStyle>,
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
	pub fn layout(&self) -> crate::layout::Layout {
		crate::layout::Layout::new(&self.document.lines, self.width, self.wrap, true)
	}

	pub fn toggle_wrap(&mut self) {
		self.wrap = !self.wrap;
		self.left = 0;
		self.top = 0;
		self.preferred_column = None;
		self.break_group();
		self.ensure_visible();
	}

	pub fn delete_word(&mut self, now: Instant) {
		let (start, end) = self
			.selection()
			.unwrap_or_else(|| (self.document.word_target(self.cursor, true), self.cursor));
		if start == end {
			self.break_group();
			return;
		}
		self.begin_edit(EditKind::Atomic, now);
		self.cursor = self.document.remove(start, end);
		self.anchor = None;
		self.pending = None;
		self.ensure_visible();
	}

	pub fn selection(&self) -> Option<(Position, Position)> {
		self.anchor
			.filter(|anchor| *anchor != self.cursor)
			.map(|anchor| (anchor.min(self.cursor), anchor.max(self.cursor)))
	}

	pub fn typing_style(&self) -> crate::styling::TextStyle {
		self.pending.unwrap_or_else(|| {
			if self.cursor == Position::default() {
				Default::default()
			} else {
				self.document.style_at(self.document.previous(self.cursor))
			}
		})
	}

	pub fn selected(&self, position: Position) -> bool {
		self.selection()
			.is_some_and(|(start, end)| start <= position && position < end)
	}

	pub fn tool_state(&self, tool: usize) -> Option<bool> {
		let matches = |style: crate::styling::TextStyle| style.matches(tool);
		if let Some((start, end)) = self.selection() {
			let a = self.document.offset(start);
			let b = self.document.offset(end);
			let first = matches(self.document.styles.at(a));
			for run in &self.document.styles.runs {
				if run.end > a && run.start < b && matches(run.style) != first {
					return None;
				}
			}
			Some(first)
		} else {
			Some(matches(self.typing_style()))
		}
	}

	pub fn apply_tool(&mut self, tool: usize) {
		let enabled = self.tool_state(tool) != Some(true);
		self.begin_edit(EditKind::Atomic, Instant::now());
		if let Some((start, end)) = self.selection() {
			self.document.styles.apply(
				self.document.offset(start),
				self.document.offset(end),
				|mut style| {
					style.set(tool, enabled);
					style
				},
			);
		} else {
			let mut style = self.typing_style();
			style.set(tool, enabled);
			self.pending = Some(style);
		}
	}

	pub fn navigate(&mut self, movement: Movement, selecting: bool) {
		if selecting {
			if self.anchor.is_none() {
				self.anchor = Some(self.cursor);
			}
		} else {
			if let Some((start, end)) = self.selection()
				&& matches!(
					movement,
					Movement::Left | Movement::Right | Movement::WordLeft | Movement::WordRight
				) {
				self.cursor = if matches!(movement, Movement::Left | Movement::WordLeft) {
					start
				} else {
					end
				};
				self.anchor = None;
				self.pending = None;
				self.break_group();
				self.ensure_visible();
				return;
			}
			self.anchor = None;
		}
		self.move_cursor(movement);
	}

	pub fn break_group(&mut self) {
		self.group = None;
	}

	fn snapshot(&self) -> Snapshot {
		Snapshot {
			anchor: self.anchor,
			pending: self.pending,
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
		let selection = self.selection();
		let style = selection.map_or_else(
			|| self.typing_style(),
			|(start, _)| self.document.style_at(start),
		);
		self.begin_edit(
			if selection.is_some() {
				EditKind::Atomic
			} else {
				kind
			},
			now,
		);
		if let Some((start, end)) = selection {
			self.document.remove(start, end);
			self.cursor = start;
		}
		let offset = self.document.offset(self.cursor);
		self.cursor = self.document.insert(self.cursor, text);
		self.document
			.styles
			.apply(offset, offset + text.len(), |_| style);
		self.document.normalize_styles();
		self.anchor = None;
		self.pending = Some(style);
		self.ensure_visible();
	}

	pub fn delete(&mut self, backwards: bool, now: Instant) {
		let (start, end) = if let Some(range) = self.selection() {
			range
		} else if backwards {
			(self.document.previous(self.cursor), self.cursor)
		} else {
			(self.cursor, self.document.next(self.cursor))
		};
		if start == end {
			self.break_group();
			return;
		}
		self.begin_edit(
			if self.selection().is_some() {
				EditKind::Atomic
			} else if backwards {
				EditKind::Backspace
			} else {
				EditKind::Delete
			},
			now,
		);
		self.cursor = self.document.remove(start, end);
		self.anchor = None;
		self.pending = None;
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
			self.anchor = snapshot.anchor;
			self.pending = snapshot.pending;
			self.document = snapshot.document;
			self.cursor = snapshot.cursor;
			self.preferred_column = None;
			self.ensure_visible();
		}
	}

	pub fn move_cursor(&mut self, movement: Movement) {
		self.pending = None;
		self.break_group();
		let row = self.cursor.row;
		match movement {
			Movement::Up | Movement::Down | Movement::PageUp | Movement::PageDown => {
				if self.wrap {
					let layout = self.layout();
					let (visual_row, x) = layout.location(self.cursor);
					let column = *self.preferred_column.get_or_insert(x);
					let distance = if matches!(movement, Movement::PageUp | Movement::PageDown) {
						self.height.max(1)
					} else {
						1
					};
					let target = if matches!(movement, Movement::Up | Movement::PageUp) {
						visual_row.saturating_sub(distance)
					} else {
						visual_row
							.saturating_add(distance)
							.min(layout.rows.len() - 1)
					};
					// A continuation row's end belongs to the next row; clamp to its last cell.
					let row = &layout.rows[target];
					let continuation = layout
						.rows
						.get(target + 1)
						.is_some_and(|next| next.line == row.line);
					self.cursor = layout.hit(
						target,
						if continuation {
							column.min(row.width.saturating_sub(1))
						} else {
							column
						},
					);
					self.ensure_visible();
					return;
				}
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
					Movement::WordLeft => self.document.word_target(self.cursor, true),
					Movement::WordRight => self.document.word_target(self.cursor, false),
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
		if self.wrap {
			let layout = self.layout();
			let (row, _) = layout.location(self.cursor);
			self.left = 0;
			self.top = self
				.top
				.min(layout.rows.len().saturating_sub(self.height.max(1)));
			if row < self.top {
				self.top = row;
			}
			if row >= self.top + self.height.max(1) {
				self.top = row + 1 - self.height.max(1);
			}
			return;
		}
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
		self.pending = None;
		self.break_group();
		self.preferred_column = None;
		if self.wrap {
			self.cursor = self.layout().hit(self.top + row, column);
			self.ensure_visible();
			return;
		}
		let row = (self.top + row).min(self.document.lines.len() - 1);
		self.cursor = Position {
			row,
			byte: byte_at_column(&self.document.lines[row], self.left + column),
		};
		self.ensure_visible();
	}

	pub fn scroll(&mut self, down: bool) {
		self.break_group();
		let count = if self.wrap {
			self.layout().rows.len()
		} else {
			self.document.lines.len()
		};
		let max = count.saturating_sub(self.height.max(1));
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
	fn wrapping_preserves_document_and_moves_by_screen_rows() {
		let mut editor = Editor::default();
		editor.resize(4, 3);
		editor.insert("abcdefghij\nXYZ", EditKind::Atomic, Instant::now());
		let original = editor.document.clone();
		editor.navigate(Movement::Start, false);
		editor.navigate(Movement::Right, false);
		editor.navigate(Movement::Right, false);
		editor.toggle_wrap();
		editor.navigate(Movement::Down, false);
		assert_eq!(editor.cursor, Position { row: 0, byte: 6 });
		editor.navigate(Movement::Down, true);
		assert_eq!(editor.cursor, Position { row: 0, byte: 10 });
		editor.navigate(Movement::Down, true);
		assert_eq!(editor.cursor, Position { row: 1, byte: 2 });
		editor.navigate(Movement::Start, false);
		editor.click(1, 1);
		assert_eq!(editor.cursor, Position { row: 0, byte: 5 });
		let cursor = editor.cursor;
		editor.resize(2, 2);
		let (row, _) = editor.layout().location(cursor);
		assert!((editor.top..editor.top + editor.height).contains(&row));
		editor.toggle_wrap();
		assert_eq!(editor.cursor, cursor);
		assert_eq!(editor.document, original);
	}

	#[test]
	fn word_deletion_preserves_styles_and_is_one_undo_transaction() {
		let mut editor = Editor::default();
		editor.apply_tool(0);
		editor.insert("one ", EditKind::Atomic, Instant::now());
		editor.apply_tool(1);
		editor.insert("café", EditKind::Atomic, Instant::now());
		let before = editor.document.clone();
		editor.delete_word(Instant::now());
		assert_eq!(editor.document.lines, ["one "]);
		assert!(editor.document.styles.at(0).bold);
		editor.undo(false);
		assert_eq!(editor.document, before);
		editor.navigate(Movement::WordLeft, true);
		assert_eq!(
			editor.selection(),
			Some((Position { row: 0, byte: 4 }, Position { row: 0, byte: 9 }))
		);
		editor.delete_word(Instant::now());
		assert_eq!(editor.document.lines, ["one "]);
	}

	#[test]
	fn selection_format_replace_and_history_preserve_styles() {
		let mut editor = Editor::default();
		editor.insert("abc\ndef", EditKind::Atomic, Instant::now());
		editor.navigate(Movement::Start, false);
		editor.navigate(Movement::Right, true);
		editor.navigate(Movement::Right, true);
		editor.apply_tool(0);
		editor.apply_tool(1);
		editor.apply_tool(10);
		assert_eq!(editor.tool_state(0), Some(true));
		editor.navigate(Movement::Right, true);
		assert_eq!(editor.tool_state(0), None);
		editor.apply_tool(0);
		assert_eq!(editor.tool_state(0), Some(true));
		editor.insert("界\nX", EditKind::Typing, Instant::now());
		assert_eq!(editor.document.lines, ["界", "X", "def"]);
		assert!(editor.document.styles.at(0).bold);
		assert!(editor.document.styles.at(0).italic);
		assert_eq!(editor.document.styles.at(0).foreground, Some(6));
		editor.undo(false);
		assert_eq!(editor.document.lines, ["abc", "def"]);
		assert_eq!(
			editor.selection(),
			Some((Position::default(), Position { row: 0, byte: 3 }))
		);
		editor.undo(true);
		assert_eq!(editor.document.lines, ["界", "X", "def"]);
	}

	#[test]
	fn style_inheritance_and_reverse_selection_collapse() {
		let mut editor = Editor::default();
		editor.insert("ab", EditKind::Atomic, Instant::now());
		editor.navigate(Movement::Home, false);
		editor.navigate(Movement::Right, true);
		editor.apply_tool(0);
		editor.navigate(Movement::Right, false);
		assert_eq!(editor.cursor.byte, 1);
		assert!(editor.typing_style().bold);
		editor.insert("X", EditKind::Typing, Instant::now());
		assert!(editor.document.styles.at(1).bold);
		assert!(!editor.document.styles.at(2).bold);
		editor.navigate(Movement::Home, false);
		assert!(!editor.typing_style().bold);
		editor.apply_tool(1);
		editor.insert("Y", EditKind::Typing, Instant::now());
		assert!(editor.document.styles.at(0).italic);
		editor.navigate(Movement::End, false);
		editor.navigate(Movement::Left, true);
		editor.navigate(Movement::Left, true);
		editor.navigate(Movement::Left, false);
		assert_eq!(editor.cursor.byte, 2);
		assert!(editor.selection().is_none());
	}

	#[test]
	fn combining_edits_do_not_split_style_boundaries() {
		let mut editor = Editor::default();
		editor.insert("e", EditKind::Atomic, Instant::now());
		editor.apply_tool(0);
		editor.insert("\u{301}", EditKind::Typing, Instant::now());
		assert_eq!(editor.document.styles.runs.len(), 1);
		assert_eq!(editor.document.styles.runs[0].end, "e\u{301}".len());
		editor.delete(true, Instant::now());
		assert!(editor.document.styles.runs.is_empty());
		editor.undo(false);
		assert_eq!(editor.document.lines, ["e\u{301}"]);
	}

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
