use super::{Error, Run, Styles, TextStyle, data::validate_text};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
	pub row: usize,
	pub byte: usize,
}

/// Authoritative styled text. Mutation is only available through checked operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
	pub(super) lines: Vec<String>,
	pub(super) styles: Styles,
}

impl Default for Document {
	fn default() -> Self {
		Self {
			lines: vec![String::new()],
			styles: Styles::default(),
		}
	}
}

impl Document {
	pub fn from_text(text: &str, style: TextStyle) -> Result<Self, Error> {
		validate_text(text)?;
		style.validate()?;
		let mut styles = Styles::default();
		styles.push(0, text.len(), style);
		Ok(Self {
			lines: text.split('\n').map(str::to_owned).collect(),
			styles,
		})
	}

	pub fn lines(&self) -> &[String] {
		&self.lines
	}

	pub fn style_runs(&self) -> &[Run] {
		&self.styles.runs
	}

	pub fn text(&self) -> String {
		self.lines.join("\n")
	}

	pub fn is_empty(&self) -> bool {
		self.lines.len() == 1 && self.lines[0].is_empty()
	}

	pub fn end(&self) -> Position {
		let row = self.lines.len() - 1;
		Position {
			row,
			byte: self.lines[row].len(),
		}
	}

	pub fn offset(&self, position: Position) -> Result<usize, Error> {
		let line = self.lines.get(position.row).ok_or(Error::InvalidPosition)?;
		if !line
			.grapheme_indices(true)
			.map(|(byte, _)| byte)
			.chain([line.len()])
			.any(|byte| byte == position.byte)
		{
			return Err(Error::InvalidPosition);
		}
		Ok(self.lines[..position.row]
			.iter()
			.map(|line| line.len() + 1)
			.sum::<usize>()
			+ position.byte)
	}

	/// Query a source byte, including a newline. The end-of-document style is default.
	pub fn style_at_offset(&self, byte: usize) -> TextStyle {
		self.styles.at(byte)
	}

	/// Query a byte within a source line, or its end. Unlike edits, style lookup
	/// does not require a grapheme boundary or rescan the line for each painted cell.
	/// Panics if the row or byte lies outside the document.
	pub fn style_at(&self, position: Position) -> TextStyle {
		assert!(position.byte <= self.lines[position.row].len());
		let offset = self.lines[..position.row]
			.iter()
			.map(|line| line.len() + 1)
			.sum::<usize>()
			+ position.byte;
		self.styles.at(offset)
	}

	/// Move from a valid grapheme boundary; stays at the beginning of the document.
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

	/// Move from a valid grapheme boundary; stays at the end of the document.
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

	/// Find the next/previous word boundary from a valid grapheme position.
	pub fn word_target(&self, position: Position, backwards: bool) -> Position {
		let text = self.text();
		let offset = self.offset(position).expect("valid document position");
		// Unicode word segments retain punctuation/apostrophe behavior across lines.
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
		let mut result = self.position_at_offset(target);
		result.byte = if backwards {
			boundary_before(&self.lines[result.row], result.byte)
		} else {
			boundary_after(&self.lines[result.row], result.byte)
		};
		result
	}

	pub(super) fn position_at_offset(&self, mut offset: usize) -> Position {
		for (row, line) in self.lines.iter().enumerate() {
			if offset <= line.len() {
				return Position { row, byte: offset };
			}
			offset -= line.len() + 1;
		}
		self.end()
	}

	/// Internal parser/operation boundary: scalar runs adopt their grapheme's leading style.
	pub(super) fn from_parsed(text: String, styles: Styles) -> Self {
		let mut normalized = Styles::default();
		let mut runs = styles.runs.iter().peekable();
		for (byte, grapheme) in text.grapheme_indices(true) {
			while runs.peek().is_some_and(|run| run.end <= byte) {
				runs.next();
			}
			let style = runs
				.peek()
				.filter(|run| run.start <= byte)
				.map_or(TextStyle::default(), |run| run.style);
			normalized.push(byte, byte + grapheme.len(), style);
		}
		Self {
			lines: text.split('\n').map(str::to_owned).collect(),
			styles: normalized,
		}
	}
}

pub(super) fn boundary_before(line: &str, byte: usize) -> usize {
	line.grapheme_indices(true)
		.map(|(index, _)| index)
		.chain([line.len()])
		.take_while(|index| *index <= byte)
		.last()
		.unwrap_or(0)
}

pub(super) fn boundary_after(line: &str, byte: usize) -> usize {
	line.grapheme_indices(true)
		.map(|(index, _)| index)
		.chain([line.len()])
		.find(|index| *index >= byte)
		.unwrap_or(line.len())
}
