use crate::scrollbar::{self, Bar, Target};
use ratatui::{
	Frame,
	layout::Rect,
	style::{Color, Modifier, Style},
	widgets::{Block, Clear, Paragraph},
};
use unicode_width::UnicodeWidthStr;

use crate::{
	app::App,
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

fn shortcut_hint(app: &App, action: crate::shortcuts::ActionId, label: &str) -> String {
	let key = app.shortcuts.config.hint(action);
	if key.is_empty() {
		String::new()
	} else {
		format!(" · {key} {label}")
	}
}

pub fn draw(frame: &mut Frame, app: &mut App) {
	let area = frame.area();
	frame.render_widget(Clear, area);
	app.tool_areas.clear();
	app.pane_toggles = [Rect::default(); 2];
	app.slim_tabs.clear();
	let slim = area.width <= 64;
	if app.slim != slim {
		app.slim = slim;
		app.panel.top = 0;
		app.tool_index = 0;
		if slim && app.focus == 1 {
			app.select_slim_tab(1);
		} else if slim && app.focus == 2 {
			app.select_slim_tab(0);
		}
	}
	app.panel.active_section = if slim {
		app.slim_tab.checked_sub(1)
	} else {
		None
	};
	app.scrollbars.clear();
	app.panel.area = Rect::default();
	app.panel.indexed_area = Rect::default();
	app.panel.gauge_areas.clear();
	app.code_area = Rect::default();
	app.code_pane = Rect::default();
	app.editor_pane = Rect::default();
	app.code_toggle = Rect::default();
	app.editor_wrap_toggle = Rect::default();
	app.code_wrap_toggle = Rect::default();
	if area.width < 20 || area.height < 5 {
		app.editor_area = Rect::default();
		app.editor.resize(0, 0);
		frame.render_widget(Paragraph::new("Resize · F1 shortcuts"), area);
		return;
	}
	frame.render_widget(
		Paragraph::new(format!(
			" Terminal UX Designer{}{}{}{}",
			shortcut_hint(app, crate::shortcuts::ActionId::ToggleCode, "code"),
			shortcut_hint(
				app,
				if app.focus == 2 {
					crate::shortcuts::ActionId::CodeWrap
				} else {
					crate::shortcuts::ActionId::EditorWrap
				},
				"wrap"
			),
			shortcut_hint(app, crate::shortcuts::ActionId::FileMenu, "files"),
			shortcut_hint(app, crate::shortcuts::ActionId::CyclePanes, "focus")
		))
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
	let tools_width = if area.width >= 65 { 35 } else { 0 };
	let available = area.height - 2;
	let code_height = if !app.code_open {
		2.min(available)
	} else if !app.editor_open {
		available.saturating_sub(2)
	} else if slim && app.slim_tab > 0 {
		(area.height / 2).max(8).min(available.saturating_sub(3))
	} else {
		(area.height / 3).max(4).min(available.saturating_sub(2))
	};
	let body = Rect::new(
		area.x,
		area.y + 1,
		area.width - tools_width,
		area.height - 2 - code_height,
	);
	if code_height > 0 {
		let code_rect = Rect::new(body.x, body.bottom(), body.width, code_height);
		let block = Block::bordered()
			.border_type(ratatui::widgets::BorderType::Rounded)
			.title(if slim && app.code_open {
				" ▾ "
			} else if slim {
				" ▸ "
			} else if app.code_open {
				" ▾ Code "
			} else {
				" ▸ Code "
			});
		let mut inner = block.inner(code_rect);
		app.code_pane = inner;
		frame.render_widget(block, code_rect);
		app.pane_toggles[1] = Rect::new(
			code_rect.x + 1,
			code_rect.y,
			(if slim { 3 } else { 8 }).min(code_rect.width.saturating_sub(1)),
			1,
		);
		if slim {
			let labels = if code_rect.width < 40 {
				["Code", "Style", "Color", "Chars"]
			} else {
				["Code", "Style", "Color", "Characters"]
			};
			let end = if app.code_open && app.slim_tab == 0 && code_rect.width >= 53 {
				code_rect.right() - 13
			} else {
				code_rect.right().saturating_sub(1)
			};
			let start = if code_rect.width < 35 {
				app.slim_tab.saturating_sub(1)
			} else {
				0
			};
			let mut x = code_rect.x + 4;
			for (i, label) in labels.iter().enumerate().skip(start) {
				let width = label.len() as u16;
				if x + width > end {
					break;
				}
				let rect = Rect::new(x, code_rect.y, width, 1);
				frame.render_widget(
					Paragraph::new(*label).style(
						Style::default()
							.fg(Color::Cyan)
							.bg(if app.slim_tab == i {
								Color::Rgb(41, 54, 83)
							} else {
								Color::Reset
							})
							.add_modifier(if app.slim_tab == i {
								Modifier::BOLD
							} else {
								Modifier::empty()
							}),
					),
					rect,
				);
				app.slim_tabs.push((rect, i));
				x += width;
				if x < end {
					frame.render_widget(Paragraph::new(" "), Rect::new(x, code_rect.y, 1, 1));
				}
				if i < 3 && x + 3 + labels[i + 1].len() as u16 <= end {
					frame.render_widget(Paragraph::new(" · "), Rect::new(x, code_rect.y, 3, 1));
					x += 3;
				} else {
					break;
				}
			}
		}

		if slim && app.slim_tab > 0 {
			inner = Rect::default();
		}
		app.code_pane = inner;

		app.code_wrap_toggle =
			if app.code_open && (!slim || (app.slim_tab == 0 && code_rect.width >= 53)) {
				wrap_button(frame, code_rect, app.code_wrap)
			} else {
				Rect::default()
			};
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
		scrollbar::draw(
			frame,
			&mut app.scrollbars,
			Bar {
				target: Target::CodeVertical,
				area: Rect::new(
					code_rect.right().saturating_sub(1),
					inner.y,
					1,
					inner.height,
				),
				content: layout.rows.len(),
				viewport: app.code_area.height as usize,
				offset: app.code_top,
			},
		);
		if !app.code_wrap {
			scrollbar::draw(
				frame,
				&mut app.scrollbars,
				Bar {
					target: Target::CodeHorizontal,
					area: Rect::new(
						app.code_area.x,
						code_rect.bottom().saturating_sub(1),
						app.code_area.width,
						1,
					),
					content: layout.rows.iter().map(|row| row.width).max().unwrap_or(0),
					viewport: app.code_area.width as usize,
					offset: app.code_left,
				},
			);
		}
	}
	let name = app
		.document_path
		.as_ref()
		.and_then(|p| p.file_name())
		.map(|s| crate::document::sanitize(&s.to_string_lossy()))
		.unwrap_or_else(|| "untitled".into());
	let border = Block::bordered()
		.border_type(ratatui::widgets::BorderType::Rounded)
		.title(format!(
			" {} Editor · {name}{} ",
			if app.editor_open { "▾" } else { "▸" },
			if app.dirty() { " *" } else { "" }
		));
	let inner = border.inner(body);
	frame.render_widget(border, body);
	app.pane_toggles[0] = Rect::new(body.x + 1, body.y, 10.min(body.width.saturating_sub(1)), 1);
	app.editor_wrap_toggle = if app.editor_open {
		wrap_button(frame, body, app.editor.wrap)
	} else {
		Rect::default()
	};
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
	scrollbar::draw(
		frame,
		&mut app.scrollbars,
		Bar {
			target: Target::EditorVertical,
			area: Rect::new(body.right().saturating_sub(1), inner.y, 1, inner.height),
			content: layout.rows.len(),
			viewport: app.editor_area.height as usize,
			offset: editor.top,
		},
	);
	if !editor.wrap {
		scrollbar::draw(
			frame,
			&mut app.scrollbars,
			Bar {
				target: Target::EditorHorizontal,
				area: Rect::new(
					app.editor_area.x,
					body.bottom().saturating_sub(1),
					app.editor_area.width,
					1,
				),
				content: layout.rows.iter().map(|row| row.width).max().unwrap_or(0) + 1,
				viewport: app.editor_area.width as usize,
				offset: editor.left,
			},
		);
	}

	let (cursor_row, cursor_x) = layout.location(editor.cursor);
	if app.settings.is_none()
		&& app.focus == 0
		&& app.files.dialog.is_none()
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
	let status = app
		.shortcuts
		.pending_label()
		.or_else(|| app.notice.clone())
		.unwrap_or_else(|| {
			format!(
				" Ln {}, Col {} · {} lines{}{} · F1 shortcuts{}",
				editor.cursor.row + 1,
				column_at(
					&editor.document.lines[editor.cursor.row],
					editor.cursor.byte
				) + 1,
				editor.document.lines.len(),
				shortcut_hint(app, crate::shortcuts::ActionId::Undo, "undo"),
				shortcut_hint(app, crate::shortcuts::ActionId::Redo, "redo"),
				shortcut_hint(app, crate::shortcuts::ActionId::Quit, "quit"),
			)
		});
	frame.render_widget(
		Paragraph::new(status).style(Style::default().fg(Color::Gray)),
		Rect::new(area.x, area.bottom() - 1, area.width, 1),
	);
	if tools_width > 0 {
		let tool_rect = Rect::new(body.right(), body.y, tools_width, area.height - 2);
		frame.render_widget(Clear, tool_rect);
		crate::panel_ui::draw(frame, app, tool_rect);
	} else if app.code_open && app.slim_tab > 0 {
		let tool_rect = Rect::new(
			body.x + 1,
			body.bottom() + 1,
			body.width.saturating_sub(2),
			code_height.saturating_sub(2),
		);
		frame.render_widget(Clear, tool_rect);
		crate::panel_ui::draw(frame, app, tool_rect);
		app.code_area = Rect::default();
		app.code_pane = Rect::default();
		app.code_wrap_toggle = Rect::default();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::editor::EditKind;
	use crate::panel::Control;
	use crate::styling::Action;
	use ratatui::{Terminal, backend::TestBackend};
	use std::time::Instant;

	#[test]
	fn rgb_gauges_stage_drag_and_keyboard_changes_until_apply() {
		use crate::styling::Ink;
		use crossterm::event::{
			Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
		};
		let mut app = App::default();
		app.panel.palette = 2;
		app.editor.insert("color", EditKind::Atomic, Instant::now());
		app.editor.navigate(crate::editor::Movement::Home, true);
		let original = app.editor.document.clone();
		let selection = app.editor.selection();
		let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
		terminal.draw(|frame| draw(frame, &mut app)).unwrap();
		let (red, _) = app.panel.gauge_areas[0];
		for (kind, x) in [
			(MouseEventKind::Down(MouseButton::Left), red.x),
			(MouseEventKind::Drag(MouseButton::Left), red.right() + 4),
			(MouseEventKind::Up(MouseButton::Left), red.right() + 4),
		] {
			app.handle(Event::Mouse(MouseEvent {
				kind,
				column: x,
				row: red.y,
				modifiers: KeyModifiers::NONE,
			}));
		}
		assert_eq!(app.panel.rgb(), Ok(Ink::Rgb(255, 192, 255)));
		app.handle(Event::Key(KeyEvent::new(
			KeyCode::Left,
			KeyModifiers::SHIFT,
		)));
		assert_eq!(app.panel.fields[1], "245");
		assert_eq!(app.panel.fields[0], "#f5c0ff");
		assert_eq!(app.editor.document, original);
		assert_eq!(app.editor.selection(), selection);
		app.handle(Event::Key(KeyEvent::new(
			KeyCode::Enter,
			KeyModifiers::NONE,
		)));
		assert_eq!(
			app.editor.document.styles.at(0).foreground,
			Ink::Rgb(245, 192, 255)
		);
		app.editor.undo(false);
		assert_eq!(app.editor.document, original);
	}

	#[test]
	fn scrollbar_drag_reaches_all_viewport_ends_without_editing() {
		use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
		for target in [
			Target::EditorVertical,
			Target::EditorHorizontal,
			Target::CodeVertical,
			Target::CodeHorizontal,
			Target::Tools,
			Target::Indexed,
		] {
			let mut app = App::default();
			app.editor.wrap = false;
			app.code_wrap = false;
			app.panel.palette = 1;
			app.editor.insert(
				&vec!["x".repeat(160); 80].join("\n"),
				EditKind::Atomic,
				Instant::now(),
			);
			app.editor.navigate(crate::editor::Movement::Start, false);
			app.editor.navigate(crate::editor::Movement::Right, true);
			let original = app.editor.document.clone();
			let selection = app.editor.selection();
			let cursor = app.editor.cursor;
			let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
			terminal.draw(|frame| draw(frame, &mut app)).unwrap();
			let bar = *app
				.scrollbars
				.iter()
				.find(|bar| bar.target == target)
				.unwrap();
			for (kind, column, row) in [
				(
					MouseEventKind::Down(MouseButton::Left),
					bar.area.x,
					bar.area.y,
				),
				(
					MouseEventKind::Drag(MouseButton::Left),
					bar.area.right() - 1,
					bar.area.bottom() - 1,
				),
				(
					MouseEventKind::Up(MouseButton::Left),
					bar.area.right() - 1,
					bar.area.bottom() - 1,
				),
			] {
				app.handle(Event::Mouse(MouseEvent {
					kind,
					column,
					row,
					modifiers: KeyModifiers::NONE,
				}));
				terminal.draw(|frame| draw(frame, &mut app)).unwrap();
			}
			let bar = app
				.scrollbars
				.iter()
				.find(|bar| bar.target == target)
				.unwrap();
			assert_eq!(bar.offset, bar.content - bar.viewport, "{target:?}");
			assert_eq!(app.editor.document, original);
			assert_eq!(app.editor.selection(), selection);
			assert_eq!(app.editor.cursor, cursor);
			app.editor.undo(false);
			assert!(app.editor.document.is_empty());
		}
	}

	#[test]
	fn indexed_palette_scrolls_independently_and_reaches_last_partial_row() {
		use crate::styling::{Action, Ink};
		use crossterm::event::{Event, KeyModifiers, MouseEvent, MouseEventKind};
		let mut app = App::default();
		app.panel.palette = 1;
		let mut terminal = Terminal::new(TestBackend::new(100, 45)).unwrap();
		terminal.draw(|frame| draw(frame, &mut app)).unwrap();
		let sections = app.panel.layout().1;
		let grid = app.panel.indexed_area;
		let original = app.editor.document.clone();
		for _ in 0..40 {
			app.handle(Event::Mouse(MouseEvent {
				kind: MouseEventKind::ScrollDown,
				column: grid.x,
				row: grid.y,
				modifiers: KeyModifiers::NONE,
			}));
			terminal.draw(|frame| draw(frame, &mut app)).unwrap();
		}
		assert_eq!(app.panel.indexed_top, 32);
		assert_eq!(app.panel.top, 0);
		assert_eq!(app.panel.layout().1, sections);
		let items = app.panel.items();
		let last = items
			.iter()
			.position(|item| item.control == Control::Style(Action::Foreground(Ink::Indexed(255))))
			.unwrap();
		assert!(app.tool_areas.iter().any(|(_, index)| *index == last));
		assert_eq!(
			app.tool_areas
				.iter()
				.filter(|(_, index)| matches!(items[*index].ink, Some(Ink::Indexed(_))))
				.count(),
			32
		);
		app.tool_index = items
			.iter()
			.position(|item| item.ink == Some(Ink::Indexed(0)))
			.unwrap();
		app.reveal_tool();
		assert_eq!(app.panel.indexed_top, 0);
		assert_eq!(app.editor.document, original);
	}

	#[test]
	fn keyboard_reaches_control_groups_and_grid_members_in_all_layouts() {
		use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
		for (width, height) in [(80, 24), (64, 24), (40, 12), (20, 12)] {
			for palette in 0..3 {
				let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
				let mut app = App::default();
				app.panel.palette = palette;
				terminal.draw(|frame| draw(frame, &mut app)).unwrap();
				let tabs = if width <= 64 { vec![1, 2, 3] } else { vec![0] };
				for tab in tabs {
					if width <= 64 {
						app.select_slim_tab(tab);
						app.editor_open = false;
					} else {
						app.focus = 1;
					}
					app.tool_index = 0;
					let items = app.panel.items();
					let groups: std::collections::BTreeSet<_> =
						items.iter().map(|item| item.group()).collect();
					let mut visited = std::collections::BTreeSet::new();
					for _ in 0..groups.len() {
						terminal.draw(|frame| draw(frame, &mut app)).unwrap();
						let group = app.panel.items()[app.tool_index].group();
						visited.insert(group);
						assert!(
							app.tool_areas
								.iter()
								.any(|(rect, index)| *index == app.tool_index && !rect.is_empty()),
							"unreachable group {group:?} at {width}x{height}"
						);
						app.handle(Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)));
					}
					assert_eq!(visited, groups);
					// A grid is one field; internal arrows can reach the last partial row.
					if let Some(index) = app.panel.items().iter().position(|item| {
						matches!(
							item.group(),
							crate::panel::Group::Colors(_) | crate::panel::Group::Glyphs(_)
						)
					}) {
						app.tool_index = index;
						let group = app.panel.items()[index].group();
						let last = app
							.panel
							.items()
							.iter()
							.rposition(|item| item.group() == group)
							.unwrap();
						app.handle(Event::Key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE)));
						terminal.draw(|frame| draw(frame, &mut app)).unwrap();
						assert_eq!(app.tool_index, last);
						assert!(app.tool_areas.iter().any(|(_, index)| *index == last));
					}
				}
				assert!(app.editor.document.is_empty());
			}
		}
	}

	#[test]
	fn collapsed_panes_reallocate_space_and_slim_layout_tracks_resize() {
		let mut app = App::default();
		let mut terminal = Terminal::new(TestBackend::new(64, 30)).unwrap();
		terminal.draw(|frame| draw(frame, &mut app)).unwrap();
		assert!(app.slim);
		assert_eq!(app.slim_tabs.len(), 4);
		let initial = app.code_area.height;
		app.toggle_pane(false);
		terminal.draw(|frame| draw(frame, &mut app)).unwrap();
		assert_eq!(app.editor_area.height, 0);
		assert!(app.code_area.height > initial);
		app.toggle_pane(true);
		terminal.draw(|frame| draw(frame, &mut app)).unwrap();
		assert!(app.editor_open);
		assert_eq!(app.code_area.height, 0);
		app.select_slim_tab(2);
		terminal.draw(|frame| draw(frame, &mut app)).unwrap();
		assert!(!app.tool_areas.is_empty());
		assert!(app.editor_area.height > 0);
		let mut wide = Terminal::new(TestBackend::new(65, 30)).unwrap();
		wide.draw(|frame| draw(frame, &mut app)).unwrap();
		assert!(!app.slim);
		assert!(app.panel.active_section.is_none());
		assert!(app.slim_tabs.is_empty());
	}

	#[test]
	fn mouse_swatch_hit_areas_apply_both_channels_and_clear_is_undoable() {
		use crate::styling::Ink;
		use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
		let mut app = App::default();
		let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
		app.editor.insert("ab", EditKind::Atomic, Instant::now());
		app.editor.navigate(crate::editor::Movement::Home, true);
		for control in [
			Control::Style(Action::Foreground(Ink::Ansi(15))),
			Control::Target(true),
			Control::Style(Action::Background(Ink::Ansi(1))),
			Control::Style(Action::Dim),
			Control::Style(Action::Strike),
		] {
			terminal.draw(|frame| draw(frame, &mut app)).unwrap();
			let index = app
				.panel
				.items()
				.iter()
				.position(|item| item.control == control)
				.unwrap();
			app.tool_index = index;
			app.reveal_tool();
			terminal.draw(|frame| draw(frame, &mut app)).unwrap();
			let (rect, _) = app
				.tool_areas
				.iter()
				.find(|(_, tool)| *tool == index)
				.unwrap();
			app.handle(Event::Mouse(MouseEvent {
				kind: MouseEventKind::Down(MouseButton::Left),
				column: rect.x,
				row: rect.y,
				modifiers: KeyModifiers::NONE,
			}));
			assert!(app.editor.selection().is_some());
		}
		let style = app.editor.document.styles.at(0);
		assert_eq!(style.foreground, Ink::Ansi(15));
		assert_eq!(style.background, Ink::Ansi(1));
		assert!(style.dim && style.strike);
		let original = app.editor.document.clone();
		app.editor.apply_tool(Action::Clear);
		assert_eq!(app.editor.document.styles.at(0), Default::default());
		app.editor.undo(false);
		assert_eq!(app.editor.document, original);
	}

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
		app.editor.wrap = true;
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
		app.editor.wrap = true;
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
		app.editor.apply_tool(Action::Bold);
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
		assert_eq!(app.editor_area, Rect::new(7, 2, 32, 2));
	}

	#[test]
	fn tiny_sizes_and_overlays_never_lose_the_document() {
		let mut app = App::default();
		app.editor
			.insert("keep me", EditKind::Atomic, Instant::now());
		for (width, height) in [(1, 1), (12, 3), (20, 5), (50, 15), (120, 30)] {
			let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
			for settings in [false, true] {
				app.settings = settings.then(|| {
					crate::shortcuts::settings::Settings::new(
						&app.shortcuts.config,
						crate::shortcuts::SettingsSection::Editor,
					)
				});
				terminal.draw(|frame| draw(frame, &mut app)).unwrap();
				assert_eq!(app.editor.document.lines, ["keep me"]);
			}
		}
	}
}
