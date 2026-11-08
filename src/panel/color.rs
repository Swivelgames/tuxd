mod clipboard;
mod rgb;
mod view;

pub use rgb::RgbDraft;
pub(super) use view::swatch_style;

use super::{Control, Item};
use crate::libraries::ansi::{Action, Ink};

#[derive(Default)]
pub struct Color {
	pub collapsed: bool,
	paste: Option<(usize, crate::libraries::clipboard::ReadRequest)>,
	pub background: bool,
	pub palette: usize,
	pub rgb: RgbDraft,
	pub indexed_top: usize,
	pub indexed_area: ratatui::layout::Rect,
	pub gauge_areas: Vec<(ratatui::layout::Rect, usize)>,
	pub gauge_drag: Option<usize>,
}

impl Color {
	pub fn color_action(&self, ink: Ink) -> Action {
		if self.background {
			Action::Background(ink)
		} else {
			Action::Foreground(ink)
		}
	}
}

impl Color {
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
							format!("{label}: {}", self.rgb.fields[i].text()),
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
		};
		(items, height)
	}
}
