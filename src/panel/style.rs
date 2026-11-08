use super::{Action, Control, Item};

#[derive(Default)]
pub struct Style {
	pub collapsed: bool,
}

impl Style {
	pub(super) fn layout(&self) -> (Vec<Item>, usize) {
		let items = [
			("B", Action::Bold),
			("I", Action::Italic),
			("U", Action::Underline),
			("D", Action::Dim),
			("S", Action::Strike),
			("Clear", Action::Clear),
		]
		.into_iter()
		.enumerate()
		.map(|(column, (label, action))| Item {
			label: label.into(),
			control: Control::Style(action),
			row: 1,
			column,
			columns: 6,
			ink: None,
		})
		.collect();
		(items, 3)
	}
}
