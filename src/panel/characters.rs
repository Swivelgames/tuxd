use super::{Control, Item};

#[derive(Default)]
pub struct Characters {
	pub collapsed: bool,
	pub tab: usize,
}
impl Characters {
	pub fn glyphs(&self) -> &'static [(char, &'static str)] {
		match self.tab {
			1 => BLOCKS,
			2 => NERD,
			_ => BOX,
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

impl Characters {
	pub(super) fn layout(&self) -> (Vec<Item>, usize) {
		let mut items = Vec::new();
		let mut add = |label: String, control, row, column, columns, ink| {
			items.push(Item {
				label,
				control,
				row,
				column,
				columns,
				ink,
			})
		};
		let height = {
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
		};
		(items, height)
	}
}
