use ratatui::{
	Frame,
	layout::Rect,
	style::{Color, Modifier, Style},
	widgets::{Block, Clear, Paragraph},
};
use unicode_width::UnicodeWidthStr;

use crate::{
	app::{App, Overlay},
	document::{Position, column_at},
	layout::Layout,
};

fn text_area(inner: Rect, count: usize) -> Rect {
	let gutter =
		(count.max(1).to_string().len().max(2) as u16 + 4).min(inner.width.saturating_sub(1));
	Rect::new(
		inner.x + gutter,
		inner.y,
		inner.width - gutter,
		inner.height,
	)
}

fn wrap_button(frame: &mut Frame, body: Rect, enabled: bool) -> Rect {
	if body.width < 16 || body.height == 0 {
		return Rect::default();
	}
	let area = Rect::new(body.right() - 12, body.y, 11, 1);
	frame.render_widget(
		Paragraph::new(if enabled { "[Wrap: on]" } else { "[Wrap: off]" })
			.style(Style::default().fg(Color::Cyan)),
		area,
	);
	area
}

struct View<'a> {
	inner: Rect,
	text: Rect,
	lines: &'a [String],
	layout: &'a Layout,
	top: usize,
	left: usize,
	active_line: usize,
}

fn draw_lines(frame: &mut Frame, view: View<'_>, style_at: impl Fn(Position) -> Style) {
	let View {
		inner,
		text,
		lines,
		layout,
		top,
		left,
		active_line,
	} = view;
	let gutter_style = Style::default()
		.fg(Color::Rgb(136, 150, 179))
		.bg(Color::Rgb(23, 28, 44));
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
			let gutter_style = if row.line == active_line {
				Style::default()
					.fg(Color::Rgb(220, 232, 255))
					.bg(Color::Rgb(41, 54, 83))
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
			let display = if source == "\t" || start != cell.x || end != cell.x + cell.width {
				" ".repeat(end - start)
			} else if source.width() > cell.width {
				"�".into()
			} else if source.width() == 0 {
				format!("◌{source}")
			} else {
				source.to_owned()
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

pub fn draw(frame: &mut Frame, app: &mut App) {
	let area = frame.area();
	frame.render_widget(Clear, area);
	app.tool_areas.clear();
	app.code_area = Rect::default();
	app.code_pane = Rect::default();
	app.editor_pane = Rect::default();
	app.code_toggle = Rect::default();
	app.editor_wrap_toggle = Rect::default();
	app.code_wrap_toggle = Rect::default();
	if area.width < 20 || area.height < 5 {
		app.editor_area = Rect::default();
		app.editor.resize(0, 0);
		frame.render_widget(
			Paragraph::new(if app.overlay == Overlay::Quit {
				"Discard? Y/N"
			} else {
				"Resize · Ctrl+Q quit"
			}),
			area,
		);
		return;
	}
	frame.render_widget(
		Paragraph::new(" Terminal UX Designer · F2 code · F3 wrap · F6 focus")
			.style(Style::default().fg(Color::Cyan)),
		Rect::new(area.x, area.y, area.width, 1),
	);
	app.code_toggle = Rect::new(area.right() - 11, area.y, 11, 1);
	frame.render_widget(
		Paragraph::new(if app.code_open {
			"[Code: on]"
		} else {
			"[Code: off]"
		}),
		app.code_toggle,
	);
	let tools_width = if area.width >= 65 { 22 } else { 0 };
	let code_height = if app.code_open && app.focus == 2 && area.height < 10 {
		area.height - 2
	} else if app.code_open && area.height >= 10 {
		(area.height / 3).max(4)
	} else {
		0
	};
	let body = Rect::new(
		area.x,
		area.y + 1,
		area.width - tools_width,
		area.height - 2 - code_height,
	);
	if code_height > 0 {
		let code_rect = Rect::new(body.x, body.bottom(), body.width, code_height);
		let block = Block::bordered().title(if app.focus == 2 {
			" Code * · Ctrl+C copy "
		} else {
			" Code · escaped ANSI "
		});
		let inner = block.inner(code_rect);
		app.code_pane = inner;
		frame.render_widget(block, code_rect);
		app.code_wrap_toggle = wrap_button(frame, code_rect, app.code_wrap);
		let lines: Vec<String> = crate::styling::code(&app.editor.document)
			.split('\n')
			.map(str::to_owned)
			.collect();
		app.code_area = text_area(inner, lines.len());
		let layout = Layout::new(&lines, app.code_area.width as usize, app.code_wrap, false);
		app.code_top = app.code_top.min(
			layout
				.rows
				.len()
				.saturating_sub(app.code_area.height as usize),
		);
		app.code_left = if app.code_wrap {
			0
		} else {
			app.code_left.min(
				layout
					.rows
					.iter()
					.map(|row| row.width)
					.max()
					.unwrap_or(0)
					.saturating_sub(app.code_area.width as usize),
			)
		};
		draw_lines(
			frame,
			View {
				inner,
				text: app.code_area,
				lines: &lines,
				layout: &layout,
				top: app.code_top,
				left: app.code_left,
				active_line: app.editor.cursor.row,
			},
			|_| Style::default(),
		);
	}
	let border = Block::bordered().title(if app.focus == 0 {
		" Editor · untitled "
	} else {
		" Editor "
	});
	let inner = border.inner(body);
	frame.render_widget(border, body);
	app.editor_wrap_toggle = wrap_button(frame, body, app.editor.wrap);
	app.editor_pane = inner;
	app.editor_area = text_area(inner, app.editor.document.lines.len());
	app.editor.resize(
		app.editor_area.width as usize,
		app.editor_area.height as usize,
	);
	let editor = &app.editor;
	let layout = editor.layout();
	draw_lines(
		frame,
		View {
			inner,
			text: app.editor_area,
			lines: &editor.document.lines,
			layout: &layout,
			top: editor.top,
			left: editor.left,
			active_line: editor.cursor.row,
		},
		|position| {
			let style = editor.document.style_at(position).render();
			if editor.selected(position) {
				style.bg(Color::DarkGray)
			} else {
				style
			}
		},
	);
	let (cursor_row, cursor_x) = layout.location(editor.cursor);
	if app.overlay == Overlay::None
		&& app.focus == 0
		&& cursor_row >= editor.top
		&& cursor_row < editor.top + app.editor_area.height as usize
		&& cursor_x >= editor.left
		&& cursor_x < editor.left + app.editor_area.width as usize
	{
		frame.set_cursor_position((
			app.editor_area.x + (cursor_x - editor.left) as u16,
			app.editor_area.y + (cursor_row - editor.top) as u16,
		));
	}
	let status = app.notice.clone().unwrap_or_else(|| {
		format!(
			" Ln {}, Col {} · {} lines · Ctrl+Z/Y undo/redo · F1 help · Ctrl+Q quit",
			editor.cursor.row + 1,
			column_at(
				&editor.document.lines[editor.cursor.row],
				editor.cursor.byte
			) + 1,
			editor.document.lines.len()
		)
	});
	frame.render_widget(
		Paragraph::new(status).style(Style::default().fg(Color::Gray)),
		Rect::new(area.x, area.bottom() - 1, area.width, 1),
	);
	if tools_width > 0 || app.focus == 1 {
		let tool_rect = if tools_width > 0 {
			Rect::new(body.right(), body.y, tools_width, area.height - 2)
		} else {
			body
		};
		frame.render_widget(Clear, tool_rect);
		let block = Block::bordered().title(if app.focus == 1 {
			" Tools * · Esc back "
		} else {
			" Tools · F6 "
		});
		let inside = block.inner(tool_rect);
		frame.render_widget(block, tool_rect);
		let top = if app.focus == 1 {
			app.tool_index
				.saturating_sub(inside.height.saturating_sub(1) as usize)
		} else {
			0
		};
		for (row, index) in (top..12).take(inside.height as usize).enumerate() {
			let state = match app.editor.tool_state(index) {
				Some(true) => "x",
				Some(false) => " ",
				None => "-",
			};
			let text = format!(
				"{} [{}] {}",
				if app.focus == 1 && app.tool_index == index {
					">"
				} else {
					" "
				},
				state,
				crate::styling::LABELS[index]
			);
			let rect = Rect::new(inside.x, inside.y + row as u16, inside.width, 1);
			frame.render_widget(Paragraph::new(text), rect);
			app.tool_areas.push((rect, index));
		}
		if tools_width == 0 {
			app.editor_pane = Rect::default();
			app.editor_area = Rect::default();
		}
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
				"Arrows · Home/End · Page Up/Down\nCtrl+Home/End: document start/end\nCtrl+Z: undo · Ctrl+Y: redo\nTab: literal tab (4-column stops)\nClick: cursor · Wheel: scroll\nCtrl+Q: quit (Y/N if text exists)\n\nText is session-local. No saving yet.\nShift+arrows / drag: select · Esc: clear\nCtrl+B/I/U: styles · F6: focus · F2: code\nF3: wrap focused pane · Ctrl+C: copy code\nCtrl/Alt+arrows: words · +Backspace: delete",
				17,
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
	fn active_gutters_follow_the_editor_cursor_in_both_panes() {
		let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
		let mut app = App::default();
		app.editor
			.insert("first\nsecond\nthird", EditKind::Atomic, Instant::now());
		app.code_open = true;
		app.editor.navigate(crate::editor::Movement::Start, false);
		let original = crate::styling::clipboard_text(&app.editor.document);
		for active in 0..3 {
			app.focus = active; // The indicator stays synchronized even outside editor focus.
			terminal.draw(|frame| draw(frame, &mut app)).unwrap();
			let buffer = terminal.backend().buffer();
			for pane in [app.editor_pane, app.code_pane] {
				for line in 0..3 {
					for x in pane.x..pane.x + 6 {
						let cell = &buffer[(x, pane.y + line as u16)];
						assert_eq!(
							cell.bg,
							if line == active {
								Color::Rgb(41, 54, 83)
							} else {
								Color::Rgb(23, 28, 44)
							}
						);
						assert_eq!(cell.modifier.contains(Modifier::BOLD), line == active);
					}
					assert!(
						!buffer[(pane.x + 6, pane.y + line as u16)]
							.modifier
							.contains(Modifier::BOLD)
					);
				}
			}
			assert_eq!(
				crate::styling::clipboard_text(&app.editor.document),
				original
			);
			app.editor.navigate(crate::editor::Movement::Down, true);
		}
	}

	#[test]
	fn active_gutter_covers_wrapped_continuations_after_scrolling() {
		let mut terminal = Terminal::new(TestBackend::new(40, 24)).unwrap();
		let mut app = App::default();
		app.editor.insert(
			&format!("{}\n{}", "a".repeat(80), vec!["next"; 20].join("\n")),
			EditKind::Atomic,
			Instant::now(),
		);
		app.editor.navigate(crate::editor::Movement::Start, false);
		app.editor.toggle_wrap();
		app.code_open = true;
		app.code_wrap = true;
		terminal.draw(|frame| draw(frame, &mut app)).unwrap();
		app.editor.top = 1;
		app.code_top = 1;
		terminal.draw(|frame| draw(frame, &mut app)).unwrap();
		let buffer = terminal.backend().buffer();
		for pane in [app.editor_pane, app.code_pane] {
			for x in pane.x..pane.x + 6 {
				let cell = &buffer[(x, pane.y)];
				assert_eq!(cell.symbol(), " ");
				assert_eq!(cell.bg, Color::Rgb(41, 54, 83));
				assert!(cell.modifier.contains(Modifier::BOLD));
			}
		}
	}

	#[test]
	fn gutters_number_logical_lines_and_stay_blank_on_continuations() {
		let mut terminal = Terminal::new(TestBackend::new(40, 20)).unwrap();
		let mut app = App::default();
		app.editor.insert(
			&format!("{}\nnext", "a".repeat(40)),
			EditKind::Atomic,
			Instant::now(),
		);
		app.editor.navigate(crate::editor::Movement::Start, false);
		app.editor.toggle_wrap();
		app.code_open = true;
		app.code_wrap = true;
		terminal.draw(|frame| draw(frame, &mut app)).unwrap();
		let buffer = terminal.backend().buffer();
		assert_eq!(buffer[(4, 2)].symbol(), "1");
		assert_eq!(buffer[(4, 3)].symbol(), " ");
		assert_eq!(buffer[(4, 4)].symbol(), "2");
		let code_y = app.code_area.y;
		assert_eq!(buffer[(4, code_y)].symbol(), "1");
		assert_eq!(buffer[(4, code_y + 1)].symbol(), " ");
		assert_eq!(buffer[(4, code_y + 2)].symbol(), "2");
		assert_eq!(buffer[(7, 3)].symbol(), "a");
		assert_eq!(
			(1..7).map(|x| buffer[(x, 2)].symbol()).collect::<String>(),
			"   1  "
		);
		assert_eq!(
			buffer[(1, app.editor_area.bottom() - 1)].bg,
			Color::Rgb(23, 28, 44)
		);
	}

	#[test]
	fn gutters_resize_for_three_digit_line_numbers() {
		let mut terminal = Terminal::new(TestBackend::new(40, 10)).unwrap();
		let mut app = App::default();
		app.editor
			.insert(&vec!["x"; 100].join("\n"), EditKind::Atomic, Instant::now());
		terminal.draw(|frame| draw(frame, &mut app)).unwrap();
		assert_eq!(app.editor_area.x, 8);
		let row = app.editor_area.bottom() - 1;
		let buffer = terminal.backend().buffer();
		assert_eq!(buffer[(3, row)].symbol(), "1");
		assert_eq!(buffer[(4, row)].symbol(), "0");
		assert_eq!(buffer[(5, row)].symbol(), "0");
	}

	#[test]
	fn selected_text_renders_document_attributes_and_selection_overlay() {
		let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
		let mut app = App::default();
		app.editor
			.insert("Hello\nworld", EditKind::Atomic, Instant::now());
		app.editor.navigate(crate::editor::Movement::Start, false);
		for _ in 0..5 {
			app.editor.navigate(crate::editor::Movement::Right, true);
		}
		app.editor.apply_tool(0);
		terminal.draw(|frame| draw(frame, &mut app)).unwrap();
		assert!(
			terminal.backend().buffer()[(7, 2)]
				.modifier
				.contains(ratatui::style::Modifier::BOLD)
		);
		assert_eq!(terminal.backend().buffer()[(7, 2)].bg, Color::DarkGray);
	}

	#[test]
	fn renders_unicode_and_tabs_at_shared_display_coordinates() {
		let mut terminal = Terminal::new(TestBackend::new(40, 10)).unwrap();
		let mut app = App::default();
		app.editor
			.insert("a\t界e\u{301}", EditKind::Atomic, Instant::now());
		terminal.draw(|frame| draw(frame, &mut app)).unwrap();
		let buffer = terminal.backend().buffer();
		assert_eq!(buffer[(7, 2)].symbol(), "a");
		assert_eq!(buffer[(11, 2)].symbol(), "界");
		assert_eq!(buffer[(13, 2)].symbol(), "e\u{301}");
		assert_eq!(app.editor_area, Rect::new(7, 2, 32, 6));
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
