//! Display rows for escaped ANSI. Copy/export use the document encoder directly.
use crate::libraries::ansi::{Document, escape};

pub fn text(document: &Document) -> String {
	document
		.to_raw_ansi()
		.split_inclusive('\n')
		.map(escape)
		.collect::<Vec<_>>()
		.join("\n")
}

/// Encoded byte offsets exclude inserted display newlines; source rows track the
/// document-linked active gutter. Syntax tokens remain independent of appearance.
pub struct Projection {
	pub lines: Vec<String>,
	pub encoded_rows: Vec<std::ops::Range<usize>>,
	pub source_rows: Vec<usize>,
	pub highlights: Vec<super::highlight::Highlight>,
}

impl Projection {
	pub fn new(document: &Document) -> Self {
		let lines: Vec<String> = text(document).split('\n').map(str::to_owned).collect();
		let highlights = lines
			.iter()
			.map(|line| super::highlight::Highlight::new(line))
			.collect();
		let mut offset = 0;
		let encoded_rows = lines
			.iter()
			.map(|line| {
				let start = offset;
				offset += line.len();
				start..offset
			})
			.collect();
		let source_rows = (0..lines.len()).collect();
		Self {
			lines,
			encoded_rows,
			source_rows,
			highlights,
		}
	}

	pub fn active_line(&self, source: usize) -> Option<usize> {
		self.source_rows.iter().position(|row| *row == source)
	}

	pub fn style_at(&self, position: crate::libraries::ansi::Position) -> ratatui::style::Style {
		debug_assert!(position.byte <= self.encoded_rows[position.row].len());
		self.highlights[position.row].style_at(position.byte)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::libraries::ansi::TextStyle;

	#[test]
	fn display_rows_do_not_leak_into_escaped_output() {
		let document = Document::from_text("a\n\tb\\n\n", TextStyle::default()).unwrap();
		assert_eq!(
			document.to_escaped_ansi(),
			"\\x1b[0ma\\n\\tb\\\\n\\n\\x1b[0m"
		);
		assert_eq!(text(&document), "\\x1b[0ma\\n\n\\tb\\\\n\\n\n\\x1b[0m");
		assert_eq!(text(&document).lines().count(), 3);
	}

	#[test]
	fn encoded_offsets_and_document_rows_survive_escapes_and_trailing_newlines() {
		let document = Document::from_text("界\t\\x1b[31m\n\n", Default::default()).unwrap();
		let projection = Projection::new(&document);
		let encoded = document.to_escaped_ansi();
		for (line, range) in projection.lines.iter().zip(&projection.encoded_rows) {
			assert_eq!(&encoded[range.clone()], line);
		}
		assert_eq!(projection.lines.concat(), encoded);
		assert_eq!(projection.active_line(2), Some(2));
		assert_eq!(projection.active_line(3), None);
	}
}
