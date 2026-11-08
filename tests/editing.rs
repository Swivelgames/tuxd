//! The editing contract is usable without application objects or terminal widgets.
use std::time::Instant;
use tuxd::libraries::{
	ansi::{Action, Position},
	editing::{EditKind, Editing, Movement, Navigation},
	text::Layout,
};

#[test]
fn visual_navigation_selection_and_atomic_history_use_pure_layout() {
	let mut editing = Editing::default();
	editing.insert("ab界é\nshort", EditKind::Atomic, Instant::now());
	let original = editing.document.clone();
	let layout = Layout::new(editing.document.lines(), 4, true, true);
	let navigate = |editing: &mut Editing, movement, selecting| {
		editing.navigate(
			movement,
			selecting,
			Navigation {
				layout: &layout,
				wrap: true,
				height: 2,
			},
		)
	};
	navigate(&mut editing, Movement::Start, false);
	navigate(&mut editing, Movement::Down, true);
	assert_eq!(editing.cursor, Position { row: 0, byte: 5 });
	assert!(editing.selection().is_some());
	editing.apply_tool(Action::Bold);
	assert_eq!(editing.tool_state(Action::Bold), Some(true));
	editing.insert("replacement", EditKind::Typing, Instant::now());
	editing.undo(false);
	assert!(editing.selection().is_some());
	editing.undo(false);
	assert_eq!(editing.document, original);
}

#[test]
fn history_retains_the_two_hundred_snapshot_bound() {
	let mut editing = Editing::default();
	for _ in 0..205 {
		editing.insert("x", EditKind::Atomic, Instant::now());
	}
	for _ in 0..205 {
		editing.undo(false);
	}
	assert_eq!(editing.document.text(), "xxxxx");
	assert!(!editing.can_undo());
	for _ in 0..205 {
		editing.undo(true);
	}
	assert_eq!(editing.document.text(), "x".repeat(205));
	assert!(!editing.can_redo());
}
