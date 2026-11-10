use super::*;
use crate::{
	panel::{Control, Effect, Panel},
	shortcuts::ActionId as A,
};

fn focus(panel: &mut Panel, control: Control) {
	panel.focus_character_control(control);
	assert_eq!(panel.items()[panel.selected].control, control);
}

#[test]
fn search_is_editable_and_enter_focuses_results_without_inserting() {
	let mut panel = Panel {
		active_section: Some(2),
		..Panel::default()
	};
	focus(&mut panel, Control::CharacterGroup);
	panel.handle(A::CharacterSearch);
	panel.insert_text("U+2500");
	assert_eq!(panel.characters.results, vec!['─']);
	assert!(matches!(panel.handle(A::Submit), Effect::None));
	assert_eq!(panel.items()[panel.selected].control, Control::Glyph('─'));
	assert!(matches!(panel.handle(A::Activate), Effect::Glyph('─')));
	panel.handle(A::CharacterSearch);
	panel.handle(A::SelectAll);
	panel.insert_text("no such character qqq");
	assert!(panel.characters.results.is_empty());
	assert!(matches!(panel.handle(A::Submit), Effect::None));
	assert_eq!(
		panel.items()[panel.selected].control,
		Control::CharacterSearch
	);
	panel.handle(A::SelectAll);
	panel.handle(A::FieldDeleteAfter);
	assert!(!panel.characters.results.is_empty());
}

#[test]
fn navigation_and_scrolling_bound_the_rendered_grid() {
	let mut panel = Panel {
		active_section: Some(2),
		..Panel::default()
	};
	panel.characters.query.set("md-".into());
	panel.characters.refresh();
	assert!(panel.characters.results.len() > 5_000);
	let first = panel.characters.results[0];
	focus(&mut panel, Control::Glyph(first));
	panel.handle(A::FieldEnd);
	assert_eq!(panel.characters.selected + 1, panel.characters.grid.len());
	let last_row = panel.characters.grid[panel.characters.selected].row;
	panel.handle(A::FieldPageUp);
	let page_row = panel.characters.grid[panel.characters.selected].row;
	assert!(last_row.abs_diff(page_row + panel.characters.rows) <= 1);
	panel.scroll_characters(usize::MAX);
	assert_eq!(
		panel.characters.top,
		panel.characters.content_rows - panel.characters.rows
	);
	panel.handle(A::FieldHome);
	assert_eq!(panel.characters.top, 0);
	assert_eq!(panel.characters.selected, 0);
}

#[test]
fn clearing_search_restores_the_browse_group_and_subgroup() {
	let mut characters = Characters::default();
	let original = characters.results.clone();
	characters.insert("U+2801");
	assert_eq!(characters.results, vec!['⠁']);
	characters.local_search = true;
	characters.refresh();
	assert!(characters.results.is_empty());
	characters.query.set(String::new());
	characters.refresh();
	assert_eq!(characters.results, original);
}

#[test]
fn dropdowns_stage_jump_cancel_and_commit_without_editing_the_document() {
	use crate::{app::App, shortcuts::InputScope};
	use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
	use ratatui::{Terminal, backend::TestBackend};
	let mut app = App::default();
	let mut terminal = Terminal::new(TestBackend::new(64, 40)).unwrap();
	terminal.draw(|frame| app.draw(frame)).unwrap();
	app.workspace.select_slim_tab(3);
	app.workspace.editor_open = false;
	terminal.draw(|frame| app.draw(frame)).unwrap();
	let key = |code| Event::Key(KeyEvent::new(code, KeyModifiers::NONE));
	focus(&mut app.workspace.panel, Control::CharacterGroup);
	app.handle(key(KeyCode::Enter));
	assert_eq!(app.input_scope(), InputScope::CharacterPicker);
	app.handle(Event::Paste("Braille".into()));
	assert_eq!(app.workspace.panel.characters.group, 0);
	terminal.draw(|frame| app.draw(frame)).unwrap();
	app.handle(key(KeyCode::Esc));
	assert_eq!(app.workspace.panel.characters.group, 0);
	assert!(app.workspace.panel.characters.dropdown.is_none());
	app.handle(key(KeyCode::Enter));
	app.handle(Event::Paste("Braille".into()));
	app.handle(key(KeyCode::Enter));
	assert_eq!(
		CATALOG.groups[app.workspace.panel.characters.group].name,
		"Braille"
	);
	assert!(app.workspace.active.session.editing.document.is_empty());
}

#[test]
fn mouse_dropdown_takes_focus_and_commits_the_clicked_choice() {
	use crate::{app::App, shortcuts::InputScope, workspace::Pane};
	use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
	use ratatui::{Terminal, backend::TestBackend};
	let mut app = App::default();
	let mut terminal = Terminal::new(TestBackend::new(64, 40)).unwrap();
	terminal.draw(|frame| app.draw(frame)).unwrap();
	app.workspace.select_slim_tab(3);
	app.workspace.editor_open = false;
	app.workspace.focus = Pane::Editor;
	terminal.draw(|frame| app.draw(frame)).unwrap();
	let panel = &app.workspace.panel;
	let items = panel.items();
	let (rect, _) = panel
		.tool_areas
		.iter()
		.find(|(_, index)| items[*index].control == Control::CharacterGroup)
		.unwrap();
	let click = |column, row| {
		Event::Mouse(MouseEvent {
			kind: MouseEventKind::Down(MouseButton::Left),
			column,
			row,
			modifiers: KeyModifiers::NONE,
		})
	};
	app.handle(click(rect.x, rect.y));
	assert_eq!(app.workspace.focus, Pane::Tools);
	assert_eq!(app.input_scope(), InputScope::CharacterPicker);
	terminal.draw(|frame| app.draw(frame)).unwrap();
	let buffer = terminal.backend().buffer();
	let (x, y) = (0..40)
		.find_map(|y| {
			let text: String = (0..64).map(|x| buffer[(x, y)].symbol()).collect();
			text.find("Arrows").map(|x| (x as u16, y))
		})
		.unwrap();
	app.handle(click(x, y));
	assert!(app.workspace.panel.characters.dropdown.is_none());
	assert_eq!(
		CATALOG.groups[app.workspace.panel.characters.group].name,
		"Arrows"
	);
	assert!(app.workspace.active.session.editing.document.is_empty());
}

#[test]
fn characters_fill_available_height_and_find_controls_share_one_row() {
	use crate::app::App;
	use ratatui::{Terminal, backend::TestBackend};
	let mut app = App::default();
	let mut terminal = Terminal::new(TestBackend::new(64, 50)).unwrap();
	terminal.draw(|frame| app.draw(frame)).unwrap();
	app.workspace.select_slim_tab(3);
	app.workspace.editor_open = false;
	terminal.draw(|frame| app.draw(frame)).unwrap();
	let panel = &app.workspace.panel;
	assert_eq!(panel.characters.height, panel.height);
	assert!(panel.characters.rows > ROWS);
	let bounds = |control| {
		let items = panel.items();
		panel
			.tool_areas
			.iter()
			.find(|(_, index)| items[*index].control == control)
			.unwrap()
			.0
	};
	let find = bounds(Control::CharacterSearch);
	let scope = bounds(Control::CharacterScope);
	let clear = bounds(Control::CharacterClear);
	assert_eq!(find.y, scope.y);
	assert_eq!(scope.y, clear.y);
	assert!(find.right() <= scope.x && scope.right() <= clear.x);
	assert_eq!(clear.width, 1);

	let mut small = Terminal::new(TestBackend::new(40, 12)).unwrap();
	small.draw(|frame| app.draw(frame)).unwrap();
	assert_eq!(app.workspace.panel.characters.height, MIN_HEIGHT);
	let mut wide = Terminal::new(TestBackend::new(100, 60)).unwrap();
	wide.draw(|frame| app.draw(frame)).unwrap();
	let panel = &app.workspace.panel;
	let (start, height) = panel.layout().1[2];
	assert_eq!(start + height, panel.height);
}

#[test]
fn fluid_grid_uses_available_width_and_keeps_one_space_between_glyphs() {
	use crate::app::App;
	use ratatui::{Terminal, backend::TestBackend};
	let mut app = App::default();
	app.workspace.panel.characters.group = CATALOG
		.groups
		.iter()
		.position(|group| group.name == "Braille")
		.unwrap();
	app.workspace.panel.characters.refresh();
	let mut counts = Vec::new();
	for width in [40, 64, 120] {
		let mut terminal = Terminal::new(TestBackend::new(width, 60)).unwrap();
		terminal.draw(|frame| app.draw(frame)).unwrap();
		if width <= 64 {
			app.workspace.select_slim_tab(3);
		}
		terminal.draw(|frame| app.draw(frame)).unwrap();
		let panel = &app.workspace.panel;
		let items = panel.items();
		let bounds: Vec<_> = panel
			.tool_areas
			.iter()
			.filter(|(_, index)| matches!(items[*index].control, Control::Glyph(_)))
			.map(|(rect, _)| *rect)
			.collect();
		let first_row = bounds[0].y;
		counts.push(bounds.iter().filter(|rect| rect.y == first_row).count());
		for pair in bounds.windows(2).filter(|pair| pair[0].y == pair[1].y) {
			assert_eq!(pair[0].right() + 1, pair[1].x);
		}
		assert!(
			bounds
				.iter()
				.all(|rect| rect.right() < panel.area.right() - 2)
		);
	}
	assert!(counts[1] > counts[0]);
	assert!(counts[2] < counts[1]);
}

#[test]
fn types_start_expanded_and_collapse_without_affecting_other_types() {
	let mut characters = Characters::default();
	let headers: Vec<_> = characters
		.grid
		.iter()
		.filter_map(|cell| match cell.control {
			Control::CharacterType(group, subtype) => Some((group, subtype)),
			_ => None,
		})
		.collect();
	assert!(headers.len() > 1);
	assert!(characters.closed.is_empty());
	let original = characters.grid.len();
	let (group, subtype) = headers[0];
	characters.toggle_type(group, subtype);
	assert!(characters.grid.len() < original);
	assert_eq!(
		characters.selected_control(),
		Some(Control::CharacterType(group, subtype))
	);
	assert!(
		characters
			.grid
			.iter()
			.any(|cell| matches!(cell.control, Control::Glyph(_)))
	);
	assert!(!characters.grid.iter().any(|cell| match cell.control {
		Control::Glyph(glyph) =>
			CATALOG.get(glyph).unwrap().subgroup == CATALOG.groups[group].subgroups[subtype],
		_ => false,
	}));
	characters.toggle_type(group, subtype);
	assert_eq!(characters.grid.len(), original);
}

#[test]
fn reflow_preserves_selection_and_double_width_glyphs_fit() {
	let mut characters = Characters {
		group: CATALOG
			.groups
			.iter()
			.position(|group| group.name == "Emoji")
			.unwrap(),
		..Characters::default()
	};
	characters.refresh();
	characters.resize(23, 28);
	characters.navigate(crate::libraries::controls::Direction::End);
	let selected = characters.selected_control();
	characters.resize(23, 11);
	characters.reveal_selection();
	assert_eq!(characters.selected_control(), selected);
	assert!(
		characters
			.grid
			.iter()
			.filter(|cell| matches!(cell.control, Control::Glyph(_)))
			.all(|cell| cell.x + 2 <= 11)
	);
	let row = characters.grid[characters.selected].row;
	assert!((characters.top..characters.top + characters.rows).contains(&row));
}

#[test]
fn type_headings_toggle_by_keyboard_and_mouse_without_inserting() {
	use crate::app::App;
	use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
	use ratatui::{Terminal, backend::TestBackend};
	let mut app = App::default();
	let mut terminal = Terminal::new(TestBackend::new(64, 40)).unwrap();
	terminal.draw(|frame| app.draw(frame)).unwrap();
	app.workspace.select_slim_tab(3);
	app.workspace.editor_open = false;
	terminal.draw(|frame| app.draw(frame)).unwrap();
	let header = app.workspace.panel.characters.grid[0].control;
	focus(&mut app.workspace.panel, header);
	let count = app.workspace.panel.characters.grid.len();
	assert!(matches!(
		app.workspace.panel.handle(A::Activate),
		Effect::Focus
	));
	assert!(app.workspace.panel.characters.grid.len() < count);
	terminal.draw(|frame| app.draw(frame)).unwrap();
	let panel = &mut app.workspace.panel;
	let items = panel.items();
	let (rect, _) = panel
		.tool_areas
		.iter()
		.find(|(_, index)| items[*index].control == header)
		.unwrap();
	let mouse = MouseEvent {
		kind: MouseEventKind::Down(MouseButton::Left),
		column: rect.x,
		row: rect.y,
		modifiers: KeyModifiers::NONE,
	};
	assert!(matches!(panel.mouse(mouse), Some(Effect::Focus)));
	assert_eq!(panel.characters.grid.len(), count);
	assert_eq!(panel.items()[panel.selected].control, header);
	assert!(app.workspace.active.session.editing.document.is_empty());
}
