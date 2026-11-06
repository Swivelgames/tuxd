use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub const TAB_STOP: usize = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
	pub row: usize,
	pub byte: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
	pub lines: Vec<String>,
	pub styles: crate::styling::Styles,
}

impl Default for Document {
	fn default() -> Self {
		Self {
			lines: vec![String::new()],
			styles: Default::default(),
		}
	}
}

impl Document {
	pub fn word_target(&self, position: Position, backwards: bool) -> Position {
		let text = self.lines.join("\n");
		let offset = self.offset(position);
		// Unicode word-boundary segments; skip whitespace, treating punctuation as
		// separate tokens. Apostrophes within words follow Unicode segmentation.
		let segments: Vec<_> = text.split_word_bound_indices().collect();
		let target = if backwards {
			segments
				.iter()
				.rev()
				.find(|(start, part)| *start < offset && !part.chars().all(char::is_whitespace))
				.map_or(0, |(start, _)| *start)
		} else {
			segments
				.iter()
				.find(|(start, part)| {
					start + part.len() > offset && !part.chars().all(char::is_whitespace)
				})
				.map_or(text.len(), |(start, part)| start + part.len())
		};
		let mut remaining = target;
		for (row, line) in self.lines.iter().enumerate() {
			if remaining <= line.len() {
				return Position {
					row,
					byte: if backwards {
						boundary_before(line, remaining)
					} else {
						boundary_after(line, remaining)
					},
				};
			}
			remaining -= line.len() + 1;
		}
		position
	}

	pub fn offset(&self, position: Position) -> usize {
		self.lines[..position.row]
			.iter()
			.map(|line| line.len() + 1)
			.sum::<usize>()
			+ position.byte
	}

	pub fn style_at(&self, position: Position) -> crate::styling::TextStyle {
		self.styles.at(self.offset(position))
	}

	pub fn normalize_styles(&mut self) {
		let text = self.lines.join("\n");
		let mut normalized = crate::styling::Styles::default();
		let mut runs = self.styles.runs.iter().peekable();
		for (byte, grapheme) in text.grapheme_indices(true) {
			while runs.peek().is_some_and(|run| run.end <= byte) {
				runs.next();
			}
			let style = runs
				.peek()
				.filter(|run| run.start <= byte)
				.map_or(crate::styling::TextStyle::default(), |run| run.style);
			normalized.push(byte, byte + grapheme.len(), style);
		}
		self.styles = normalized;
	}

	#[cfg(test)]
	pub fn is_empty(&self) -> bool {
		self.lines.len() == 1 && self.lines[0].is_empty()
	}

	pub fn insert(&mut self, position: Position, text: &str) -> Position {
		self.styles.insert(
			self.offset(position),
			text.len(),
			crate::styling::TextStyle::default(),
		);
		let tail = self.lines[position.row].split_off(position.byte);
		let parts: Vec<_> = text.split('\n').collect();
		self.lines[position.row].push_str(parts[0]);
		let row = position.row + parts.len() - 1;
		let byte = if parts.len() == 1 {
			position.byte + parts[0].len()
		} else {
			parts.last().unwrap().len()
		};
		if parts.len() > 1 {
			self.lines.splice(
				position.row + 1..position.row + 1,
				parts[1..].iter().map(|part| (*part).to_owned()),
			);
		}
		self.lines[row].push_str(&tail);
		// Insertion can join a combining sequence or an emoji with following text.
		Position {
			row,
			byte: boundary_after(&self.lines[row], byte),
		}
	}

	pub fn remove(&mut self, start: Position, end: Position) -> Position {
		self.styles.remove(self.offset(start), self.offset(end));
		if start.row == end.row {
			self.lines[start.row].replace_range(start.byte..end.byte, "");
		} else {
			let tail = self.lines[end.row][end.byte..].to_owned();
			self.lines[start.row].truncate(start.byte);
			self.lines[start.row].push_str(&tail);
			self.lines.drain(start.row + 1..=end.row);
		}
		self.normalize_styles();
		Position {
			row: start.row,
			byte: boundary_before(&self.lines[start.row], start.byte),
		}
	}

	pub fn previous(&self, position: Position) -> Position {
		if position.byte > 0 {
			Position {
				byte: boundary_before(&self.lines[position.row], position.byte - 1),
				..position
			}
		} else if position.row > 0 {
			Position {
				row: position.row - 1,
				byte: self.lines[position.row - 1].len(),
			}
		} else {
			position
		}
	}

	pub fn next(&self, position: Position) -> Position {
		if position.byte < self.lines[position.row].len() {
			Position {
				byte: boundary_after(&self.lines[position.row], position.byte + 1),
				..position
			}
		} else if position.row + 1 < self.lines.len() {
			Position {
				row: position.row + 1,
				byte: 0,
			}
		} else {
			position
		}
	}
}

fn boundary_before(line: &str, byte: usize) -> usize {
	line.grapheme_indices(true)
		.map(|(index, _)| index)
		.chain([line.len()])
		.take_while(|index| *index <= byte)
		.last()
		.unwrap_or(0)
}

fn boundary_after(line: &str, byte: usize) -> usize {
	line.grapheme_indices(true)
		.map(|(index, _)| index)
		.chain([line.len()])
		.find(|index| *index >= byte)
		.unwrap_or(line.len())
}

pub struct Glyph<'a> {
	pub byte: usize,
	pub column: usize,
	pub width: usize,
	pub text: &'a str,
}

pub fn glyphs(line: &str) -> impl Iterator<Item = Glyph<'_>> {
	let mut column = 0;
	line.grapheme_indices(true).map(move |(byte, text)| {
		let width = if text == "\t" {
			TAB_STOP - column % TAB_STOP
		} else {
			text.width().max(1)
		};
		let glyph = Glyph {
			byte,
			column,
			width,
			text,
		};
		column += width;
		glyph
	})
}

pub fn column_at(line: &str, byte: usize) -> usize {
	glyphs(line)
		.take_while(|glyph| glyph.byte < byte)
		.map(|glyph| glyph.width)
		.sum()
}

pub fn byte_at_column(line: &str, column: usize) -> usize {
	// A click inside a tab or wide grapheme lands at its leading boundary.
	glyphs(line)
		.find(|glyph| column < glyph.column + glyph.width)
		.map_or(line.len(), |glyph| glyph.byte)
}

pub fn sanitize(text: &str) -> String {
	let mut result = String::new();
	let mut chars = text.chars().peekable();
	while let Some(ch) = chars.next() {
		match ch {
			'\r' => {
				if chars.peek() == Some(&'\n') {
					chars.next();
				}
				result.push('\n');
			}
			'\n' | '\t' => result.push(ch),
			ch if ch.is_control() => result.extend(ch.escape_default()),
			ch => result.push(ch),
		}
	}
	result
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn word_boundaries_skip_whitespace_cross_lines_and_keep_graphemes() {
		let mut doc = Document::default();
		doc.insert(Position::default(), "one  café!\n e\u{301}");
		let after_one = doc.word_target(Position::default(), false);
		assert_eq!(after_one, Position { row: 0, byte: 3 });
		let after_cafe = doc.word_target(after_one, false);
		assert_eq!(after_cafe, Position { row: 0, byte: 10 });
		let punctuation = doc.word_target(after_cafe, false);
		assert_eq!(punctuation, Position { row: 0, byte: 11 });
		assert_eq!(
			doc.word_target(punctuation, false),
			Position { row: 1, byte: 4 }
		);
		assert_eq!(
			doc.word_target(Position { row: 1, byte: 1 }, true),
			after_cafe
		);
		assert_eq!(
			doc.word_target(Position { row: 1, byte: 4 }, true),
			Position { row: 1, byte: 1 }
		);
	}

	#[test]
	fn multiline_insert_remove_and_trailing_newline() {
		let mut doc = Document::default();
		let end = doc.insert(Position::default(), "headTAIL");
		assert_eq!(end.byte, 8);
		let end = doc.insert(Position { row: 0, byte: 4 }, "a\nb\n");
		assert_eq!(doc.lines, ["heada", "b", "TAIL"]);
		assert_eq!(end, Position { row: 2, byte: 0 });
		doc.remove(Position { row: 0, byte: 4 }, end);
		assert_eq!(doc.lines, ["headTAIL"]);
	}

	#[test]
	fn navigation_uses_whole_graphemes_and_crosses_newlines() {
		let mut doc = Document::default();
		let end = doc.insert(Position::default(), "e\u{301}界👩‍💻\n");
		let newline = doc.previous(end);
		assert_eq!(
			newline,
			Position {
				row: 0,
				byte: "e\u{301}界👩‍💻".len()
			}
		);
		let emoji = doc.previous(newline);
		assert_eq!(emoji.byte, "e\u{301}界".len());
		assert_eq!(doc.next(emoji), newline);
		assert_eq!(doc.next(newline), end);
		let wide = doc.previous(emoji);
		assert_eq!(wide.byte, "e\u{301}".len());
		assert_eq!(doc.previous(wide), Position::default());
	}

	#[test]
	fn editing_can_merge_graphemes_without_leaving_invalid_cursor() {
		let mut doc = Document::default();
		doc.insert(Position::default(), "👩💻");
		let cursor = doc.insert(
			Position {
				row: 0,
				byte: "👩".len(),
			},
			"\u{200d}",
		);
		assert_eq!(cursor.byte, "👩‍💻".len());
		assert_eq!(doc.previous(cursor), Position::default());
		doc = Document::default();
		doc.insert(Position::default(), "e\n\u{301}");
		let cursor = doc.remove(Position { row: 0, byte: 1 }, Position { row: 1, byte: 0 });
		assert_eq!(doc.lines, ["e\u{301}"]);
		assert_eq!(cursor, Position::default());
	}

	#[test]
	fn display_mapping_handles_tabs_wide_and_combining_text() {
		let line = "a\t界e\u{301}";
		assert_eq!(column_at(line, line.len()), 7);
		assert_eq!(byte_at_column(line, 3), 1);
		assert_eq!(byte_at_column(line, 5), 2);
		assert_eq!(byte_at_column(line, 6), 5);
		assert_eq!(byte_at_column(line, 100), line.len());
		for glyph in glyphs(line) {
			assert_eq!(
				byte_at_column(line, column_at(line, glyph.byte)),
				glyph.byte
			);
		}
	}

	#[test]
	fn paste_normalizes_newlines_and_makes_controls_visible() {
		assert_eq!(
			sanitize("a\r\nb\rc\t\n\x1b[31m\0\u{85}"),
			"a\nb\nc\t\n\\u{1b}[31m\\u{0}\\u{85}"
		);
	}
}
