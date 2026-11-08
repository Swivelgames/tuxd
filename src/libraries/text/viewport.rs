use super::Layout;
use crate::libraries::ansi::Position;

/// Owned display coordinates, independent of content and editing semantics.
#[derive(Default)]
pub struct Viewport {
	pub wrap: bool,
	pub top: usize,
	pub left: usize,
	pub width: usize,
	pub height: usize,
}

impl Viewport {
	pub fn layout(&self, lines: &[String], cursor_space: bool) -> Layout {
		Layout::new(lines, self.width, self.wrap, cursor_space)
	}

	pub fn toggle_wrap(&mut self) {
		self.wrap = !self.wrap;
		self.reset();
	}

	pub fn reset(&mut self) {
		self.top = 0;
		self.left = 0;
	}

	pub fn clamp(&mut self, layout: &Layout, cursor_space: bool) {
		self.top = self.top.min(layout.rows.len().saturating_sub(self.height));
		self.left = if self.wrap {
			0
		} else {
			let columns = layout.rows.iter().map(|row| row.width).max().unwrap_or(0);
			self.left
				.min((columns + usize::from(cursor_space)).saturating_sub(self.width))
		};
	}

	pub fn reveal(&mut self, layout: &Layout, cursor: Position) {
		// A zero-sized hidden pane retains a valid one-cell navigation viewport.
		let height = self.height.max(1);
		let width = self.width.max(1);
		let (row, column) = layout.location(cursor);
		self.top = self.top.min(layout.rows.len().saturating_sub(height));
		self.top = self.top.min(row).max((row + 1).saturating_sub(height));
		self.left = if self.wrap {
			0
		} else {
			let columns = layout.rows.iter().map(|row| row.width).max().unwrap_or(0);
			self.left
				.min((columns + 1).saturating_sub(width))
				.min(column)
				.max((column + 1).saturating_sub(width))
		};
	}

	pub fn hit(&self, layout: &Layout, column: usize, row: usize) -> Position {
		layout.hit(
			self.top.saturating_add(row),
			self.left.saturating_add(column),
		)
	}

	pub fn scroll(&mut self, down: bool, count: usize, layout: &Layout) {
		self.top = if down {
			self.top
				.saturating_add(count)
				.min(layout.rows.len().saturating_sub(self.height.max(1)))
		} else {
			self.top.saturating_sub(count)
		};
	}
}
