//! Input for scripts/character_width_smoke.py; no terminal I/O here.
use tuxd::libraries::text::display::display_grapheme;
use unicode_width::UnicodeWidthStr;

fn main() {
	let raw = std::env::args().any(|arg| arg == "--raw");
	let mut samples: Vec<String> = include_str!("../src/libraries/characters/data/catalog.tsv")
		.lines()
		.map(|line| {
			let code = u32::from_str_radix(line.split('\t').next().unwrap(), 16).unwrap();
			char::from_u32(code).unwrap().to_string()
		})
		.collect();
	for glyph in ['☝', '⛹', '✌', '✍', '🏋', '🏌', '🕴', '🕵', '🖐'] {
		for suffix in ['\u{fe0e}', '\u{fe0f}'] {
			samples.push(format!("{glyph}{suffix}"));
		}
	}
	samples.extend(["👩🏽‍💻", "🇺🇳", "❤️", "1️⃣"].map(String::from));
	for source in samples {
		let display = if raw {
			source.clone()
		} else {
			display_grapheme(&source).into_owned()
		};
		println!(
			"{}",
			serde_json::json!({"source": source, "display": display, "width": display.width()})
		);
	}
}
