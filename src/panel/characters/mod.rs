//! Character browsing state; catalog policy and search live in the library.
mod clipboard;
mod dropdown;
mod grid;
mod input;
mod view;

#[cfg(test)]
mod tests;
use crate::libraries::{
	characters::CATALOG,
	controls::{self, Direction, Field},
};
use ratatui::layout::Rect;

pub const MIN_HEIGHT: usize = 23;
pub const ROWS: usize = MIN_HEIGHT - 7;

pub struct Characters {
	pub collapsed: bool,
	pub group: usize,
	pub query: Field,
	pub local_search: bool,
	pub results: Vec<char>,
	pub selected: usize,
	grid: Vec<grid::Cell>,
	closed: std::collections::BTreeSet<(usize, usize)>,
	width: usize,
	pub content_rows: usize,
	pub top: usize,
	pub rows: usize,
	pub height: usize,
	pub dropdown: Option<dropdown::Dropdown>,
	pub area: Rect,
	paste: Option<crate::libraries::clipboard::ReadRequest>,
}
impl Default for Characters {
	fn default() -> Self {
		let mut this = Self {
			collapsed: false,
			group: 0,
			query: Field::default(),
			local_search: false,
			results: Vec::new(),
			selected: 0,
			grid: Vec::new(),
			closed: Default::default(),
			width: 28,
			content_rows: 0,
			top: 0,
			rows: ROWS,
			height: MIN_HEIGHT,
			dropdown: None,
			area: Rect::default(),
			paste: None,
		};
		this.refresh();
		this
	}
}
impl Characters {
	pub fn resize(&mut self, available: usize, width: usize) -> bool {
		let height = available.max(MIN_HEIGHT);
		let width = width.max(1);
		let changed = self.height != height || self.width != width;
		self.height = height;
		self.rows = height - 7;
		if self.width != width {
			self.width = width;
			self.reflow();
		}
		self.scroll(self.top);
		changed
	}

	pub fn refresh(&mut self) {
		let group = &CATALOG.groups[self.group];
		let browsing = self.query.text().trim().is_empty();
		self.results = CATALOG.search(
			self.query.text(),
			(browsing || self.local_search).then_some(group.name),
			None,
		);
		self.selected = 0;
		self.top = 0;
		self.reflow();
		self.selected = self
			.grid
			.iter()
			.position(|cell| matches!(cell.control, crate::panel::Control::Glyph(_)))
			.unwrap_or(0);
	}
	pub fn insert(&mut self, text: &str) {
		let text: String = text.chars().filter(|c| !c.is_control()).collect();
		self.query.insert(&text, 256);
		self.refresh();
	}
	pub fn choose_group(&mut self, direction: Direction) {
		let len = CATALOG.groups.len();
		self.group = controls::navigate(self.group, len, len, 1, direction);
		self.refresh();
	}
}
