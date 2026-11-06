use crate::styling::{Action, Ink};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
	Section(usize),
	Style(Action),
	Target(bool),
	Palette(usize),
	GlyphTab(usize),
	Glyph(char),
	Field(usize),
	Apply,
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
	GlyphTabs,
	Glyphs(usize),
}
impl Item {
	pub fn group(&self) -> Group {
		match self.control {
			Control::Section(n) => Group::Section(n),
			Control::Target(_) => Group::Target,
			Control::Palette(_) => Group::Palette,
			Control::GlyphTab(_) => Group::GlyphTabs,
			Control::Field(n) => Group::Field(n),
			Control::Apply => Group::Apply,
			Control::Glyph(ch) => Group::Glyphs(if BOX.iter().any(|(c, _)| *c == ch) {
				0
			} else if BLOCKS.iter().any(|(c, _)| *c == ch) {
				1
			} else {
				2
			}),
			Control::Style(Action::Foreground(ink) | Action::Background(ink)) => match ink {
				Ink::Ansi(_) => Group::Colors(0),
				Ink::Indexed(_) => Group::Colors(1),
				_ => Group::Reset,
			},
			Control::Style(_) => Group::Styles,
		}
	}
}
pub struct Panel {
	pub field_cursors: [usize; 4],
	group_memory: std::collections::BTreeMap<Group, Control>,
	pub active_section: Option<usize>,
	pub gauge_areas: Vec<(ratatui::layout::Rect, usize)>,
	pub gauge_drag: Option<usize>,
	pub indexed_top: usize,
	pub indexed_area: ratatui::layout::Rect,
	pub collapsed: [bool; 3],
	pub background: bool,
	pub palette: usize,
	pub glyph_tab: usize,
	pub fields: [String; 4],
	pub replace_field: bool,
	pub error: Option<String>,
	pub top: usize,
	pub height: usize,
	pub area: ratatui::layout::Rect,
}

impl Default for Panel {
	fn default() -> Self {
		Self {
			field_cursors: [7, 3, 3, 3],
			group_memory: Default::default(),
			active_section: None,
			gauge_areas: Vec::new(),
			gauge_drag: None,
			indexed_top: 0,
			indexed_area: Default::default(),
			collapsed: [false; 3],
			background: false,
			palette: 0,
			glyph_tab: 0,
			fields: ["#80c0ff".into(), "128".into(), "192".into(), "255".into()],
			replace_field: true,
			error: None,
			top: 0,
			height: 0,
			area: Default::default(),
		}
	}
}

pub const BOX: &[(char, &str)] = &[
	('─', "Horizontal"),
	('│', "Vertical"),
	('┌', "Top left"),
	('┐', "Top right"),
	('└', "Bottom left"),
	('┘', "Bottom right"),
	('├', "Tee right"),
	('┤', "Tee left"),
	('┬', "Tee down"),
	('┴', "Tee up"),
	('┼', "Cross"),
	('═', "Double horizontal"),
	('║', "Double vertical"),
	('╔', "Double top left"),
	('╗', "Double top right"),
	('╚', "Double bottom left"),
	('╝', "Double bottom right"),
	('╬', "Double cross"),
];
pub const BLOCKS: &[(char, &str)] = &[
	('█', "Full block"),
	('▀', "Upper half"),
	('▄', "Lower half"),
	('▌', "Left half"),
	('▐', "Right half"),
	('░', "Light shade"),
	('▒', "Medium shade"),
	('▓', "Dark shade"),
];
pub const NERD: &[(char, &str)] = &[
	('\u{f015}', "Home"),
	('\u{f07b}', "Folder"),
	('\u{f15b}', "File"),
	('\u{f013}', "Gear"),
	('\u{f00c}', "Check"),
	('\u{f00d}', "Close"),
	('\u{f071}', "Warning"),
	('\u{f120}', "Terminal"),
];

impl Panel {
	pub fn glyphs(&self) -> &'static [(char, &'static str)] {
		match self.glyph_tab {
			1 => BLOCKS,
			2 => NERD,
			_ => BOX,
		}
	}
	pub fn color_action(&self, ink: Ink) -> Action {
		if self.background {
			Action::Background(ink)
		} else {
			Action::Foreground(ink)
		}
	}
	pub fn rgb(&self) -> Result<Ink, String> {
		let hex = self.fields[0]
			.trim()
			.strip_prefix('#')
			.unwrap_or(self.fields[0].trim());
		if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
			return Err("Use #RRGGBB (6 hex digits)".into());
		}
		let mut rgb = [0; 3];
		for (index, component) in rgb.iter_mut().enumerate() {
			*component = self.fields[index + 1]
				.parse::<u8>()
				.map_err(|_| "RGB components: 0–255".to_owned())?;
		}
		Ok(Ink::Rgb(rgb[0], rgb[1], rgb[2]))
	}
	pub fn edit(&mut self, field: usize, text: &str, backspace: bool) {
		let text = crate::document::sanitize(text).replace(['\n', '\t'], " ");
		use unicode_segmentation::UnicodeSegmentation;
		let cursor = self.field_cursors[field].min(self.fields[field].len());
		let mut cursor = cursor;
		while !self.fields[field].is_char_boundary(cursor) {
			cursor -= 1;
		}
		self.field_cursors[field] = cursor;
		if self.replace_field {
			self.fields[field].clear();
			self.field_cursors[field] = 0;
		}
		let was_selected = self.replace_field;
		self.replace_field = false;
		if backspace && !was_selected {
			let cursor = self.field_cursors[field];
			let previous = self.fields[field][..cursor]
				.grapheme_indices(true)
				.next_back()
				.map_or(0, |(at, _)| at);
			self.fields[field].replace_range(previous..cursor, "");
			self.field_cursors[field] = previous;
		} else if !backspace && self.fields[field].len() + text.len() <= 16 {
			self.fields[field].insert_str(self.field_cursors[field], &text);
			self.field_cursors[field] += text.len();
		}
		if field == 0 {
			let value = self.fields[0].trim();
			let value = value.strip_prefix('#').unwrap_or(value);
			if value.len() == 6 && value.bytes().all(|b| b.is_ascii_hexdigit()) {
				let value = u32::from_str_radix(value, 16).unwrap();
				for (i, shift) in [16, 8, 0].into_iter().enumerate() {
					self.fields[i + 1] = ((value >> shift) & 255).to_string();
				}
			}
		} else if let (Ok(r), Ok(g), Ok(b)) = (
			self.fields[1].parse::<u8>(),
			self.fields[2].parse::<u8>(),
			self.fields[3].parse::<u8>(),
		) {
			self.fields[0] = format!("#{r:02x}{g:02x}{b:02x}");
		}
		for n in 0..4 {
			if n != field {
				self.field_cursors[n] = self.fields[n].len();
			}
		}
		self.error = self.rgb().err();
	}
	pub fn move_caret(&mut self, field: usize, action: crate::shortcuts::ActionId) {
		use crate::shortcuts::ActionId as A;
		use unicode_segmentation::UnicodeSegmentation;
		let text = &self.fields[field];
		let cursor = self.field_cursors[field].min(text.len());
		self.field_cursors[field] = match action {
			A::FieldHome => 0,
			A::FieldEnd => text.len(),
			A::FieldLeft => text[..cursor]
				.grapheme_indices(true)
				.next_back()
				.map_or(0, |(i, _)| i),
			A::FieldRight => cursor + text[cursor..].graphemes(true).next().map_or(0, str::len),
			_ => cursor,
		};
		self.replace_field = false;
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
		let target = groups[(at + if back { groups.len() - 1 } else { 1 }) % groups.len()];
		let preferred = match target {
			Group::Target => Some(Control::Target(self.background)),
			Group::Palette => Some(Control::Palette(self.palette)),
			Group::GlyphTabs => Some(Control::GlyphTab(self.glyph_tab)),
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
			if !(self.indexed_top..self.indexed_top + 5).contains(&row) {
				return None;
			}
			return Some(item.row - self.indexed_top * 2);
		}
		Some(item.row)
	}

	pub fn set_component(&mut self, field: usize, value: u8) {
		self.replace_field = true;
		self.edit(field, &value.to_string(), false);
		self.replace_field = true;
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
			let height = if self.active_section.is_none() && self.collapsed[section] {
				2
			} else {
				match section {
					0 => {
						for (i, (name, action)) in [
							("B", Action::Bold),
							("I", Action::Italic),
							("U", Action::Underline),
							("D", Action::Dim),
							("S", Action::Strike),
							("Clear", Action::Clear),
						]
						.into_iter()
						.enumerate()
						{
							add(name.into(), Control::Style(action), 1, i, 6, None);
						}
						3
					}
					1 => {
						for (i, name) in ["Foreground", "Background"].into_iter().enumerate() {
							add(name.into(), Control::Target(i == 1), 1, i, 2, None);
						}
						for (i, name) in ["Basic", "256", "RGB"].into_iter().enumerate() {
							add(name.into(), Control::Palette(i), 3, i, 3, None);
						}
						match self.palette {
							0 | 1 => {
								let count = if self.palette == 0 { 16 } else { 256 };
								let columns = if self.palette == 0 { 8 } else { 7 };
								for n in 0..count {
									let ink = if self.palette == 0 {
										Ink::Ansi(n as u8)
									} else {
										Ink::Indexed(n as u8)
									};
									add(
										format!("{n:02}"),
										Control::Style(self.color_action(ink)),
										5 + n / columns * 2,
										n % columns,
										columns,
										Some(ink),
									);
								}
								let reset_row = 5 + count.div_ceil(columns).min(5) * 2;
								add(
									"Reset".into(),
									Control::Style(self.color_action(Ink::Default)),
									reset_row,
									0,
									1,
									None,
								);
								reset_row + 2
							}
							_ => {
								for (i, label) in ["Hex", "R", "G", "B"].into_iter().enumerate() {
									add(
										format!("{label}: {}", self.fields[i]),
										Control::Field(i),
										if i == 0 { 5 } else { 6 + i },
										0,
										1,
										None,
									);
								}
								add("Apply".into(), Control::Apply, 11, 0, 1, None);
								add(
									"Reset".into(),
									Control::Style(self.color_action(Ink::Default)),
									11,
									0,
									1,
									None,
								);
								13
							}
						}
					}
					_ => {
						for (i, name) in ["Box", "Blocks", "Nerd"].into_iter().enumerate() {
							add(name.into(), Control::GlyphTab(i), 1, i, 3, None);
						}
						for (i, (ch, _)) in self.glyphs().iter().enumerate() {
							add(
								ch.to_string(),
								Control::Glyph(*ch),
								3 + i / 6 * 2,
								i % 6,
								6,
								None,
							);
						}
						8 + (self.glyphs().len().div_ceil(6) - 1) * 2
					}
				}
			};
			sections.push((start, height));
			start += height + 1;
		}
		if self.active_section.is_some() {
			items.retain(|item| !matches!(item.control, Control::Section(_)));
		}
		(items, sections)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn rgb_validation_and_synchronization() {
		let mut panel = Panel::default();
		panel.edit(0, "#00ff80", false);
		assert_eq!(panel.rgb(), Ok(Ink::Rgb(0, 255, 128)));
		panel.replace_field = true;
		panel.edit(1, "256", false);
		assert!(panel.rgb().is_err());
		panel.replace_field = true;
		panel.edit(1, "255", false);
		assert_eq!(panel.fields[0], "#ffff80");
		assert_eq!(panel.rgb(), Ok(Ink::Rgb(255, 255, 128)));
		for invalid in ["", "#fff", "#gg0000", "🦀000", "0000000", "\x1b[31m"] {
			panel.replace_field = true;
			panel.edit(0, invalid, false);
			assert!(panel.rgb().is_err(), "{invalid}");
		}
	}
}
