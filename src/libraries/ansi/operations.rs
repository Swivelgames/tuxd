use super::document::{boundary_after, boundary_before};
use super::{Action, Document, Error, Position, Styles, TextStyle};
use std::ops::Range;

impl Document {
	fn checked_range(&self, range: Range<Position>) -> Result<Range<usize>, Error> {
		let start = self.offset(range.start)?;
		let end = self.offset(range.end)?;
		if start > end {
			return Err(Error::ReversedRange);
		}
		Ok(start..end)
	}

	pub fn insert(&mut self, position: Position, text: &str) -> Result<Position, Error> {
		let fragment = Self::from_text(text, TextStyle::default())?;
		self.replace(position..position, &fragment)
	}

	pub fn remove(&mut self, start: Position, end: Position) -> Result<Position, Error> {
		self.replace(start..end, &Self::default())
	}

	/// Atomically replace a range with a styled fragment, then normalize joined graphemes.
	pub fn replace(&mut self, range: Range<Position>, fragment: &Self) -> Result<Position, Error> {
		let range = self.checked_range(range)?;
		let inserted = fragment.text();
		let mut text = self.text();
		text.replace_range(range.clone(), &inserted);

		let mut styles = Styles::default();
		for run in &self.styles.runs {
			styles.push(run.start, run.end.min(range.start), run.style);
		}
		for run in &fragment.styles.runs {
			styles.push(range.start + run.start, range.start + run.end, run.style);
		}
		for run in &self.styles.runs {
			if run.end > range.end {
				styles.push(
					run.start.max(range.end) - range.end + range.start + inserted.len(),
					run.end - range.end + range.start + inserted.len(),
					run.style,
				);
			}
		}

		let next = Self::from_parsed(text, styles);
		let mut cursor = next.position_at_offset(range.start + inserted.len());
		cursor.byte = if inserted.is_empty() {
			boundary_before(&next.lines[cursor.row], cursor.byte)
		} else {
			boundary_after(&next.lines[cursor.row], cursor.byte)
		};
		*self = next;
		Ok(cursor)
	}

	pub fn extract(&self, range: Range<Position>) -> Result<Self, Error> {
		let range = self.checked_range(range)?;
		let mut styles = Styles::default();
		for run in &self.styles.runs {
			if run.start < range.end && run.end > range.start {
				styles.push(
					run.start.max(range.start) - range.start,
					run.end.min(range.end) - range.start,
					run.style,
				);
			}
		}
		Ok(Self::from_parsed(self.text()[range].to_owned(), styles))
	}

	pub fn set_style(&mut self, range: Range<Position>, style: TextStyle) -> Result<(), Error> {
		let range = self.checked_range(range)?;
		style.validate()?;
		self.styles.apply(range.start, range.end, |_| style);
		Ok(())
	}

	pub fn apply_style(
		&mut self,
		range: Range<Position>,
		action: Action,
		enabled: bool,
	) -> Result<(), Error> {
		let range = self.checked_range(range)?;
		let mut candidate = TextStyle::default();
		candidate.set(action, enabled);
		candidate.validate()?;
		self.styles.apply(range.start, range.end, |mut style| {
			style.set(action, enabled);
			style
		});
		Ok(())
	}

	/// None means a mixed selection; an empty range has no active style.
	pub fn style_state(
		&self,
		range: Range<Position>,
		action: Action,
	) -> Result<Option<bool>, Error> {
		let range = self.checked_range(range)?;
		if range.is_empty() {
			return Ok(Some(false));
		}
		let first = self.styles.at(range.start).matches(action);
		let mixed = self.styles.runs.iter().any(|run| {
			run.end > range.start && run.start < range.end && run.style.matches(action) != first
		});
		Ok(if mixed { None } else { Some(first) })
	}
}
