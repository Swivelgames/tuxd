//! RGB field/gauge painting; the panel maps local bounds into its viewport.
use super::Color;
use crate::theme;
use ratatui::{
	buffer::Buffer,
	layout::Rect,
	style::{Color as InkColor, Modifier, Style},
	text::Span,
	widgets::{Gauge, Paragraph, Widget},
};
use unicode_width::UnicodeWidthStr;

impl Color {
	pub(in crate::panel) fn draw_field(
		&self,
		field: usize,
		rect: Rect,
		style: Style,
		focused: bool,
		buffer: &mut Buffer,
	) -> Option<Rect> {
		let label = ["Hex: ", "R: ", "G: ", "B: "][field];
		Paragraph::new(label)
			.style(Style::default().fg(theme::muted_color()))
			.render(Rect::new(rect.x, rect.y, label.len() as u16, 1), buffer);
		let input = Rect::new(
			rect.x + label.len() as u16,
			rect.y,
			if field == 0 {
				9.min(rect.width.saturating_sub(11))
			} else {
				rect.width.saturating_sub(label.len() as u16)
			},
			1,
		);
		let background = if focused {
			theme::focus_color()
		} else {
			theme::field_color()
		};
		if field == 0 {
			let field = &self.rgb.fields[0];
			let value = field.text();
			let cursor = value[..field.cursor()].width();
			let scroll = if focused && !self.rgb.replace_field {
				(cursor + 1).saturating_sub(input.width as usize)
			} else {
				0
			};
			Paragraph::new(value)
				.scroll((0, scroll as u16))
				.style(style.bg(background))
				.render(input, buffer);
			if focused && !self.rgb.replace_field && input.width > 0 {
				buffer[(input.x + (cursor - scroll) as u16, input.y)]
					.set_style(Style::default().add_modifier(Modifier::REVERSED));
			}

			let preview = Rect::new(rect.right().saturating_sub(5), rect.y, 5, 1);
			let preview_color = self
				.rgb
				.rgb()
				.map_or(background, crate::adapters::ratatui::ink_color);
			Paragraph::new(if self.rgb.rgb().is_ok() { "" } else { "  ?" })
				.style(Style::default().fg(theme::muted_color()).bg(preview_color))
				.render(preview, buffer);
		} else {
			let value = self.rgb.fields[field].text().parse::<u8>().unwrap_or(0);
			let fill = [
				InkColor::Reset,
				theme::rgb_red_color(),
				theme::rgb_green_color(),
				theme::rgb_blue_color(),
			][field];
			Gauge::default()
				.ratio(value as f64 / 255.0)
				.label(Span::styled(
					format!(
						"{:<width$}",
						self.rgb.fields[field].text(),
						width = input.width as usize
					),
					Style::default()
						.fg(theme::range_text_color())
						.add_modifier(if focused {
							Modifier::UNDERLINED
						} else {
							Modifier::empty()
						}),
				))
				.gauge_style(Style::default().fg(fill).bg(background))
				.render(input, buffer);
			return Some(input);
		}
		None
	}
}

impl Color {
	/// Keep the palette's button widths and gaps consistent at narrow sizes.
	pub(in crate::panel) fn swatch_bounds(
		&self,
		item: &crate::panel::Item,
		area_width: u16,
		row: u16,
	) -> Rect {
		let width = area_width.saturating_sub(4) as usize;
		let cell_width = width / item.columns;
		let button_width = (if self.palette == 0 { 2 } else { 3 })
			.min(cell_width)
			.max(1);
		let preferred_gap = if self.palette == 0 { 2 } else { 1 };
		let gap = preferred_gap
			.min(width.saturating_sub(button_width * item.columns) / (item.columns - 1));

		Rect::new(
			(2 + item.column * (button_width + gap)) as u16,
			row,
			button_width as u16,
			1,
		)
	}
}

/// Swatches preview document ink; UI theme colors must not replace their contrast.
pub(in crate::panel) fn swatch_style(
	ink: crate::libraries::ansi::Ink,
	style: Style,
	focused: bool,
	selected: bool,
) -> Style {
	use crate::libraries::ansi::Ink;
	let light = match ink {
		Ink::Rgb(r, g, b) => r as u32 * 299 + g as u32 * 587 + b as u32 * 114 > 140000,
		Ink::Ansi(n) | Ink::Indexed(n) => {
			matches!(n, 3 | 6 | 7 | 10..=15)
				|| (16..232).contains(&n) && (n - 16) / 6 % 6 >= 3
				|| n >= 246
		}
		_ => false,
	};
	let mut style = style
		.bg(crate::adapters::ratatui::ink_color(ink))
		.fg(if light {
			InkColor::Black
		} else {
			InkColor::White
		});
	if selected {
		style = style.add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
	}
	if focused {
		style = style.add_modifier(Modifier::REVERSED);
	}
	style
}
