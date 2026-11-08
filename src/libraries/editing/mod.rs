use crate::libraries::ansi::Action;
use crate::libraries::text::Layout;
use std::time::{Duration, Instant};

use crate::libraries::ansi::{Document, Position};
use crate::libraries::text::display::{byte_at_column, column_at};

/// Pure visual information supplied by the owning panel.
pub struct Navigation<'a> {
	pub layout: &'a Layout,
	pub wrap: bool,
	pub height: usize,
}

const HISTORY_LIMIT: usize = 200;
const GROUP_PAUSE: Duration = Duration::from_millis(750);

#[derive(Clone)]
struct Snapshot {
	anchor: Option<Position>,
	pending: Option<crate::libraries::ansi::TextStyle>,
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
pub struct Editing {
	pub revision: u64,
	pub anchor: Option<Position>,
	pub pending: Option<crate::libraries::ansi::TextStyle>,
	pub document: Document,
	pub cursor: Position,
	preferred_column: Option<usize>,
	undo: Vec<Snapshot>,
	redo: Vec<Snapshot>,
	group: Option<(EditKind, Instant)>,
}

impl Editing {
	pub fn can_undo(&self) -> bool {
		!self.undo.is_empty()
	}

	pub fn can_redo(&self) -> bool {
		!self.redo.is_empty()
	}

	pub fn new(document: Document) -> Self {
		Self {
			document,
			..Self::default()
		}
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
		self.cursor = self
			.document
			.remove(start, end)
			.expect("valid deletion range");
		self.anchor = None;
		self.pending = None;
	}

	pub fn selection(&self) -> Option<(Position, Position)> {
		self.anchor
			.filter(|anchor| *anchor != self.cursor)
			.map(|anchor| (anchor.min(self.cursor), anchor.max(self.cursor)))
	}

	pub fn typing_style(&self) -> crate::libraries::ansi::TextStyle {
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

	pub fn tool_state(&self, tool: Action) -> Option<bool> {
		if let Some((start, end)) = self.selection() {
			self.document
				.style_state(start..end, tool)
				.expect("valid selection")
		} else {
			Some(self.typing_style().matches(tool))
		}
	}

	pub fn apply_tool(&mut self, tool: Action) {
		let enabled = self.tool_state(tool) != Some(true);
		self.begin_edit(EditKind::Atomic, Instant::now());
		if let Some((start, end)) = self.selection() {
			self.document
				.apply_style(start..end, tool, enabled)
				.expect("valid tool and selection");
		} else {
			let mut style = self.typing_style();
			style.set(tool, enabled);
			self.pending = Some(style);
		}
	}

	pub fn navigate(&mut self, movement: Movement, selecting: bool, view: Navigation<'_>) {
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
				return;
			}
			self.anchor = None;
		}
		self.move_cursor(movement, view);
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
		self.revision = self.revision.wrapping_add(1);
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
		let fragment = Document::from_text(text, style).expect("normalized editor input");
		let (start, end) = selection.unwrap_or((self.cursor, self.cursor));

		self.begin_edit(
			if selection.is_some() {
				EditKind::Atomic
			} else {
				kind
			},
			now,
		);
		self.cursor = self
			.document
			.replace(start..end, &fragment)
			.expect("valid selection");
		self.anchor = None;
		self.pending = Some(style);
	}

	pub fn selected_document(&self) -> Option<Document> {
		let (start, end) = self.selection()?;
		Some(self.document.extract(start..end).expect("valid selection"))
	}

	pub fn paste_document(&mut self, document: &Document, formatted: bool) {
		if document.is_empty() {
			return;
		}
		let typing = self.typing_style();
		let plain;
		let fragment = if formatted {
			document
		} else {
			plain = Document::from_text(&document.text(), typing).expect("valid paste content");
			&plain
		};
		let (start, end) = self.selection().unwrap_or((self.cursor, self.cursor));

		self.begin_edit(EditKind::Atomic, Instant::now());
		self.cursor = self
			.document
			.replace(start..end, fragment)
			.expect("valid paste target");
		self.anchor = None;
		self.pending = Some(typing);
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
		self.cursor = self
			.document
			.remove(start, end)
			.expect("valid deletion range");
		self.anchor = None;
		self.pending = None;
	}

	pub fn undo(&mut self, redo: bool) {
		self.break_group();
		let snapshot = if redo {
			self.redo.pop()
		} else {
			self.undo.pop()
		};
		if let Some(snapshot) = snapshot {
			self.revision = self.revision.wrapping_add(1);
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
		}
	}

	pub fn move_cursor(&mut self, movement: Movement, view: Navigation<'_>) {
		self.pending = None;
		self.break_group();
		let row = self.cursor.row;
		match movement {
			Movement::Up | Movement::Down | Movement::PageUp | Movement::PageDown => {
				if view.wrap {
					let layout = view.layout;
					let (visual_row, x) = layout.location(self.cursor);
					let column = *self.preferred_column.get_or_insert(x);
					let distance = if matches!(movement, Movement::PageUp | Movement::PageDown) {
						view.height.max(1)
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
					return;
				}
				let column = *self.preferred_column.get_or_insert_with(|| {
					column_at(&self.document.lines()[row], self.cursor.byte)
				});
				let distance = if matches!(movement, Movement::PageUp | Movement::PageDown) {
					view.height.max(1)
				} else {
					1
				};
				let row = if matches!(movement, Movement::Up | Movement::PageUp) {
					row.saturating_sub(distance)
				} else {
					row.saturating_add(distance)
						.min(self.document.lines().len() - 1)
				};
				self.cursor = Position {
					row,
					byte: byte_at_column(&self.document.lines()[row], column),
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
						byte: self.document.lines()[row].len(),
					},
					Movement::Start => Position::default(),
					Movement::Finish => {
						let row = self.document.lines().len() - 1;
						Position {
							row,
							byte: self.document.lines()[row].len(),
						}
					}
					_ => unreachable!(),
				};
			}
		}
	}

	/// A display adapter supplies the hit position; editing owns caret semantics.
	pub fn place_cursor(&mut self, position: Position) {
		self.pending = None;
		self.break_group();
		self.preferred_column = None;
		self.cursor = position;
	}

	pub fn reset_movement(&mut self) {
		self.preferred_column = None;
		self.break_group();
	}
}
