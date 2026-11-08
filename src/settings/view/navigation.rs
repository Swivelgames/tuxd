use crate::components::scrollbar::{self, Axis, Bar};
use crate::settings::sections::Section;
use crate::settings::view::text;
use crate::settings::{Hit, Settings};
use crate::theme::{self, accent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;

impl Settings {
	pub(super) fn draw_categories(&mut self, frame: &mut Frame, area: Rect) {
		let height = area.height as usize;
		if height == 0 {
			return;
		}
		let categories = self.categories();
		let at = categories
			.iter()
			.position(|cat| *cat == self.category)
			.unwrap();
		self.nav_top = self.nav_top.min(categories.len().saturating_sub(height));
		if self.follow {
			if at < self.nav_top {
				self.nav_top = at;
			} else if at >= self.nav_top + height {
				self.nav_top = at + 1 - height;
			}
		}
		for (i, cat) in categories
			.iter()
			.copied()
			.enumerate()
			.skip(self.nav_top)
			.take(height)
		{
			let row = Rect::new(
				area.x,
				area.y + (i - self.nav_top) as u16,
				area.width.saturating_sub(1),
				1,
			);
			let selected = cat == self.category;
			text(
				frame,
				row,
				format!(
					"{}{}",
					if matches!(cat, Section::ThemeGroup | Section::ShortcutsGroup) {
						if self.closed_groups.contains(&cat) {
							"▸ "
						} else {
							"▾ "
						}
					} else if selected {
						"  › "
					} else {
						"    "
					},
					cat.label()
				),
				if selected {
					accent().bg(theme::selected_color())
				} else {
					Style::default()
				},
			);
			self.hits.push((row, Hit::Category(cat)));
		}
		if height < categories.len() {
			let bar = Bar {
				target: Axis::Vertical,
				area: Rect::new(area.right() - 1, area.y, 1, area.height),
				content: categories.len(),
				viewport: height,
				offset: self.nav_top,
			};
			scrollbar::draw(frame, &mut Vec::new(), bar);
			self.nav_bar = Some(bar);
		}
	}
}
