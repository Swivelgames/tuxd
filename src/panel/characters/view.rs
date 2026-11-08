use super::Characters;
use crate::{
	libraries::characters::CATALOG,
	panel::{Control, Item},
};

impl Characters {
	pub(in crate::panel) fn layout(&self) -> (Vec<Item>, usize) {
		let group = &CATALOG.groups[self.group];
		let mut items = Vec::new();
		let mut add = |label, control, row, column, columns| {
			items.push(Item {
				label,
				control,
				row,
				column,
				columns,
				ink: None,
			})
		};
		add(
			format!("Group: {} ▾", group.name),
			Control::CharacterGroup,
			1,
			0,
			1,
		);
		add("Search".into(), Control::CharacterSearch, 3, 0, 1);
		add(
			if self.local_search { "Group" } else { "All" }.into(),
			Control::CharacterScope,
			3,
			0,
			2,
		);
		add("x".into(), Control::CharacterClear, 3, 1, 2);

		let first = self.grid.partition_point(|cell| cell.row < self.top);
		for cell in self.grid[first..]
			.iter()
			.take_while(|cell| cell.row < self.top + self.rows)
		{
			let label = match cell.control {
				Control::CharacterType(group, subtype) => {
					let marker = if self.closed.contains(&(group, subtype)) {
						"▸"
					} else {
						"▾"
					};
					let group = &CATALOG.groups[group];
					let name = group.subgroups[subtype];
					if !self.query.text().trim().is_empty() && !self.local_search {
						format!("{marker} {} · {name}", group.name)
					} else {
						format!("{marker} {name}")
					}
				}
				Control::Glyph(glyph) => glyph.to_string(),
				_ => unreachable!(),
			};
			add(
				label,
				cell.control,
				5 + cell.row - self.top,
				cell.x,
				self.width,
			);
		}
		(items, self.height)
	}
}
