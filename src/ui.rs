use ratatui::{
	Frame,
	layout::Rect,
	style::{Color, Style},
	widgets::{Block, Borders, Clear, Paragraph},
};
use unicode_width::UnicodeWidthStr;

use crate::{
	app::{App, Overlay},
	document::{column_at, glyphs},
};

pub fn draw(frame: &mut Frame, app: &mut App) {
	let area = frame.area();
	frame.render_widget(Clear, area);
	if area.width < 20 || area.height < 5 {
		app.editor_area = Rect::default();
		app.editor.resize(0, 0);
		let message = match app.overlay {
			Overlay::Quit => "Discard? Y/N",
			_ => "Resize · Ctrl+Q quit",
		};
		frame.render_widget(Paragraph::new(message), area);
		return;
	}
	let header = Rect::new(area.x, area.y, area.width, 1);
	frame.render_widget(
		Paragraph::new(" Terminal UX Designer · session only")
			.style(Style::default().fg(Color::Cyan)),
		header,
	);
	let body = Rect::new(area.x, area.y + 1, area.width, area.height - 2);
	let border = Block::default()
		.borders(Borders::ALL)
		.title(" Editor · untitled ");
	let inner = border.inner(body);
	frame.render_widget(border, body);
	app.editor_area = inner;
	app.editor
		.resize(inner.width as usize, inner.height as usize);
	let editor = &app.editor;
	for (row, line) in editor
		.document
		.lines
		.iter()
		.skip(editor.top)
		.take(inner.height as usize)
		.enumerate()
	{
		for glyph in glyphs(line) {
			if glyph.column + glyph.width <= editor.left {
				continue;
			}
			if glyph.column >= editor.left + inner.width as usize {
				break;
			}
			// Leave clipped glyphs blank rather than rendering half a wide character.
			if glyph.column < editor.left
				|| glyph.column + glyph.width > editor.left + inner.width as usize
				|| glyph.text == "\t"
			{
				continue;
			}
			let display = if glyph.text.width() == 0 {
				format!("◌{}", glyph.text)
			} else {
				glyph.text.to_owned()
			};
			frame.buffer_mut().set_stringn(
				inner.x + (glyph.column - editor.left) as u16,
				inner.y + row as u16,
				display,
				glyph.width,
				Style::default(),
			);
		}
	}
	let column = column_at(
		&editor.document.lines[editor.cursor.row],
		editor.cursor.byte,
	);
	let status = format!(
		" Ln {}, Col {} · {} lines · Ctrl+Z/Y undo/redo · F1 help · Ctrl+Q quit",
		editor.cursor.row + 1,
		column + 1,
		editor.document.lines.len()
	);
	frame.render_widget(
		Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
		Rect::new(area.x, area.bottom() - 1, area.width, 1),
	);
	if app.overlay == Overlay::None
		&& editor.cursor.row >= editor.top
		&& editor.cursor.row < editor.top + inner.height as usize
		&& column >= editor.left
		&& column < editor.left + inner.width as usize
	{
		frame.set_cursor_position((
			inner.x + (column - editor.left) as u16,
			inner.y + (editor.cursor.row - editor.top) as u16,
		));
	}
	if app.overlay != Overlay::None {
		if area.width < 50 || area.height < 15 {
			frame.render_widget(Clear, area);
			let text = if app.overlay == Overlay::Quit {
				"Discard session?
Y: quit
N / Esc: keep editing
No saving yet."
			} else {
				"F1 / Esc: close help
Ctrl+Q: quit
Ctrl+Z/Y: undo/redo
Arrows, Home/End, PgUp/Dn
Ctrl+Home/End: start/end
Tab: 4 columns
Click: cursor
Wheel: scroll
No saving yet."
			};
			frame.render_widget(
				Paragraph::new(text).wrap(ratatui::widgets::Wrap { trim: false }),
				area,
			);
			return;
		}
		let (title, text, height) = match app.overlay {
			Overlay::Help => (
				" Help · F1 / Esc close ",
				"Arrows · Home/End · Page Up/Down\nCtrl+Home/End: document start/end\nCtrl+Z: undo · Ctrl+Y: redo\nTab: literal tab (4-column stops)\nClick: cursor · Wheel: scroll\nCtrl+Q: quit (Y/N if text exists)\n\nText is session-local. No saving yet.\nNo selection or styling in Slice 1.",
				13,
			),
			Overlay::Quit => (
				" Discard session? ",
				"Saving is not available yet.\nDiscard all text and quit?\nY: discard · N / Esc: keep editing",
				5,
			),
			Overlay::None => unreachable!(),
		};
		let width = area.width.saturating_sub(2).min(64);
		let height = area.height.saturating_sub(2).min(height);
		let popup = Rect::new(
			area.x + (area.width - width) / 2,
			area.y + (area.height - height) / 2,
			width,
			height,
		);
		frame.render_widget(Clear, popup);
		frame.render_widget(
			Paragraph::new(text)
				.wrap(ratatui::widgets::Wrap { trim: false })
				.block(Block::bordered().title(title)),
			popup,
		);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::editor::EditKind;
	use ratatui::{Terminal, backend::TestBackend};
	use std::time::Instant;

	#[test]
	fn renders_unicode_and_tabs_at_shared_display_coordinates() {
		let mut terminal = Terminal::new(TestBackend::new(40, 10)).unwrap();
		let mut app = App::default();
		app.editor
			.insert("a\t界e\u{301}", EditKind::Atomic, Instant::now());
		terminal.draw(|frame| draw(frame, &mut app)).unwrap();
		let buffer = terminal.backend().buffer();
		assert_eq!(buffer[(1, 2)].symbol(), "a");
		assert_eq!(buffer[(5, 2)].symbol(), "界");
		assert_eq!(buffer[(7, 2)].symbol(), "e\u{301}");
		assert_eq!(app.editor_area, Rect::new(1, 2, 38, 6));
	}

	#[test]
	fn tiny_sizes_and_overlays_never_lose_the_document() {
		let mut app = App::default();
		app.editor
			.insert("keep me", EditKind::Atomic, Instant::now());
		for (width, height) in [(1, 1), (12, 3), (20, 5), (50, 15), (120, 30)] {
			let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
			for overlay in [Overlay::None, Overlay::Help, Overlay::Quit] {
				app.overlay = overlay;
				terminal.draw(|frame| draw(frame, &mut app)).unwrap();
				assert_eq!(app.editor.document.lines, ["keep me"]);
			}
		}
	}
}
