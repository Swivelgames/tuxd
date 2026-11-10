use crate::libraries::ansi::Action;
use crate::libraries::editing::EditKind;
use crate::panel::Control;
use crate::workspace::{Pane, TextPane};
use crate::{app::App, theme};
use ratatui::{Terminal, backend::TestBackend};
use ratatui::{layout::Rect, style::Modifier};
use std::time::Instant;

#[test]
fn code_highlighting_is_view_only_and_tracks_horizontal_scroll() {
	let mut app = App::default();
	app.workspace.active.editor.insert(
		&mut app.workspace.active.session.editing,
		&format!("abc {}", "literal \\x1b[31m 界 👩🏽‍💻 ".repeat(4)),
		EditKind::Atomic,
		Instant::now(),
	);
	let document = app.workspace.active.session.editing.document.clone();
	let raw = document.to_raw_ansi();
	let copied = document.to_escaped_ansi();
	let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
	terminal.draw(|f| app.draw(f)).unwrap();
	let area = app.workspace.active.code.surface.text;
	let buffer = terminal.backend().buffer();
	assert_eq!(buffer[(area.x, area.y)].fg, theme::syntax_sequence_color());
	assert_eq!(
		buffer[(area.x + 5, area.y)].fg,
		theme::syntax_escape_color()
	);
	assert!(
		buffer[(area.x + 5, area.y)]
			.modifier
			.contains(Modifier::BOLD)
	);
	assert_eq!(buffer[(area.x + 7, area.y)].fg, theme::text_color());
	app.workspace.active.code.surface.viewport.wrap = false;
	app.workspace.active.code.surface.viewport.left = 5;
	terminal.draw(|f| app.draw(f)).unwrap();
	assert_eq!(
		terminal.backend().buffer()[(
			app.workspace.active.code.surface.text.x,
			app.workspace.active.code.surface.text.y
		)]
			.fg,
		theme::syntax_escape_color()
	);
	assert_eq!(app.workspace.active.session.editing.document, document);
	assert_eq!(
		app.workspace.active.session.editing.document.to_raw_ansi(),
		raw
	);
	assert_eq!(
		app.workspace
			.active
			.session
			.editing
			.document
			.to_escaped_ansi(),
		copied
	);
}

#[test]
fn rgb_gauges_stage_drag_and_keyboard_changes_until_apply() {
	use crate::libraries::ansi::Ink;
	use crossterm::event::{
		Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
	};
	let mut app = App::default();
	app.workspace.panel.color.palette = 2;
	app.workspace.active.editor.insert(
		&mut app.workspace.active.session.editing,
		"color",
		EditKind::Atomic,
		Instant::now(),
	);
	app.workspace.active.editor.navigate(
		&mut app.workspace.active.session.editing,
		crate::libraries::editing::Movement::Home,
		true,
	);
	let original = app.workspace.active.session.editing.document.clone();
	let selection = app.workspace.active.session.editing.selection();
	let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
	terminal.draw(|frame| app.draw(frame)).unwrap();
	let (red, _) = app.workspace.panel.color.gauge_areas[0];
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
	assert_eq!(
		app.workspace.panel.color.rgb.rgb(),
		Ok(Ink::Rgb(255, 192, 255))
	);
	app.handle(Event::Key(KeyEvent::new(
		KeyCode::Left,
		KeyModifiers::SHIFT,
	)));
	assert_eq!(app.workspace.panel.color.rgb.fields[1].text(), "245");
	assert_eq!(app.workspace.panel.color.rgb.fields[0].text(), "#f5c0ff");
	assert_eq!(app.workspace.active.session.editing.document, original);
	assert_eq!(app.workspace.active.session.editing.selection(), selection);
	app.handle(Event::Key(KeyEvent::new(
		KeyCode::Enter,
		KeyModifiers::NONE,
	)));
	assert_eq!(
		app.workspace
			.active
			.session
			.editing
			.document
			.style_at_offset(0)
			.foreground,
		Ink::Rgb(245, 192, 255)
	);
	app.workspace
		.active
		.editor
		.undo(&mut app.workspace.active.session.editing, false);
	assert_eq!(app.workspace.active.session.editing.document, original);
}

#[test]
fn scrollbar_drag_reaches_all_viewport_ends_without_editing() {
	use crate::components::scrollbar::{Axis, Bar};
	use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
	fn lookup(app: &App, target: usize) -> Bar {
		if target < 4 {
			let surface = if target < 2 {
				&app.workspace.active.editor.surface
			} else {
				&app.workspace.active.code.surface
			};
			let axis = if target.is_multiple_of(2) {
				Axis::Vertical
			} else {
				Axis::Horizontal
			};
			*surface.bars.iter().find(|bar| bar.target == axis).unwrap()
		} else {
			let target = if target == 4 {
				crate::panel::scroll::Target::Tools
			} else {
				crate::panel::scroll::Target::Indexed
			};
			let bar = app
				.workspace
				.panel
				.scrollbars
				.iter()
				.find(|bar| bar.target == target)
				.unwrap();
			Bar {
				target: Axis::Vertical,
				area: bar.area,
				content: bar.content,
				viewport: bar.viewport,
				offset: bar.offset,
			}
		}
	}
	for target in 0..6 {
		let mut app = App::default();
		app.workspace.active.editor.surface.viewport.wrap = false;
		app.workspace.active.code.surface.viewport.wrap = false;
		app.workspace.panel.color.palette = 1;
		app.workspace.active.editor.insert(
			&mut app.workspace.active.session.editing,
			&vec!["x".repeat(160); 80].join("\n"),
			EditKind::Atomic,
			Instant::now(),
		);
		app.workspace.active.editor.navigate(
			&mut app.workspace.active.session.editing,
			crate::libraries::editing::Movement::Start,
			false,
		);
		app.workspace.active.editor.navigate(
			&mut app.workspace.active.session.editing,
			crate::libraries::editing::Movement::Right,
			true,
		);
		let original = app.workspace.active.session.editing.document.clone();
		let selection = app.workspace.active.session.editing.selection();
		let cursor = app.workspace.active.session.editing.cursor;
		let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
		terminal.draw(|frame| app.draw(frame)).unwrap();
		let bar = lookup(&app, target);
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
			terminal.draw(|frame| app.draw(frame)).unwrap();
		}
		let bar = lookup(&app, target);
		assert_eq!(bar.offset, bar.content - bar.viewport, "{target:?}");
		assert_eq!(app.workspace.active.session.editing.document, original);
		assert_eq!(app.workspace.active.session.editing.selection(), selection);
		assert_eq!(app.workspace.active.session.editing.cursor, cursor);
		app.workspace
			.active
			.editor
			.undo(&mut app.workspace.active.session.editing, false);
		assert!(app.workspace.active.session.editing.document.is_empty());
	}
}

#[test]
fn indexed_palette_scrolls_independently_and_reaches_last_partial_row() {
	use crate::libraries::ansi::{Action, Ink};
	use crossterm::event::{Event, KeyModifiers, MouseEvent, MouseEventKind};
	let mut app = App::default();
	app.workspace.panel.color.palette = 1;
	let mut terminal = Terminal::new(TestBackend::new(100, 45)).unwrap();
	terminal.draw(|frame| app.draw(frame)).unwrap();
	let sections = app.workspace.panel.layout().1;
	let grid = app.workspace.panel.color.indexed_area;
	let original = app.workspace.active.session.editing.document.clone();
	for _ in 0..40 {
		app.handle(Event::Mouse(MouseEvent {
			kind: MouseEventKind::ScrollDown,
			column: grid.x,
			row: grid.y,
			modifiers: KeyModifiers::NONE,
		}));
		terminal.draw(|frame| app.draw(frame)).unwrap();
	}
	assert_eq!(app.workspace.panel.color.indexed_top, 32);
	assert_eq!(app.workspace.panel.top, 0);
	assert_eq!(app.workspace.panel.layout().1, sections);
	let items = app.workspace.panel.items();
	let last = items
		.iter()
		.position(|item| item.control == Control::Style(Action::Foreground(Ink::Indexed(255))))
		.unwrap();
	assert!(
		app.workspace
			.panel
			.tool_areas
			.iter()
			.any(|(_, index)| *index == last)
	);
	assert_eq!(
		app.workspace
			.panel
			.tool_areas
			.iter()
			.filter(|(_, index)| matches!(items[*index].ink, Some(Ink::Indexed(_))))
			.count(),
		32
	);
	app.workspace.panel.selected = items
		.iter()
		.position(|item| item.ink == Some(Ink::Indexed(0)))
		.unwrap();
	app.workspace.reveal_tool();
	assert_eq!(app.workspace.panel.color.indexed_top, 0);
	assert_eq!(app.workspace.active.session.editing.document, original);
}

#[test]
fn keyboard_reaches_control_groups_and_grid_members_in_all_layouts() {
	use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
	for (width, height) in [(80, 24), (64, 24), (40, 12), (20, 12)] {
		for palette in 0..3 {
			let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
			let mut app = App::default();
			app.workspace.panel.color.palette = palette;
			terminal.draw(|frame| app.draw(frame)).unwrap();
			let tabs = if width <= 64 { vec![1, 2, 3] } else { vec![0] };
			for tab in tabs {
				if width <= 64 {
					app.workspace.select_slim_tab(tab);
					app.workspace.editor_open = false;
				} else {
					app.workspace.focus = Pane::Tools;
				}
				app.workspace.panel.selected = 0;
				let items = app.workspace.panel.items();
				let groups: std::collections::BTreeSet<_> =
					items.iter().map(|item| item.group()).collect();
				let mut visited = std::collections::BTreeSet::new();
				for _ in 0..groups.len() {
					terminal.draw(|frame| app.draw(frame)).unwrap();
					let group = app.workspace.panel.items()[app.workspace.panel.selected].group();
					visited.insert(group);
					assert!(
						app.workspace
							.panel
							.tool_areas
							.iter()
							.any(|(rect, index)| *index == app.workspace.panel.selected
								&& !rect.is_empty()),
						"unreachable group {group:?} at {width}x{height}"
					);
					app.handle(Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)));
				}
				assert_eq!(visited, groups);
				// A grid is one field; internal arrows can reach the last partial row.
				if let Some(index) = app.workspace.panel.items().iter().position(|item| {
					matches!(
						item.group(),
						crate::panel::Group::Colors(_) | crate::panel::Group::Glyphs
					)
				}) {
					app.workspace.panel.selected = index;
					let group = app.workspace.panel.items()[index].group();
					let last = app
						.workspace
						.panel
						.items()
						.iter()
						.rposition(|item| item.group() == group)
						.unwrap();
					app.handle(Event::Key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE)));
					terminal.draw(|frame| app.draw(frame)).unwrap();
					let last = if group == crate::panel::Group::Glyphs {
						assert_eq!(
							app.workspace.panel.items()[app.workspace.panel.selected].control,
							app.workspace.panel.characters.selected_control().unwrap()
						);
						app.workspace.panel.selected
					} else {
						last
					};
					assert_eq!(app.workspace.panel.selected, last);
					assert!(
						app.workspace
							.panel
							.tool_areas
							.iter()
							.any(|(_, index)| *index == last)
					);
				}
			}
			assert!(app.workspace.active.session.editing.document.is_empty());
		}
	}
}

#[test]
fn collapsed_panes_reallocate_space_and_slim_layout_tracks_resize() {
	let mut app = App::default();
	let mut terminal = Terminal::new(TestBackend::new(64, 30)).unwrap();
	terminal.draw(|frame| app.draw(frame)).unwrap();
	assert!(app.workspace.slim);
	assert_eq!(app.workspace.slim_tabs.len(), 4);
	let initial = app.workspace.active.code.surface.text.height;
	app.workspace.toggle_pane(TextPane::Editor);
	terminal.draw(|frame| app.draw(frame)).unwrap();
	assert_eq!(app.workspace.active.editor.surface.text.height, 0);
	assert!(app.workspace.active.code.surface.text.height > initial);
	app.workspace.toggle_pane(TextPane::Code);
	terminal.draw(|frame| app.draw(frame)).unwrap();
	assert!(app.workspace.editor_open);
	assert_eq!(app.workspace.active.code.surface.text.height, 0);
	app.workspace.select_slim_tab(2);
	terminal.draw(|frame| app.draw(frame)).unwrap();
	assert!(!app.workspace.panel.tool_areas.is_empty());
	assert!(app.workspace.active.editor.surface.text.height > 0);
	let mut wide = Terminal::new(TestBackend::new(65, 30)).unwrap();
	wide.draw(|frame| app.draw(frame)).unwrap();
	assert!(!app.workspace.slim);
	assert!(app.workspace.panel.active_section.is_none());
	assert!(app.workspace.slim_tabs.is_empty());
}

#[test]
fn mouse_swatch_hit_areas_apply_both_channels_and_clear_is_undoable() {
	use crate::libraries::ansi::Ink;
	use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
	let mut app = App::default();
	let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
	app.workspace.active.editor.insert(
		&mut app.workspace.active.session.editing,
		"ab",
		EditKind::Atomic,
		Instant::now(),
	);
	app.workspace.active.editor.navigate(
		&mut app.workspace.active.session.editing,
		crate::libraries::editing::Movement::Home,
		true,
	);
	for control in [
		Control::Style(Action::Foreground(Ink::Ansi(15))),
		Control::Target(true),
		Control::Style(Action::Background(Ink::Ansi(1))),
		Control::Style(Action::Dim),
		Control::Style(Action::Strike),
	] {
		terminal.draw(|frame| app.draw(frame)).unwrap();
		let index = app
			.workspace
			.panel
			.items()
			.iter()
			.position(|item| item.control == control)
			.unwrap();
		app.workspace.panel.selected = index;
		app.workspace.reveal_tool();
		terminal.draw(|frame| app.draw(frame)).unwrap();
		let (rect, _) = app
			.workspace
			.panel
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
		assert!(app.workspace.active.session.editing.selection().is_some());
	}
	let style = app
		.workspace
		.active
		.session
		.editing
		.document
		.style_at_offset(0);
	assert_eq!(style.foreground, Ink::Ansi(15));
	assert_eq!(style.background, Ink::Ansi(1));
	assert!(style.dim && style.strike);
	let original = app.workspace.active.session.editing.document.clone();
	app.workspace
		.active
		.session
		.editing
		.apply_tool(Action::Clear);
	assert_eq!(
		app.workspace
			.active
			.session
			.editing
			.document
			.style_at_offset(0),
		Default::default()
	);
	app.workspace
		.active
		.editor
		.undo(&mut app.workspace.active.session.editing, false);
	assert_eq!(app.workspace.active.session.editing.document, original);
}

#[test]
fn active_gutters_follow_the_editor_cursor_in_both_panes() {
	let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
	let mut app = App::default();
	app.workspace.active.editor.insert(
		&mut app.workspace.active.session.editing,
		"first\nsecond\nthird",
		EditKind::Atomic,
		Instant::now(),
	);
	app.workspace.code_open = true;
	app.workspace.active.editor.navigate(
		&mut app.workspace.active.session.editing,
		crate::libraries::editing::Movement::Start,
		false,
	);
	let original = app
		.workspace
		.active
		.session
		.editing
		.document
		.to_escaped_ansi();
	for (active, focus) in [Pane::Editor, Pane::Tools, Pane::Code]
		.into_iter()
		.enumerate()
	{
		app.workspace.focus = focus; // The indicator stays synchronized even outside editor focus.
		terminal.draw(|frame| app.draw(frame)).unwrap();
		let buffer = terminal.backend().buffer();
		for pane in [
			app.workspace.active.editor.surface.inner,
			app.workspace.active.code.surface.inner,
		] {
			for line in 0..3 {
				for x in pane.x..pane.x + 6 {
					let cell = &buffer[(x, pane.y + line as u16)];
					assert_eq!(
						cell.bg,
						if line == active {
							theme::selected_color()
						} else {
							theme::field_color()
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
			app.workspace
				.active
				.session
				.editing
				.document
				.to_escaped_ansi(),
			original
		);
		app.workspace.active.editor.navigate(
			&mut app.workspace.active.session.editing,
			crate::libraries::editing::Movement::Down,
			true,
		);
	}
}

#[test]
fn active_gutter_covers_wrapped_continuations_after_scrolling() {
	let mut terminal = Terminal::new(TestBackend::new(40, 24)).unwrap();
	let mut app = App::default();
	app.workspace.active.editor.insert(
		&mut app.workspace.active.session.editing,
		&format!("{}\n{}", "a".repeat(80), vec!["next"; 20].join("\n")),
		EditKind::Atomic,
		Instant::now(),
	);
	app.workspace.active.editor.navigate(
		&mut app.workspace.active.session.editing,
		crate::libraries::editing::Movement::Start,
		false,
	);
	app.workspace.active.editor.surface.viewport.wrap = true;
	app.workspace.code_open = true;
	app.workspace.active.code.surface.viewport.wrap = true;
	terminal.draw(|frame| app.draw(frame)).unwrap();
	app.workspace.active.editor.surface.viewport.top = 1;
	app.workspace.active.code.surface.viewport.top = 1;
	terminal.draw(|frame| app.draw(frame)).unwrap();
	let buffer = terminal.backend().buffer();
	for pane in [
		app.workspace.active.editor.surface.inner,
		app.workspace.active.code.surface.inner,
	] {
		for x in pane.x..pane.x + 6 {
			let cell = &buffer[(x, pane.y)];
			assert_eq!(cell.symbol(), " ");
			assert_eq!(cell.bg, theme::selected_color());
			assert!(cell.modifier.contains(Modifier::BOLD));
		}
	}
}

#[test]
fn gutters_number_logical_lines_and_stay_blank_on_continuations() {
	let mut terminal = Terminal::new(TestBackend::new(40, 20)).unwrap();
	let mut app = App::default();
	app.workspace.active.editor.insert(
		&mut app.workspace.active.session.editing,
		&format!("{}\nnext", "a".repeat(40)),
		EditKind::Atomic,
		Instant::now(),
	);
	app.workspace.active.editor.navigate(
		&mut app.workspace.active.session.editing,
		crate::libraries::editing::Movement::Start,
		false,
	);
	app.workspace.active.editor.surface.viewport.wrap = true;
	app.workspace.code_open = true;
	app.workspace.active.code.surface.viewport.wrap = true;
	terminal.draw(|frame| app.draw(frame)).unwrap();
	let buffer = terminal.backend().buffer();
	assert_eq!(buffer[(4, 1)].symbol(), "1");
	assert_eq!(buffer[(4, 2)].symbol(), " ");
	assert_eq!(buffer[(4, 3)].symbol(), "2");
	let code_y = app.workspace.active.code.surface.text.y;
	assert_eq!(buffer[(4, code_y)].symbol(), "1");
	assert_eq!(buffer[(4, code_y + 1)].symbol(), " ");
	assert_eq!(buffer[(4, code_y + 2)].symbol(), "2");
	assert_eq!(buffer[(7, 2)].symbol(), "a");
	assert_eq!(
		(1..7).map(|x| buffer[(x, 1)].symbol()).collect::<String>(),
		"   1  "
	);
	assert_eq!(
		buffer[(1, app.workspace.active.editor.surface.text.bottom() - 1)].bg,
		theme::field_color()
	);
}

#[test]
fn gutters_resize_for_three_digit_line_numbers() {
	let mut terminal = Terminal::new(TestBackend::new(40, 10)).unwrap();
	let mut app = App::default();
	app.workspace.active.editor.insert(
		&mut app.workspace.active.session.editing,
		&vec!["x"; 100].join("\n"),
		EditKind::Atomic,
		Instant::now(),
	);
	terminal.draw(|frame| app.draw(frame)).unwrap();
	assert_eq!(app.workspace.active.editor.surface.text.x, 8);
	let row = app.workspace.active.editor.surface.text.bottom() - 1;
	let buffer = terminal.backend().buffer();
	assert_eq!(buffer[(3, row)].symbol(), "1");
	assert_eq!(buffer[(4, row)].symbol(), "0");
	assert_eq!(buffer[(5, row)].symbol(), "0");
}

#[test]
fn selected_text_renders_document_attributes_and_selection_overlay() {
	let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
	let mut app = App::default();
	app.workspace.active.editor.insert(
		&mut app.workspace.active.session.editing,
		"Hello\nworld",
		EditKind::Atomic,
		Instant::now(),
	);
	app.workspace.active.editor.navigate(
		&mut app.workspace.active.session.editing,
		crate::libraries::editing::Movement::Start,
		false,
	);
	for _ in 0..5 {
		app.workspace.active.editor.navigate(
			&mut app.workspace.active.session.editing,
			crate::libraries::editing::Movement::Right,
			true,
		);
	}
	app.workspace
		.active
		.session
		.editing
		.apply_tool(Action::Bold);
	terminal.draw(|frame| app.draw(frame)).unwrap();
	assert!(
		terminal.backend().buffer()[(7, 1)]
			.modifier
			.contains(ratatui::style::Modifier::BOLD)
	);
	assert_eq!(
		terminal.backend().buffer()[(7, 1)].bg,
		ratatui::style::Color::DarkGray
	);
}

#[test]
fn renders_unicode_and_tabs_at_shared_display_coordinates() {
	let mut terminal = Terminal::new(TestBackend::new(40, 10)).unwrap();
	let mut app = App::default();
	app.workspace.active.editor.insert(
		&mut app.workspace.active.session.editing,
		"a\t界e\u{301}",
		EditKind::Atomic,
		Instant::now(),
	);
	terminal.draw(|frame| app.draw(frame)).unwrap();
	let buffer = terminal.backend().buffer();
	assert_eq!(buffer[(7, 1)].symbol(), "a");
	assert_eq!(buffer[(11, 1)].symbol(), "界");
	assert_eq!(buffer[(13, 1)].symbol(), "e\u{301}");
	assert_eq!(
		app.workspace.active.editor.surface.text,
		Rect::new(7, 1, 31, 4)
	);
}

#[test]
fn tiny_sizes_and_overlays_never_lose_the_document() {
	let mut app = App::default();
	app.workspace.active.editor.insert(
		&mut app.workspace.active.session.editing,
		"keep me",
		EditKind::Atomic,
		Instant::now(),
	);
	for (width, height) in [(1, 1), (12, 3), (20, 5), (50, 15), (120, 30)] {
		let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
		for settings in [false, true] {
			app.settings = settings.then(|| {
				crate::settings::Settings::new(
					&app.shortcuts.config,
					crate::shortcuts::registry::Category::Editor,
				)
			});
			terminal.draw(|frame| app.draw(frame)).unwrap();
			assert_eq!(
				app.workspace.active.session.editing.document.lines(),
				["keep me"]
			);
		}
	}
}

#[test]
fn unwrapped_panes_reserve_a_row_only_for_horizontal_overflow() {
	use crate::components::scrollbar::Axis;
	use crate::shortcuts::ActionId;

	let mut app = App::default();
	let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
	terminal.draw(|frame| app.draw(frame)).unwrap();
	let buffer = &app.workspace.active;
	let full_heights = [
		buffer.editor.surface.inner.height,
		buffer.code.surface.inner.height,
	];

	app.execute_action(ActionId::EditorWrap);
	for (content, reserved_rows) in [("short".to_owned(), 0), ("x".repeat(120), 1)] {
		app.workspace.active.editor.insert(
			&mut app.workspace.active.session.editing,
			&content,
			EditKind::Atomic,
			Instant::now(),
		);
		terminal.draw(|frame| app.draw(frame)).unwrap();
		let buffer = &app.workspace.active;
		for (surface, full_height) in [&buffer.editor.surface, &buffer.code.surface]
			.into_iter()
			.zip(full_heights)
		{
			assert_eq!(surface.inner.height, full_height - reserved_rows);
			assert_eq!(
				surface
					.bars
					.iter()
					.any(|bar| bar.target == Axis::Horizontal),
				reserved_rows > 0,
			);
		}
	}
}
