use crate::libraries::ansi::Ink;
use crate::libraries::{
	ansi::{Action, Position},
	editing::{EditKind, Movement},
};
use crate::test_editor::Harness as Editor;
use std::time::{Duration, Instant};
use unicode_segmentation::UnicodeSegmentation;

#[test]
fn replacing_a_newline_with_combining_tail_is_one_complete_edit() {
	let mut editor = Editor::default();
	editor.insert("e\n\u{301}", EditKind::Atomic, Instant::now());
	editor.cursor = Position { row: 0, byte: 1 };
	editor.anchor = Some(Position { row: 1, byte: 0 });
	let before = editor.document.clone();
	editor.insert("X", EditKind::Typing, Instant::now());
	assert_eq!(editor.document.text(), "eX\u{301}");
	assert_eq!(editor.cursor, editor.document.end());
	editor.undo(false);
	assert_eq!(editor.document, before);
	assert_eq!(
		editor.selection(),
		Some((Position { row: 0, byte: 1 }, Position { row: 1, byte: 0 }))
	);
}

#[test]
fn wrapping_preserves_document_and_moves_by_screen_rows() {
	let mut editor = Editor::default();
	editor.resize(4, 3);
	editor.insert("abcdefghij\nXYZ", EditKind::Atomic, Instant::now());
	let original = editor.document.clone();
	editor.navigate(Movement::Start, false);
	editor.navigate(Movement::Right, false);
	editor.navigate(Movement::Right, false);
	editor.toggle_wrap();
	editor.navigate(Movement::Down, false);
	assert_eq!(editor.cursor, Position { row: 0, byte: 6 });
	editor.navigate(Movement::Down, true);
	assert_eq!(editor.cursor, Position { row: 0, byte: 10 });
	editor.navigate(Movement::Down, true);
	assert_eq!(editor.cursor, Position { row: 1, byte: 2 });
	editor.navigate(Movement::Start, false);
	editor.click(1, 1);
	assert_eq!(editor.cursor, Position { row: 0, byte: 5 });
	let cursor = editor.cursor;
	editor.resize(2, 2);
	let (row, _) = editor.layout().location(cursor);
	assert!(
		(editor.panel.surface.viewport.top
			..editor.panel.surface.viewport.top + editor.panel.surface.viewport.height)
			.contains(&row)
	);
	editor.toggle_wrap();
	assert_eq!(editor.cursor, cursor);
	assert_eq!(editor.document, original);
}

#[test]
fn word_deletion_preserves_styles_and_is_one_undo_transaction() {
	let mut editor = Editor::default();
	editor.apply_tool(Action::Bold);
	editor.insert("one ", EditKind::Atomic, Instant::now());
	editor.apply_tool(Action::Italic);
	editor.insert("café", EditKind::Atomic, Instant::now());
	let before = editor.document.clone();
	editor.delete_word(Instant::now());
	assert_eq!(editor.document.lines(), ["one "]);
	assert!(editor.document.style_at_offset(0).bold);
	editor.undo(false);
	assert_eq!(editor.document, before);
	editor.navigate(Movement::WordLeft, true);
	assert_eq!(
		editor.selection(),
		Some((Position { row: 0, byte: 4 }, Position { row: 0, byte: 9 }))
	);
	editor.delete_word(Instant::now());
	assert_eq!(editor.document.lines(), ["one "]);
}

#[test]
fn selection_format_replace_and_history_preserve_styles() {
	let mut editor = Editor::default();
	editor.insert("abc\ndef", EditKind::Atomic, Instant::now());
	editor.navigate(Movement::Start, false);
	editor.navigate(Movement::Right, true);
	editor.navigate(Movement::Right, true);
	editor.apply_tool(Action::Bold);
	editor.apply_tool(Action::Italic);
	editor.apply_tool(Action::Foreground(Ink::Ansi(6)));
	assert_eq!(editor.tool_state(Action::Bold), Some(true));
	editor.navigate(Movement::Right, true);
	assert_eq!(editor.tool_state(Action::Bold), None);
	editor.apply_tool(Action::Bold);
	assert_eq!(editor.tool_state(Action::Bold), Some(true));
	editor.insert("界\nX", EditKind::Typing, Instant::now());
	assert_eq!(editor.document.lines(), ["界", "X", "def"]);
	assert!(editor.document.style_at_offset(0).bold);
	assert!(editor.document.style_at_offset(0).italic);
	assert_eq!(editor.document.style_at_offset(0).foreground, Ink::Ansi(6));
	editor.undo(false);
	assert_eq!(editor.document.lines(), ["abc", "def"]);
	assert_eq!(
		editor.selection(),
		Some((Position::default(), Position { row: 0, byte: 3 }))
	);
	editor.undo(true);
	assert_eq!(editor.document.lines(), ["界", "X", "def"]);
}

#[test]
fn style_inheritance_and_reverse_selection_collapse() {
	let mut editor = Editor::default();
	editor.insert("ab", EditKind::Atomic, Instant::now());
	editor.navigate(Movement::Home, false);
	editor.navigate(Movement::Right, true);
	editor.apply_tool(Action::Bold);
	editor.navigate(Movement::Right, false);
	assert_eq!(editor.cursor.byte, 1);
	assert!(editor.typing_style().bold);
	editor.insert("X", EditKind::Typing, Instant::now());
	assert!(editor.document.style_at_offset(1).bold);
	assert!(!editor.document.style_at_offset(2).bold);
	editor.navigate(Movement::Home, false);
	assert!(!editor.typing_style().bold);
	editor.apply_tool(Action::Italic);
	editor.insert("Y", EditKind::Typing, Instant::now());
	assert!(editor.document.style_at_offset(0).italic);
	editor.navigate(Movement::End, false);
	editor.navigate(Movement::Left, true);
	editor.navigate(Movement::Left, true);
	editor.navigate(Movement::Left, false);
	assert_eq!(editor.cursor.byte, 2);
	assert!(editor.selection().is_none());
}

#[test]
fn combining_edits_do_not_split_style_boundaries() {
	let mut editor = Editor::default();
	editor.insert("e", EditKind::Atomic, Instant::now());
	editor.apply_tool(Action::Bold);
	editor.insert("\u{301}", EditKind::Typing, Instant::now());
	assert_eq!(editor.document.style_runs().len(), 1);
	assert_eq!(editor.document.style_runs()[0].end, "e\u{301}".len());
	editor.delete(true, Instant::now());
	assert!(editor.document.style_runs().is_empty());
	editor.undo(false);
	assert_eq!(editor.document.lines(), ["e\u{301}"]);
}

#[test]
fn typing_pause_navigation_and_paste_define_undo_groups() {
	let mut editor = Editor::default();
	let now = Instant::now();
	editor.insert("a", EditKind::Typing, now);
	editor.insert("b", EditKind::Typing, now + Duration::from_millis(100));
	editor.insert("c", EditKind::Typing, now + Duration::from_secs(1));
	editor.undo(false);
	assert_eq!(editor.document.lines(), ["ab"]);
	editor.undo(false);
	assert!(editor.document.is_empty());
	editor.undo(true);
	assert_eq!(editor.document.lines(), ["ab"]);
	editor.move_cursor(Movement::Left);
	editor.insert("X\nY", EditKind::Atomic, now);
	assert_eq!(editor.document.lines(), ["aX", "Yb"]);
	editor.undo(false);
	assert_eq!(editor.document.lines(), ["ab"]);
	assert_eq!(editor.cursor.byte, 1);
	editor.insert("z", EditKind::Typing, now);
	editor.undo(true);
	assert_eq!(editor.document.lines(), ["azb"]);
}

#[test]
fn deletes_join_lines_and_restore_whole_graphemes() {
	let mut editor = Editor::default();
	let now = Instant::now();
	editor.insert("e\u{301}\n界👩‍💻", EditKind::Atomic, now);
	editor.delete(true, now);
	assert_eq!(editor.document.lines(), ["e\u{301}", "界"]);
	editor.undo(false);
	assert_eq!(editor.document.lines(), ["e\u{301}", "界👩‍💻"]);
	editor.move_cursor(Movement::Home);
	editor.delete(true, now);
	assert_eq!(editor.document.lines(), ["e\u{301}界👩‍💻"]);
	editor.move_cursor(Movement::Home);
	editor.delete(false, now);
	assert_eq!(editor.document.lines(), ["界👩‍💻"]);
}

#[test]
fn vertical_navigation_remembers_column_across_short_lines() {
	let mut editor = Editor::default();
	editor.resize(10, 3);
	editor.insert("abcdef\nx\nabcdef", EditKind::Atomic, Instant::now());
	editor.move_cursor(Movement::Up);
	assert_eq!(editor.cursor, Position { row: 1, byte: 1 });
	editor.move_cursor(Movement::Up);
	assert_eq!(editor.cursor, Position { row: 0, byte: 6 });
}

#[test]
fn resizing_scroll_and_clicks_keep_valid_coordinates() {
	let mut editor = Editor::default();
	editor.resize(4, 2);
	editor.insert("0123456789\nx\ny\nz", EditKind::Atomic, Instant::now());
	assert_eq!(editor.panel.surface.viewport.top, 2);
	editor.scroll(false);
	assert_eq!(editor.panel.surface.viewport.top, 0);
	editor.click(3, 0);
	assert_eq!(editor.cursor.byte, 3);
	editor.move_cursor(Movement::End);
	assert_eq!(editor.panel.surface.viewport.left, 7);
	editor.resize(100, 100);
	assert_eq!(editor.panel.surface.viewport.top, 0);
	assert_eq!(editor.panel.surface.viewport.left, 0);
	// Expand the viewport without moving the document position.
	assert_eq!(editor.cursor.byte, 10);
	editor.resize(0, 0);
	editor.move_cursor(Movement::Finish);
	editor.resize(4, 2);
	assert_eq!(editor.panel.surface.viewport.top, 2);
	editor.move_cursor(Movement::Start);
	editor.delete(false, Instant::now());
	assert_eq!(editor.panel.surface.viewport.left, 0);
}

#[test]
fn deterministic_edit_sequences_preserve_invariants_and_roundtrip_history() {
	let mut editor = Editor::default();
	editor.resize(8, 3);
	let mut state = 12345u32;
	for _ in 0..180 {
		state = state.wrapping_mul(1664525).wrapping_add(1013904223);
		match state % 8 {
			0..=2 => editor.insert(
				["e\u{301}", "界", "\n", "\t", "👩‍💻"][(state as usize / 8) % 5],
				EditKind::Atomic,
				Instant::now(),
			),
			3 => {
				editor.break_group();
				editor.delete(true, Instant::now());
			}
			4 => {
				editor.break_group();
				editor.delete(false, Instant::now());
			}
			5 => editor.move_cursor(Movement::Left),
			6 => editor.move_cursor(Movement::Up),
			_ => editor.move_cursor(Movement::Right),
		}
		let line = &editor.document.lines()[editor.cursor.row];
		assert!(
			line.grapheme_indices(true)
				.map(|(byte, _)| byte)
				.chain([line.len()])
				.any(|byte| byte == editor.cursor.byte)
		);
	}
	let final_document = editor.document.clone();
	while editor.can_undo() {
		editor.undo(false);
	}
	assert!(editor.document.is_empty());
	while editor.can_redo() {
		editor.undo(true);
	}
	assert_eq!(editor.document, final_document);
}
