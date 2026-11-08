use super::{Focus, Picker};
use crate::files::Action;
use crate::{
	components::{
		field,
		scrollbar::{self, Axis, Bar},
	},
	libraries::text::display::sanitize,
	theme,
};
use ratatui::{
	Frame,
	layout::Rect,
	style::{Modifier, Style},
	text::{Line, Span},
	widgets::{BorderType, Clear, Paragraph},
};

impl Picker {
	pub fn draw(&mut self, frame: &mut Frame, title: &str, error: Option<&str>) {
		let area = frame.area();
		let width = area
			.width
			.saturating_sub(if area.width > 40 { 4 } else { 0 })
			.min(96);
		let available = area
			.height
			.saturating_sub(if area.height > 16 { 2 } else { 0 });
		let height = available.min(34);
		let popup = Rect::new(
			(area.width - width) / 2,
			(area.height - height) / 2,
			width,
			height,
		);
		frame.render_widget(Clear, popup);
		let block = theme::block()
			.border_type(BorderType::Rounded)
			.title(if self.import_prompt {
				" Import this file as ANSI? ".to_owned()
			} else {
				format!(" {title} ")
			});
		let inner = block.inner(popup);
		frame.render_widget(block, popup);
		self.buttons.clear();
		self.format_buttons.clear();
		self.bar = None;
		self.list_area = Rect::default();
		self.name_area = Rect::default();
		self.location_area = Rect::default();
		if inner.width == 0 || inner.height == 0 {
			return;
		}
		let row = |y: u16| {
			Rect::new(
				inner.x + 1.min(inner.width),
				inner.y + y.min(inner.height),
				inner.width.saturating_sub(2),
				u16::from(y < inner.height),
			)
		};
		if inner.height < 10 {
			if matches!(self.focus, Focus::List | Focus::Location) {
				self.focus = Focus::Name;
			}
			self.name_area = row(0);
			field::draw(&self.name, frame, self.name_area, self.focus == Focus::Name);
			if inner.height > 1 {
				frame.render_widget(Paragraph::new("Enter: choose · Esc: cancel"), row(1));
			}
			if self.has_format() && inner.height > 2 {
				self.draw_format(frame, row(2));
			}
			return;
		}
		let label_width = 11.min(row(0).width);
		frame.render_widget(Paragraph::new("Location:"), row(0));
		self.location_area = Rect::new(
			row(0).x + label_width,
			row(0).y,
			row(0).width - label_width,
			1,
		);
		field::draw(
			&self.location,
			frame,
			self.location_area,
			self.focus == Focus::Location,
		);
		let list_start = if self.has_format() {
			self.draw_format(frame, row(2));
			4
		} else {
			1
		};
		let list_box = Rect::new(
			inner.x,
			inner.y + list_start,
			inner.width,
			inner.height.saturating_sub(list_start + 3),
		);
		let block = theme::block()
			.border_type(BorderType::Rounded)
			.border_style(Style::default().fg(if self.focus == Focus::List {
				theme::accent_color()
			} else {
				theme::border_color()
			}));
		let list_inner = block.inner(list_box);
		frame.render_widget(block, list_box);
		self.list_area = Rect::new(
			list_inner.x + 1.min(list_inner.width),
			list_inner.y,
			list_inner.width.saturating_sub(2),
			list_inner.height,
		);
		if let Some(explorer) = &self.explorer {
			let count = explorer.files().len();
			self.top = self
				.top
				.min(count.saturating_sub(self.list_area.height as usize));
			for (index, file) in explorer
				.files()
				.iter()
				.enumerate()
				.skip(self.top)
				.take(self.list_area.height as usize)
			{
				let rect = Rect::new(
					self.list_area.x,
					self.list_area.y + (index - self.top) as u16,
					self.list_area.width,
					1,
				);
				let selected = explorer.selected_idx() == index;
				let style = Style::default()
					.fg(if file.is_dir {
						theme::accent_color()
					} else {
						theme::text_color()
					})
					.bg(if selected {
						theme::selected_color()
					} else {
						theme::surface_color()
					});
				frame.render_widget(
					Paragraph::new(format!(
						"{} {}",
						if selected && self.focus == Focus::List {
							"›"
						} else {
							" "
						},
						file.name
					))
					.style(style),
					rect,
				);
			}
			let mut bars = Vec::new();
			scrollbar::draw(
				frame,
				&mut bars,
				Bar {
					target: Axis::Vertical,
					area: Rect::new(
						self.list_area.right(),
						self.list_area.y,
						1,
						self.list_area.height,
					),
					content: count,
					viewport: self.list_area.height as usize,
					offset: self.top,
				},
			);
			self.bar = bars.first().copied();
		}
		let y = inner.height - 3;
		frame.render_widget(Paragraph::new("File/path:"), row(y));
		self.name_area = Rect::new(
			row(y).x + label_width,
			row(y).y,
			row(y).width - label_width,
			1,
		);
		field::draw(&self.name, frame, self.name_area, self.focus == Focus::Name);
		let label = match self.action {
			Action::Open => "Open",
			Action::Save | Action::SaveAs => "Save",
			Action::ImportRaw | Action::ImportEscaped => "Import",
			_ => "Export",
		};
		let button_row = row(y + 2);
		let start = button_row.width.saturating_sub(label.len() as u16 + 12);
		for (offset, text, focus) in [
			(start, label, Focus::Submit),
			(start + label.len() as u16 + 4, "Cancel", Focus::Cancel),
		] {
			if offset >= button_row.width {
				continue;
			}
			let rect = Rect::new(
				button_row.x + offset,
				button_row.y,
				(text.len() as u16 + 2).min(button_row.width - offset),
				1,
			);
			frame.render_widget(
				Paragraph::new(format!(" {text} ")).style(
					Style::default()
						.fg(theme::accent_color())
						.add_modifier(Modifier::BOLD)
						.bg(if self.focus == focus {
							theme::selected_color()
						} else {
							theme::field_color()
						}),
				),
				rect,
			);
			self.buttons.push((rect, focus));
		}
		if let Some(message) = self.error.as_deref().or(error) {
			frame.render_widget(
				Paragraph::new(format!("Error: {}", sanitize(message)))
					.style(Style::default().fg(theme::error_color())),
				row(y + 1),
			);
		} else if self.paste.is_some() {
			frame.render_widget(Paragraph::new("Reading clipboard…"), row(inner.height - 1));
		}
	}

	pub(super) fn draw_format(&mut self, frame: &mut Frame, row: Rect) {
		for (offset, label, escaped) in [(0, "Raw ANSI", false), (14, "Escaped ANSI", true)] {
			if offset >= row.width {
				continue;
			}
			let selected = escaped == self.escaped();
			let rect = Rect::new(
				row.x + offset,
				row.y,
				(label.len() as u16 + 3).min(row.width - offset),
				row.height,
			);
			let bg = if selected {
				theme::selected_color()
			} else {
				theme::surface_color()
			};
			let line = Line::from(vec![
				Span::styled(
					"● ",
					Style::default().fg(if selected {
						theme::accent_color()
					} else {
						theme::inactive_indicator_color()
					}),
				),
				Span::styled(
					label,
					Style::default().add_modifier(if selected {
						Modifier::BOLD
					} else {
						Modifier::empty()
					}),
				),
			]);
			frame.render_widget(
				Paragraph::new(line).style(Style::default().bg(bg).add_modifier(
					if self.focus == Focus::Format && selected {
						Modifier::UNDERLINED
					} else {
						Modifier::empty()
					},
				)),
				rect,
			);
			self.format_buttons.push((rect, escaped));
		}
	}
}
