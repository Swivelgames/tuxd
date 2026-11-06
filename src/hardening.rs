//! Cross-module acceptance and repeatable performance checks.
use crate::{
	app::App,
	editor::{EditKind, Editor, Movement},
	persistence,
	styling::{self, Action, Ink},
	ui,
};
use std::time::Instant;
use unicode_segmentation::UnicodeSegmentation;

fn verify(editor: &Editor) {
	let bytes = persistence::encode(&editor.document).unwrap();
	assert_eq!(persistence::decode(&bytes).unwrap(), editor.document);
	for position in [Some(editor.cursor), editor.anchor].into_iter().flatten() {
		let line = &editor.document.lines[position.row];
		assert!(
			line.grapheme_indices(true)
				.map(|(i, _)| i)
				.chain([line.len()])
				.any(|i| i == position.byte)
		);
	}
	let ansi = styling::ansi(&editor.document);
	let imported = crate::ansi_import::parse(&ansi, crate::ansi_import::Mode::Raw).unwrap();
	assert_eq!(imported.document, editor.document);
	assert_eq!(ansi, styling::ansi(&editor.document));
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

fn fixture() -> crate::document::Document {
	let mut doc = crate::document::Document::default();
	doc.lines.clear();
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
		doc.styles.push(offset, end, style);
		offset = end;
		doc.lines.push(line);
	}
	doc
}

#[test]
fn representative_document_survives_resize_and_pane_changes() {
	let mut app = App::default();
	app.editor.document = fixture();
	let original = app.editor.document.clone();
	for (w, h) in [(120, 40), (64, 24), (40, 12), (1, 1), (80, 24)] {
		let mut terminal =
			ratatui::Terminal::new(ratatui::backend::TestBackend::new(w, h)).unwrap();
		for code in [true, false, true] {
			app.toggle_pane(code);
			terminal.draw(|frame| ui::draw(frame, &mut app)).unwrap();
			assert_eq!(app.editor.document, original);
		}
	}
	verify(&app.editor);
}

#[test]
#[ignore = "run in release mode with --nocapture to measure the documented fixture"]
fn modest_document_performance() {
	let mut app = App::default();
	app.editor.document = fixture();
	if let Ok(path) = std::env::var("TUXD_PERF_FIXTURE") {
		std::fs::write(path, persistence::encode(&app.editor.document).unwrap()).unwrap();
	}
	let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 40)).unwrap();
	let mut samples = Vec::new();
	for i in 0..40 {
		let start = Instant::now();
		app.editor.navigate(
			if i % 2 == 0 {
				Movement::Finish
			} else {
				Movement::Start
			},
			false,
		);
		app.editor.insert("x", EditKind::Atomic, Instant::now());
		terminal.draw(|frame| ui::draw(frame, &mut app)).unwrap();
		app.editor.undo(false);
		samples.push(start.elapsed().as_secs_f64() * 1000.0);
	}
	samples.sort_by(f64::total_cmp);
	println!(
		"1000 lines, {} text bytes, 1000 style runs; edit + frame + undo: median {:.2} ms, p95 {:.2} ms, max {:.2} ms",
		app.editor.document.lines.join("\n").len(),
		samples[20],
		samples[37],
		samples[39]
	);
	let start = Instant::now();
	verify(&app.editor);
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
	let layout = crate::layout::Layout::new(&editor.document.lines, 120, true, true);
	for (y, row) in layout.rows.iter().enumerate() {
		for cell in &row.cells {
			for x in cell.x..cell.x + cell.width {
				assert_eq!(layout.hit(y, x).byte, cell.byte);
			}
		}
	}
	verify(&editor);
	assert!(styling::ansi(&editor.document).contains(text));
	editor.navigate(Movement::Home, true);
	editor.apply_tool(Action::Bold);
	assert_eq!(editor.document.lines[0], text);
	verify(&editor);
}

#[test]
fn composed_emoji_newline_redraw_clears_previous_rows() {
	let mut app = App::default();
	app.editor
		.insert("café é 界 👩🏽‍💻 🇺🇳", EditKind::Atomic, Instant::now());
	app.editor.navigate(Movement::Home, false);
	let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(110, 36)).unwrap();
	terminal.draw(|f| ui::draw(f, &mut app)).unwrap();
	let initial = terminal.backend().buffer().clone();
	for _ in 0..10 {
		app.editor.insert("\n", EditKind::Atomic, Instant::now());
		terminal.draw(|f| ui::draw(f, &mut app)).unwrap();
	}
	for _ in 0..10 {
		app.editor.delete(true, Instant::now());
		terminal.draw(|f| ui::draw(f, &mut app)).unwrap();
	}
	assert_eq!(terminal.backend().buffer(), &initial);
	verify(&app.editor);
}
