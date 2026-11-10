//! Document tabs retain their active item when labels overflow the terminal.
use crate::theme;
use ratatui::{
	Frame,
	layout::Rect,
	style::{Modifier, Style},
	widgets::Paragraph,
};

struct Tab {
	id: crate::session::SessionId,
	label: String,
	width: u16,
}

impl super::Workspace {
	pub(super) fn draw_tabs(&mut self, frame: &mut Frame, area: Rect) {
		self.tab_hits.clear();
		self.tab_close_hits.clear();
		let max_width = area.width.saturating_sub(4).max(1) as usize;
		let tabs: Vec<_> = self
			.buffers()
			.map(|buffer| {
				let label = tab_label(&buffer.name(), buffer.session.dirty(), max_width);
				let width =
					unicode_width::UnicodeWidthStr::width(label.as_str()).min(max_width) as u16;
				Tab {
					id: buffer.session.id,
					label,
					width,
				}
			})
			.collect();

		self.tab_start = self.tab_start.min(self.active_index);
		while self.tab_start < self.active_index
			&& tabs[self.tab_start..=self.active_index]
				.iter()
				.map(|tab| u32::from(tab.width) + 1)
				.sum::<u32>()
				> u32::from(area.width.saturating_sub(4))
		{
			self.tab_start += 1;
		}
		frame.render_widget(
			Paragraph::new("  "),
			Rect::new(area.x, area.y, 2.min(area.width), 1),
		);
		if self.tab_start > 0 {
			frame.render_widget(Paragraph::new("‹"), Rect::new(area.x, area.y, 1, 1));
			self.tab_hits
				.push((Rect::new(area.x, area.y, 2, 1), self.tab_start - 1));
		}
		let mut x = area.x + 2;
		for (index, tab) in tabs.iter().enumerate().skip(self.tab_start) {
			let width = tab.width;
			if x + width > area.right().saturating_sub(2) {
				let rect = Rect::new(area.right().saturating_sub(2), area.y, 2, 1);
				frame.render_widget(Paragraph::new(" ›"), rect);
				self.tab_hits.push((rect, index));
				break;
			}
			let rect = Rect::new(x, area.y, width, 1);
			let style = if index == self.active_index {
				Style::default()
					.fg(theme::accent_color())
					.bg(theme::selected_color())
					.add_modifier(Modifier::BOLD)
			} else {
				theme::muted()
			};
			frame.render_widget(Paragraph::new(tab.label.as_str()).style(style), rect);
			self.tab_hits.push((rect, index));
			self.tab_close_hits
				.push((Rect::new(rect.right() - 2, rect.y, 1, 1), tab.id));
			x += width + 1;
			if x <= area.right().saturating_sub(2) {
				frame.render_widget(Paragraph::new(" "), Rect::new(x - 1, area.y, 1, 1));
			}
		}
	}
}

fn tab_label(name: &str, dirty: bool, width: usize) -> String {
	use unicode_segmentation::UnicodeSegmentation;
	use unicode_width::UnicodeWidthStr;
	let suffix = if dirty { " * x " } else { " x " };
	let available = width.saturating_sub(1 + suffix.len());
	let mut title = String::new();
	for grapheme in name.graphemes(true) {
		let displayed = crate::libraries::text::display::display_grapheme(grapheme);
		if title.width() + displayed.width() > available {
			break;
		}
		title.push_str(&displayed);
	}
	format!(" {title}{suffix}")
}
