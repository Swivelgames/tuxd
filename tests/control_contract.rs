use terminal_ux_designer::libraries::controls::{self, Direction, Field, FieldAction as A, Range};

#[test]
fn field_edits_use_graphemes_and_preserve_boundaries_when_text_joins() {
	let mut field = Field::new("e\u{301}界".into());
	field.apply(A::Left);
	assert_eq!(field.cursor(), "e\u{301}".len());
	field.apply(A::Backspace);
	assert_eq!(field.text(), "界");
	assert_eq!(field.cursor(), 0);
	field.apply(A::Delete);
	assert_eq!(field.text(), "");

	// Removing the separator joins the two regional indicators into one grapheme.
	field.set("🇺x🇳".into());
	field.apply(A::Left);
	field.apply(A::Backspace);
	assert_eq!(field.text(), "🇺🇳");
	assert_eq!(field.cursor(), field.text().len());
	field.apply(A::Left);
	assert_eq!(field.cursor(), 0);
	field.insert("a", 100);
	assert_eq!(field.cursor(), 1);
	field.apply(A::End);
	field.apply(A::Backspace);
	assert_eq!(field.text(), "a");
}

#[test]
fn selection_replacement_limits_and_navigation_remain_explicit() {
	let mut field = Field::selecting("old".into());
	field.insert("new", 3);
	assert_eq!(field.text(), "new");
	assert!(!field.selected());
	field.insert("x", 3);
	assert_eq!(field.text(), "new");
	field.apply(A::SelectAll);
	field.apply(A::Beginning);
	assert!(!field.selected());
	assert_eq!(field.cursor(), 0);
	field.apply(A::Right);
	field.insert("!", 4);
	assert_eq!(field.text(), "n!ew");
	field.apply(A::SelectAll);
	field.apply(A::Delete);
	assert_eq!(field.text(), "");
	// Preserve field callers' existing replace-first limit semantics.
	field.set("old".into());
	field.select_all();
	field.insert("too long", 3);
	assert_eq!(field.text(), "");
}

#[test]
fn grid_boundaries_pages_and_cycle_are_shared_without_wrapping_grid_rows() {
	assert_eq!(controls::navigate(6, 10, 7, 5, Direction::Down), 9);
	assert_eq!(controls::navigate(9, 10, 7, 5, Direction::Down), 9);
	assert_eq!(controls::navigate(3, 10, 7, 5, Direction::Up), 3);
	assert_eq!(controls::navigate(9, 10, 7, 5, Direction::Up), 2);
	assert_eq!(controls::navigate(0, 256, 7, 5, Direction::PageDown), 35);
	assert_eq!(controls::navigate(255, 256, 7, 5, Direction::PageUp), 220);
	assert_eq!(controls::navigate(4, 0, 0, 0, Direction::End), 0);
	assert_eq!(controls::cycle(0, 3, true), 2);
	assert_eq!(controls::cycle(2, 3, false), 0);
	assert_eq!(controls::cycle(7, 0, false), 0);
}

#[test]
fn range_keys_and_pointer_share_clamped_values() {
	let range = Range { min: 0, max: 255 };
	assert_eq!(range.step(250, 10), 255);
	assert_eq!(range.step(3, -10), 0);
	assert_eq!(range.at_cell(0, 10), 0);
	assert_eq!(range.at_cell(9, 10), 255);
	assert_eq!(range.at_cell(100, 10), 255);
	assert_eq!(range.at_cell(0, 0), 0);
}
