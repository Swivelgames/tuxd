//! Semantic control mechanics; callers own input mapping, validation and effects.
mod field;
pub use field::{Field, FieldAction};

pub fn cycle(index: usize, count: usize, back: bool) -> usize {
	if count == 0 {
		return 0;
	}
	(index.min(count - 1) + if back { count - 1 } else { 1 }) % count
}

#[derive(Clone, Copy)]
pub enum Direction {
	Previous,
	Next,
	Up,
	Down,
	PageUp,
	PageDown,
	Beginning,
	End,
}

/// Bounded row-major navigation, preserving the column until the final partial row.
pub fn navigate(
	index: usize,
	count: usize,
	columns: usize,
	page_rows: usize,
	direction: Direction,
) -> usize {
	if count == 0 {
		return 0;
	}
	let last = count - 1;
	let index = index.min(last);
	let columns = columns.max(1);
	match direction {
		Direction::Beginning => 0,
		Direction::End => last,
		Direction::Previous => index.saturating_sub(1),
		Direction::Next => index.saturating_add(1).min(last),
		Direction::Up => {
			if index >= columns {
				index - columns
			} else {
				index
			}
		}
		Direction::Down => {
			if index / columns < last / columns {
				index.saturating_add(columns).min(last)
			} else {
				index
			}
		}
		Direction::PageUp => index.saturating_sub(columns.saturating_mul(page_rows)),
		Direction::PageDown => index
			.saturating_add(columns.saturating_mul(page_rows))
			.min(last),
	}
}

#[derive(Clone, Copy)]
pub struct Range {
	pub min: u8,
	pub max: u8,
}

impl Range {
	pub fn step(self, value: u8, amount: i16) -> u8 {
		(value as i32 + amount as i32).clamp(self.min as i32, self.max as i32) as u8
	}

	pub fn at_cell(self, cell: usize, width: usize) -> u8 {
		let extent = width.saturating_sub(1);
		(self.min as usize + cell.min(extent) * (self.max - self.min) as usize / extent.max(1))
			as u8
	}
}
