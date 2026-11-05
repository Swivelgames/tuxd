use crate::document::{Position, glyphs};

#[derive(Debug)]
pub struct Cell {
	pub byte: usize,
	pub end: usize,
	pub x: usize,
	pub width: usize,
}

#[derive(Debug)]
pub struct Row {
	pub line: usize,
	pub start: usize,
	pub end: usize,
	pub width: usize,
	pub cells: Vec<Cell>,
}

/// One mapping shared by rendering, cursor movement, scrolling, and mouse input.
pub struct Layout {
	pub rows: Vec<Row>,
}

impl Layout {
	pub fn new(lines: &[String], width: usize, wrap: bool, cursor_space: bool) -> Self {
		let width = width.max(1);
		let mut rows = Vec::new();
		for (line, text) in lines.iter().enumerate() {
			let mut row = Row {
				line,
				start: 0,
				end: 0,
				width: 0,
				cells: Vec::new(),
			};
			for glyph in glyphs(text) {
				let cells = if wrap {
					glyph.width.min(width)
				} else {
					glyph.width
				};
				if wrap && row.width > 0 && row.width + cells > width {
					rows.push(row);
					row = Row {
						line,
						start: glyph.byte,
						end: glyph.byte,
						width: 0,
						cells: Vec::new(),
					};
				}
				row.cells.push(Cell {
					byte: glyph.byte,
					end: glyph.byte + glyph.text.len(),
					x: row.width,
					width: cells,
				});
				row.width += cells;
				row.end = glyph.byte + glyph.text.len();
			}
			let full = wrap && cursor_space && row.width == width;
			rows.push(row);
			if full {
				rows.push(Row {
					line,
					start: text.len(),
					end: text.len(),
					width: 0,
					cells: Vec::new(),
				});
			}
		}
		Self { rows }
	}

	pub fn location(&self, position: Position) -> (usize, usize) {
		let index = self
			.rows
			.iter()
			.rposition(|row| row.line == position.row && row.start <= position.byte)
			.unwrap_or(0);
		let row = &self.rows[index];
		let x = row
			.cells
			.iter()
			.find(|cell| cell.byte >= position.byte)
			.map_or(row.width, |cell| cell.x);
		(index, x)
	}

	pub fn hit(&self, row: usize, x: usize) -> Position {
		let row = &self.rows[row.min(self.rows.len() - 1)];
		Position {
			row: row.line,
			byte: row
				.cells
				.iter()
				.find(|cell| x < cell.x + cell.width)
				.map_or(row.end, |cell| cell.byte),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn wraps_graphemes_and_tabs_and_maps_continuations() {
		let lines = vec!["ab界e\u{301}\tZ".into(), String::new()];
		let layout = Layout::new(&lines, 4, true, true);
		assert_eq!(
			layout.rows.iter().map(|row| row.line).collect::<Vec<_>>(),
			[0, 0, 0, 1]
		);
		for row in &layout.rows {
			for cell in &row.cells {
				let position = Position {
					row: row.line,
					byte: cell.byte,
				};
				let (y, x) = layout.location(position);
				assert_eq!(layout.hit(y, x), position);
				assert!(cell.x + cell.width <= 4);
			}
		}
	}

	#[test]
	fn full_rows_leave_a_visible_end_cursor_and_narrow_glyphs_fit() {
		let lines = vec!["abcd".into()];
		let layout = Layout::new(&lines, 4, true, true);
		assert_eq!(layout.location(Position { row: 0, byte: 4 }), (1, 0));
		assert_eq!(Layout::new(&lines, 4, true, false).rows.len(), 1);
		let narrow = Layout::new(&["界\t".into()], 1, true, true);
		assert!(narrow.rows.iter().all(|row| row.width <= 1));
	}
}
