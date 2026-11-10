mod characters;
mod color;
mod input;
mod mouse;
pub mod scroll;
pub mod shortcuts;
mod style;
mod view;

pub use input::Effect;

use crate::libraries::ansi::{Action, Ink};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
	Section(usize),
	Style(Action),
	Target(bool),
	Palette(usize),
	CharacterGroup,
	CharacterType(usize, usize),
	CharacterSearch,
	CharacterScope,
	CharacterClear,
	Glyph(char),
	Field(usize),
	Apply,
}

impl Control {
	pub fn section(self) -> usize {
		match self {
			Self::Section(section) => section,
			Self::CharacterGroup
			| Self::CharacterType(_, _)
			| Self::CharacterSearch
			| Self::CharacterScope
			| Self::CharacterClear
			| Self::Glyph(_) => 2,
			Self::Target(_)
			| Self::Palette(_)
			| Self::Field(_)
			| Self::Apply
			| Self::Style(Action::Foreground(_) | Action::Background(_)) => 1,
			Self::Style(_) => 0,
		}
	}
}

pub struct Item {
	pub label: String,
	pub control: Control,
	pub row: usize,
	pub column: usize,
	pub columns: usize,
	pub ink: Option<Ink>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Group {
	Section(usize),
	Styles,
	Target,
	Palette,
	Colors(usize),
	Field(usize),
	Apply,
	Reset,
	CharacterCategory,
	CharacterSearch,
	CharacterScope,
	CharacterClear,
	Glyphs,
}

impl Item {
	pub fn group(&self) -> Group {
		match self.control {
			Control::Section(n) => Group::Section(n),
			Control::Target(_) => Group::Target,
			Control::Palette(_) => Group::Palette,
			Control::CharacterGroup => Group::CharacterCategory,
			Control::CharacterType(_, _) => Group::Glyphs,
			Control::CharacterSearch => Group::CharacterSearch,
			Control::CharacterScope => Group::CharacterScope,
			Control::CharacterClear => Group::CharacterClear,
			Control::Field(n) => Group::Field(n),
			Control::Apply => Group::Apply,
			Control::Glyph(_) => Group::Glyphs,
			Control::Style(Action::Foreground(ink) | Action::Background(ink)) => match ink {
				Ink::Ansi(_) => Group::Colors(0),
				Ink::Indexed(_) => Group::Colors(1),
				_ => Group::Reset,
			},
			Control::Style(_) => Group::Styles,
		}
	}
}

#[derive(Default)]
pub struct Panel {
	pub selected: usize,
	pub tool_areas: Vec<(ratatui::layout::Rect, usize)>,
	pub scrollbars: Vec<crate::components::scrollbar::Bar<scroll::Target>>,
	pub scrollbar_drag: Option<(scroll::Target, usize)>,
	pub style: style::Style,
	pub color: color::Color,
	pub characters: characters::Characters,
	group_memory: std::collections::BTreeMap<Group, Control>,
	pub active_section: Option<usize>,
	pub top: usize,
	pub height: usize,
	pub area: ratatui::layout::Rect,
}

impl Panel {
	/// Drop prior frame hit geometry, including when Tools will be hidden.
	pub fn hide(&mut self) {
		self.tool_areas.clear();
		self.scrollbars.clear();
		self.area = ratatui::layout::Rect::default();
		self.color.indexed_area = ratatui::layout::Rect::default();
		self.characters.area = ratatui::layout::Rect::default();
		self.color.gauge_areas.clear();
	}

	pub fn release_pointer(&mut self) {
		self.scrollbar_drag = None;
		self.color.gauge_drag = None;
	}

	pub fn collapsed(&self, section: usize) -> bool {
		match section {
			0 => self.style.collapsed,
			1 => self.color.collapsed,
			_ => self.characters.collapsed,
		}
	}

	pub fn toggle_section(&mut self, section: usize) {
		let collapsed = match section {
			0 => &mut self.style.collapsed,
			1 => &mut self.color.collapsed,
			_ => &mut self.characters.collapsed,
		};
		*collapsed = !*collapsed;
	}

	pub fn next_field(&mut self, index: usize, back: bool) -> usize {
		let items = self.items();
		if items.is_empty() {
			return 0;
		}
		let current = &items[index.min(items.len() - 1)];
		self.group_memory.insert(current.group(), current.control);
		let mut groups = Vec::new();
		for item in &items {
			if !groups.contains(&item.group()) {
				groups.push(item.group());
			}
		}
		let at = groups
			.iter()
			.position(|g| *g == current.group())
			.unwrap_or(0);
		let target = groups[crate::libraries::controls::cycle(at, groups.len(), back)];
		let preferred = match target {
			Group::Target => Some(Control::Target(self.color.background)),
			Group::Palette => Some(Control::Palette(self.color.palette)),
			Group::Glyphs => self.characters.selected_control(),
			_ => self.group_memory.get(&target).copied(),
		};
		items
			.iter()
			.position(|i| i.group() == target && Some(i.control) == preferred)
			.or_else(|| items.iter().position(|i| i.group() == target))
			.unwrap_or(0)
	}

	pub fn display_row(&self, item: &Item) -> Option<usize> {
		if let Some(Ink::Indexed(n)) = item.ink {
			let row = n as usize / 7;
			if !(self.color.indexed_top..self.color.indexed_top + 5).contains(&row) {
				return None;
			}
			return Some(item.row - self.color.indexed_top * 2);
		}
		Some(item.row)
	}

	pub fn items(&self) -> Vec<Item> {
		self.layout().0
	}

	/// Virtual rows shared by rendering, scrolling, and hit testing.
	pub fn layout(&self) -> (Vec<Item>, Vec<(usize, usize)>) {
		let mut items = Vec::new();
		let mut sections = Vec::new();
		let mut start = 0;
		for section in 0..3 {
			if self.active_section.is_some_and(|active| active != section) {
				sections.push((start, 0));
				continue;
			}
			let mut add = |label: String, control, row, column, columns, ink| {
				items.push(Item {
					label,
					control,
					row: start + row,
					column,
					columns,
					ink,
				});
			};
			add(
				["Style", "Color", "Characters"][section].into(),
				Control::Section(section),
				0,
				0,
				1,
				None,
			);
			let height = if self.active_section.is_none() && self.collapsed(section) {
				1
			} else {
				let (mut content, height) = match section {
					0 => self.style.layout(),
					1 => self.color.layout(),
					_ => self.characters.layout(),
				};
				for item in &mut content {
					item.row += start;
				}
				items.extend(content);
				height
			};
			sections.push((start, height));
			start += height;
		}
		if self.active_section.is_some() {
			items.retain(|item| !matches!(item.control, Control::Section(_)));
		}
		(items, sections)
	}
}

impl Panel {
	pub fn input_scope(&self, index: usize) -> crate::shortcuts::InputScope {
		use crate::shortcuts::InputScope as S;
		if self.characters.dropdown.is_some() {
			return S::CharacterPicker;
		}
		self.items()
			.get(index)
			.map_or(S::Tools, |item| match item.control {
				Control::Field(0) => S::Hex,
				Control::Field(_) => S::Channel,
				Control::Target(_) => S::Radio,
				Control::Palette(_) => S::ToolTabs,
				Control::CharacterSearch => S::CharacterSearch,
				Control::Section(2) => S::Characters,
				Control::CharacterGroup
				| Control::CharacterType(_, _)
				| Control::CharacterScope
				| Control::CharacterClear
				| Control::Glyph(_) => S::Characters,
				_ if item.ink.is_some() => S::Grid,
				_ => S::Tools,
			})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn rgb_validation_and_synchronization() {
		let mut panel = Panel::default();
		panel.color.rgb.edit(0, "#00ff80", false);
		assert_eq!(panel.color.rgb.rgb(), Ok(Ink::Rgb(0, 255, 128)));
		panel.color.rgb.replace_field = true;
		panel.color.rgb.edit(1, "256", false);
		assert!(panel.color.rgb.rgb().is_err());
		panel.color.rgb.replace_field = true;
		panel.color.rgb.edit(1, "255", false);
		assert_eq!(panel.color.rgb.fields[0].text(), "#ffff80");
		assert_eq!(panel.color.rgb.rgb(), Ok(Ink::Rgb(255, 255, 128)));
		for invalid in ["", "#fff", "#gg0000", "🦀000", "0000000", "\x1b[31m"] {
			panel.color.rgb.replace_field = true;
			panel.color.rgb.edit(0, invalid, false);
			assert!(panel.color.rgb.rgb().is_err(), "{invalid}");
		}
	}
}
