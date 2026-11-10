use super::{Row, Sidebar};
use crate::{
	components::scrollbar::{self, Axis, Bar},
	libraries::text::display::sanitize,
	theme,
};
use ratatui::{
	Frame,
	layout::Rect,
	style::{Modifier, Style},
	text::Span,
	widgets::{Borders, Clear, Paragraph},
};

impl Sidebar {
	fn label(&self, row: &Row) -> String {
		match row {
			Row::Spacer => String::new(),
			Row::OpenFiles => format!(" {} Open Files ", if self.open_files { "▾" } else { "▸" }),
			Row::Root => {
				let name = self
					.root()
					.map(|path| {
						sanitize(
							&path
								.file_name()
								.unwrap_or(path.as_os_str())
								.to_string_lossy(),
						)
					})
					.unwrap_or_default();
				format!(" {} {name} ", if self.root_open { "▾" } else { "▸" })
			}
			Row::Buffer { label, .. } => format!(" {label}"),
			Row::Entry(entry) => format!(
				" {}{} {}{}",
				"  ".repeat(entry.depth.min(12)),
				if entry.expanded {
					"▾"
				} else if entry.directory {
					"▸"
				} else {
					" "
				},
				sanitize(&entry.path.file_name().unwrap_or_default().to_string_lossy()),
				if entry.directory { "/" } else { "" }
			),
		}
	}

	fn draw(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
		self.area = area;
		self.bars.clear();
		frame.render_widget(Clear, area);
		frame.render_widget(
			theme::block()
				.border_type(ratatui::widgets::BorderType::Rounded)
				.title(" tuxd ‹ "),
			area,
		);
		let height = area.height.saturating_sub(2) as usize;
		self.top = self.top.min(self.rows.len().saturating_sub(height));
		for (index, row) in self.rows.iter().enumerate().skip(self.top).take(height) {
			if matches!(row, Row::Spacer) {
				continue;
			}
			let y = area.y + 1 + (index - self.top) as u16;
			let heading = matches!(row, Row::OpenFiles | Row::Root);
			let style = row_style(row, focused && index == self.selected);
			if heading {
				draw_heading(
					frame,
					Rect::new(area.x, y, area.width, 1),
					self.label(row),
					style,
				);
				continue;
			}
			let width = area
				.width
				.saturating_sub(if matches!(row, Row::Buffer { .. }) {
					6
				} else {
					3
				});
			frame.render_widget(
				Paragraph::new(self.label(row)).style(style),
				Rect::new(area.x + 1, y, width, 1),
			);
			if let Row::Buffer { dirty, .. } = row {
				frame.render_widget(
					Paragraph::new(if *dirty { "*" } else { " " }).style(style),
					Rect::new(area.right() - 5, y, 1, 1),
				);
				frame.render_widget(
					Paragraph::new("x").style(style),
					Rect::new(area.right() - 3, y, 1, 1),
				);
			}
		}
		scrollbar::draw(
			frame,
			&mut self.bars,
			Bar {
				target: Axis::Vertical,
				area: Rect::new(area.right() - 2, area.y + 1, 1, height as u16),
				content: self.rows.len(),
				viewport: height,
				offset: self.top,
			},
		);
	}
}

fn row_style(row: &Row, selected: bool) -> Style {
	let emphasized = matches!(
		row,
		Row::OpenFiles | Row::Root | Row::Buffer { active: true, .. }
	);
	Style::default()
		.fg(if emphasized {
			theme::accent_color()
		} else {
			theme::text_color()
		})
		.bg(if selected {
			theme::selected_color()
		} else {
			theme::surface_color()
		})
		.add_modifier(if emphasized {
			Modifier::BOLD
		} else {
			Modifier::empty()
		})
}

fn draw_heading(frame: &mut Frame, area: Rect, label: String, style: Style) {
	frame.render_widget(
		theme::block()
			.borders(Borders::TOP)
			.title(Span::styled(label, style)),
		area,
	);
	let border = Style::default().fg(theme::border_color());
	for (x, symbol) in [(area.x, "├"), (area.right() - 1, "┤")] {
		frame.render_widget(
			Paragraph::new(symbol).style(border),
			Rect::new(x, area.y, 1, 1),
		);
	}
}

impl super::super::Workspace {
	pub(in crate::workspace) fn draw_directory_sidebar(&mut self, frame: &mut Frame, area: Rect) {
		self.refresh_sidebar_rows();
		self.sidebar
			.draw(frame, area, self.focus == super::super::Pane::Sidebar);
	}
}
