//! Shared flow geometry for drawing, keyboard navigation, and scrolling.
use super::Characters;
use crate::{
	libraries::{characters::CATALOG, controls::Direction},
	panel::Control,
};
use unicode_width::UnicodeWidthChar;

pub(super) struct Cell {
	pub control: Control,
	pub row: usize,
	pub x: usize,
}

struct Section {
	group: usize,
	subtype: usize,
	glyphs: Vec<char>,
}

impl Characters {
	pub(super) fn reflow(&mut self) {
		let focused = self.selected_control();
		let mut sections: Vec<Section> = Vec::new();
		for &glyph in &self.results {
			let Some(entry) = CATALOG.get(glyph) else {
				continue;
			};
			let group = CATALOG
				.groups
				.iter()
				.position(|group| group.name == entry.group)
				.unwrap();
			let subtype = CATALOG.groups[group]
				.subgroups
				.iter()
				.position(|name| *name == entry.subgroup)
				.unwrap();
			if let Some(section) = sections
				.iter_mut()
				.find(|section| section.group == group && section.subtype == subtype)
			{
				section.glyphs.push(glyph);
			} else {
				sections.push(Section {
					group,
					subtype,
					glyphs: vec![glyph],
				});
			}
		}

		self.grid.clear();
		let mut row = 0;
		for Section {
			group,
			subtype,
			glyphs,
		} in sections
		{
			self.grid.push(Cell {
				control: Control::CharacterType(group, subtype),
				row,
				x: 0,
			});
			row += 2;
			if self.closed.contains(&(group, subtype)) {
				continue;
			}
			let mut x = 0;
			for glyph in glyphs {
				let width = glyph.width().unwrap_or(1).max(1);
				if width > self.width {
					continue;
				}
				if x > 0 && x + width > self.width {
					row += 2;
					x = 0;
				}
				self.grid.push(Cell {
					control: Control::Glyph(glyph),
					row,
					x,
				});
				x += width + 1;
			}
			row += 2;
		}
		self.content_rows = self.grid.last().map_or(0, |cell| cell.row + 1);
		self.selected = focused
			.and_then(|control| self.grid.iter().position(|cell| cell.control == control))
			.unwrap_or(0);
		self.scroll(self.top);
	}

	pub fn selected_control(&self) -> Option<Control> {
		self.grid.get(self.selected).map(|cell| cell.control)
	}

	pub fn select(&mut self, control: Control) {
		if let Some(index) = self.grid.iter().position(|cell| cell.control == control) {
			self.selected = index;
		}
	}

	pub fn selected_glyph(&self) -> Option<char> {
		match self.selected_control()? {
			Control::Glyph(glyph) => Some(glyph),
			_ => None,
		}
	}

	pub fn toggle_type(&mut self, group: usize, subtype: usize) {
		let key = (group, subtype);
		self.select(Control::CharacterType(group, subtype));
		if !self.closed.remove(&key) {
			self.closed.insert(key);
		}
		self.reflow();
		self.reveal_selection();
	}

	pub fn navigate(&mut self, direction: Direction) {
		let Some(cell) = self.grid.get(self.selected) else {
			return;
		};
		let last = self.grid.len() - 1;
		self.selected = match direction {
			Direction::Previous => self.selected.saturating_sub(1),
			Direction::Next => (self.selected + 1).min(last),
			Direction::Beginning => 0,
			Direction::End => last,
			Direction::Up | Direction::Down | Direction::PageUp | Direction::PageDown => {
				let back = matches!(direction, Direction::Up | Direction::PageUp);
				let page = matches!(direction, Direction::PageUp | Direction::PageDown);
				let step = if page { self.rows } else { 1 };
				let target = if back {
					cell.row.saturating_sub(step)
				} else {
					cell.row + step
				};
				self.grid
					.iter()
					.enumerate()
					.filter(|(_, candidate)| {
						if back {
							candidate.row < cell.row
						} else {
							candidate.row > cell.row
						}
					})
					.min_by_key(|(_, candidate)| {
						(candidate.row.abs_diff(target), candidate.x.abs_diff(cell.x))
					})
					.map_or(self.selected, |(index, _)| index)
			}
		};
		self.reveal_selection();
	}

	pub fn reveal_selection(&mut self) {
		if let Some(cell) = self.grid.get(self.selected) {
			self.top = self
				.top
				.min(cell.row)
				.max(cell.row.saturating_sub(self.rows - 1));
		}
		self.scroll(self.top);
	}

	pub fn scroll(&mut self, top: usize) {
		self.top = top.min(self.content_rows.saturating_sub(self.rows));
	}

	pub fn select_visible(&mut self) {
		let row = self.grid.get(self.selected).map_or(0, |cell| cell.row);
		if !(self.top..self.top + self.rows).contains(&row) {
			self.selected = self
				.grid
				.iter()
				.enumerate()
				.filter(|(_, cell)| (self.top..self.top + self.rows).contains(&cell.row))
				.min_by_key(|(_, cell)| cell.row.abs_diff(row))
				.map_or(0, |(index, _)| index);
		}
	}
}
