//! Cross-module acceptance and repeatable performance checks.
use crate::test_editor::Harness as Editor;
use crate::workspace::TextPane;
use crate::{
	app::App,
	libraries::ansi::{self as styling, Action, Ink},
	libraries::editing::{EditKind, Movement},
};
use std::time::Instant;
use unicode_segmentation::UnicodeSegmentation;

fn verify(editor: &crate::libraries::editing::Editing) {
	let bytes = crate::adapters::native_document::encode(&editor.document).unwrap();
	assert_eq!(
		crate::adapters::native_document::decode(&bytes).unwrap(),
		editor.document
	);
	for position in [Some(editor.cursor), editor.anchor].into_iter().flatten() {
		let line = &editor.document.lines()[position.row];
		assert!(
			line.grapheme_indices(true)
				.map(|(i, _)| i)
				.chain([line.len()])
				.any(|i| i == position.byte)
		);
	}
	let ansi = editor.document.to_raw_ansi();
	let imported = crate::libraries::ansi::parse(&ansi, crate::libraries::ansi::Mode::Raw).unwrap();
	assert_eq!(imported.document, editor.document);
	assert_eq!(ansi, editor.document.to_raw_ansi());
}

#[test]
fn mixed_unicode_styles_history_and_storage() {
	for seed in [1u64, 17, 987654321] {
		let mut editor = Editor::default();
		let mut random = seed;
		for step in 0..1200 {
			random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
			let n = (random >> 32) as usize;
			match n % 12 {
				0..=3 => editor.insert(
					[
						"e\u{301}",
						"界",
						"👩🏽‍💻",
						"🇺🇳",
						"\t",
						"\n",
						"\u{200d}",
						"\u{301}",
						"क्ष",
					][n / 12 % 9],
					EditKind::Atomic,
					Instant::now(),
				),
				4 => editor.navigate(Movement::Left, n & 128 != 0),
				5 => editor.navigate(Movement::Right, n & 128 != 0),
				6 => editor.apply_tool(
					[
						Action::Bold,
						Action::Italic,
						Action::Underline,
						Action::Foreground(Ink::Rgb(80, 120, 240)),
						Action::Background(Ink::Indexed(200)),
					][n / 12 % 5],
				),
				7 => {
					editor.break_group();
					editor.delete(n & 128 != 0, Instant::now());
				}
				8 => editor.undo(false),
				9 => editor.undo(true),
				10 => {
					editor.resize(1 + n % 80, 1 + n % 24);
					editor.toggle_wrap();
				}
				_ => editor.navigate(Movement::WordLeft, n & 128 != 0),
			}
			verify(&editor);
			if step % 100 == 0 {
				let before = editor.document.clone();
				editor.insert("temporary", EditKind::Atomic, Instant::now());
				let after = editor.document.clone();
				editor.undo(false);
				assert_eq!(editor.document, before);
				editor.undo(true);
				assert_eq!(editor.document, after);
				editor.undo(false);
			}
		}
	}
}

fn fixture() -> crate::libraries::ansi::Document {
	let mut text = String::new();
	let mut runs = Vec::new();
	let mut offset = 0;
	for i in 0..1000 {
		let line = format!(
			"{i:04}\tTerminal design: café e\u{301} 界 👩🏽‍💻 🇺🇳 ─│┌┐ sample text for scrolling and wrapping."
		);
		let end = offset + line.len() + usize::from(i < 999);
		let style = styling::TextStyle {
			bold: i % 2 == 0,
			foreground: Ink::Indexed((i % 256) as u8),
			..Default::default()
		};
		runs.push(crate::libraries::ansi::Run {
			start: offset,
			end,
			style,
		});
		offset = end;
		text.push_str(&line);
		if i < 999 {
			text.push('\n');
		}
	}
	crate::libraries::ansi::Document::try_from(crate::libraries::ansi::DocumentData {
		text,
		styles: crate::libraries::ansi::Styles { runs },
	})
	.unwrap()
}

#[test]
fn representative_document_survives_resize_and_pane_changes() {
	let mut app = App::default();
	app.workspace.active.session.editing.document = fixture();
	let original = app.workspace.active.session.editing.document.clone();
	for (w, h) in [(120, 40), (64, 24), (40, 12), (1, 1), (80, 24)] {
		let mut terminal =
			ratatui::Terminal::new(ratatui::backend::TestBackend::new(w, h)).unwrap();
		for pane in [TextPane::Code, TextPane::Editor, TextPane::Code] {
			app.workspace.toggle_pane(pane);
			terminal.draw(|frame| app.draw(frame)).unwrap();
			assert_eq!(app.workspace.active.session.editing.document, original);
		}
	}
	verify(&app.workspace.active.session.editing);
}

#[test]
#[ignore = "run in release mode with --nocapture to measure the documented fixture"]
fn modest_document_performance() {
	let mut app = App::default();
	app.workspace.active.session.editing.document = fixture();
	if let Ok(path) = std::env::var("TUXD_PERF_FIXTURE") {
		std::fs::write(
			path,
			crate::adapters::native_document::encode(
				&app.workspace.active.session.editing.document,
			)
			.unwrap(),
		)
		.unwrap();
	}
	let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 40)).unwrap();
	let mut samples = Vec::new();
	for i in 0..40 {
		let start = Instant::now();
		app.workspace.active.editor.navigate(
			&mut app.workspace.active.session.editing,
			if i % 2 == 0 {
				Movement::Finish
			} else {
				Movement::Start
			},
			false,
		);
		app.workspace.active.editor.insert(
			&mut app.workspace.active.session.editing,
			"x",
			EditKind::Atomic,
			Instant::now(),
		);
		terminal.draw(|frame| app.draw(frame)).unwrap();
		app.workspace
			.active
			.editor
			.undo(&mut app.workspace.active.session.editing, false);
		samples.push(start.elapsed().as_secs_f64() * 1000.0);
	}
	samples.sort_by(f64::total_cmp);
	println!(
		"1000 lines, {} text bytes, 1000 style runs; edit + frame + undo: median {:.2} ms, p95 {:.2} ms, max {:.2} ms",
		app.workspace
			.active
			.session
			.editing
			.document
			.lines()
			.join("\n")
			.len(),
		samples[20],
		samples[37],
		samples[39]
	);
	let start = Instant::now();
	verify(&app.workspace.active.session.editing);
	println!(
		"validate + project roundtrip + ANSI roundtrip: {:.2} ms",
		start.elapsed().as_secs_f64() * 1000.0
	);
}

#[test]
fn fallback_layout_preserves_grapheme_hits_and_roundtrips() {
	let text = "café é 界 👩🏽‍💻 🇺🇳";
	let mut editor = Editor::default();
	editor.insert(text, EditKind::Atomic, Instant::now());
	let layout = crate::libraries::text::Layout::new(editor.document.lines(), 120, true, true);
	for (y, row) in layout.rows.iter().enumerate() {
		for cell in &row.cells {
			for x in cell.x..cell.x + cell.width {
				assert_eq!(layout.hit(y, x).byte, cell.byte);
			}
		}
	}
	verify(&editor);
	assert!(editor.document.to_raw_ansi().contains(text));
	editor.navigate(Movement::Home, true);
	editor.apply_tool(Action::Bold);
	assert_eq!(editor.document.lines()[0], text);
	verify(&editor);
}

#[test]
fn composed_emoji_newline_redraw_clears_previous_rows() {
	let mut app = App::default();
	app.workspace.active.editor.insert(
		&mut app.workspace.active.session.editing,
		"café é 界 👩🏽‍💻 🇺🇳",
		EditKind::Atomic,
		Instant::now(),
	);
	app.workspace.active.editor.navigate(
		&mut app.workspace.active.session.editing,
		Movement::Home,
		false,
	);
	let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(110, 36)).unwrap();
	terminal.draw(|f| app.draw(f)).unwrap();
	let initial = terminal.backend().buffer().clone();
	for _ in 0..10 {
		app.workspace.active.editor.insert(
			&mut app.workspace.active.session.editing,
			"\n",
			EditKind::Atomic,
			Instant::now(),
		);
		terminal.draw(|f| app.draw(f)).unwrap();
	}
	for _ in 0..10 {
		app.workspace.active.editor.delete(
			&mut app.workspace.active.session.editing,
			true,
			Instant::now(),
		);
		terminal.draw(|f| app.draw(f)).unwrap();
	}
	// A wide glyph's continuation is not independently drawn by the terminal.
	// TestBackend may retain its prior style when the glyph moves back over it.
	for y in 0..36 {
		let mut x = 0;
		while x < 110 {
			let expected = &initial[(x, y)];
			assert_eq!(
				&terminal.backend().buffer()[(x, y)],
				expected,
				"cell ({x}, {y})"
			);
			x += unicode_width::UnicodeWidthStr::width(expected.symbol()).max(1) as u16;
		}
	}
	verify(&app.workspace.active.session.editing);
}
