use crate::{
	libraries::{ansi::Position, text::Layout},
	theme,
};
use ratatui::{
	Frame,
	layout::Rect,
	style::{Modifier, Style},
	widgets::Block,
};
use unicode_width::UnicodeWidthStr;
pub fn text_area(inner: Rect, count: usize) -> Rect {
	let gutter =
		(count.max(1).to_string().len().max(2) as u16 + 4).min(inner.width.saturating_sub(1));
	Rect::new(
		inner.x + gutter,
		inner.y,
		inner.width - gutter,
		inner.height,
	)
}

pub struct View<'a> {
	pub inner: Rect,
	pub text: Rect,
	pub lines: &'a [String],
	pub layout: &'a Layout,
	pub top: usize,
	pub left: usize,
	pub active_line: Option<usize>,
	pub gutter_style: Style,
}

pub fn draw_lines(frame: &mut Frame, view: View<'_>, style_at: impl Fn(Position) -> Style) {
	let View {
		inner,
		text,
		lines,
		layout,
		top,
		left,
		active_line,
		gutter_style,
	} = view;
	frame.render_widget(
		Block::default().style(gutter_style),
		Rect::new(inner.x, inner.y, text.x - inner.x, inner.height),
	);
	for (screen_row, row) in layout
		.rows
		.iter()
		.skip(top)
		.take(text.height as usize)
		.enumerate()
	{
		let y = text.y + screen_row as u16;
		if text.x > inner.x {
			let gutter_width = (text.x - inner.x) as usize;
			let number = if row.start == 0 {
				format!(
					"  {:>width$}  ",
					row.line + 1,
					width = gutter_width.saturating_sub(4)
				)
			} else {
				" ".repeat(gutter_width)
			};
			let gutter_style = if Some(row.line) == active_line {
				Style::default()
					.fg(theme::selected_text_color())
					.bg(theme::selected_color())
					.add_modifier(Modifier::BOLD)
			} else {
				gutter_style
			};
			frame
				.buffer_mut()
				.set_stringn(inner.x, y, number, gutter_width, gutter_style);
		}
		for cell in &row.cells {
			let start = cell.x.max(left);
			let end = (cell.x + cell.width).min(left + text.width as usize);
			if start >= end {
				continue;
			}
			let source = &lines[row.line][cell.byte..cell.end];
			let style = style_at(Position {
				row: row.line,
				byte: cell.byte,
			});
			let rendered = crate::libraries::text::display::display_grapheme(source);
			let display = if source == "\t" {
				" ".repeat(end - start)
			} else if rendered.is_ascii() {
				let mut visible = rendered[start - cell.x..]
					.chars()
					.take(end - start)
					.collect::<String>();
				// A fallback remains one editable grapheme even in a narrow wrapped row.
				if cell.width < rendered.len() && !visible.is_empty() {
					visible.pop();
					visible.push('…');
				}
				visible
			} else if start != cell.x || end != cell.x + cell.width {
				" ".repeat(end - start)
			} else if rendered.width() > cell.width {
				"�".into()
			} else {
				rendered.into_owned()
			};
			frame.buffer_mut().set_stringn(
				text.x + (start - left) as u16,
				y,
				display,
				end - start,
				style,
			);
		}
		// A selected newline gets a visible cell after the last document glyph.
		if row.end == lines[row.line].len()
			&& row.width >= left
			&& row.width < left + text.width as usize
		{
			frame.buffer_mut().set_string(
				text.x + (row.width - left) as u16,
				y,
				" ",
				style_at(Position {
					row: row.line,
					byte: row.end,
				}),
			);
		}
	}
}
