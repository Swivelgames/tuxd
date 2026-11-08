use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub const TAB_STOP: usize = 4;

pub struct Glyph<'a> {
	pub byte: usize,
	pub column: usize,
	pub width: usize,
	pub text: &'a str,
}

/// Conservative display for clusters whose width differs across terminal stacks.
/// ZWJ/modifier/presentation sequences and regional indicators are escaped until
/// component rendering can negotiate terminal widths. Never use this for export.
/// Document bytes and extended-grapheme editing remain unchanged.
pub fn display_grapheme(text: &str) -> std::borrow::Cow<'_, str> {
	let scalar_width: usize = text.chars().map(|c| c.width().unwrap_or(0)).sum();
	if scalar_width != text.width()
		|| text.contains('\u{200d}')
		|| text
			.chars()
			.any(|c| ('\u{1f1e6}'..='\u{1f1ff}').contains(&c))
	{
		std::borrow::Cow::Owned(
			text.chars()
				.map(|c| format!("\\u{{{:x}}}", c as u32))
				.collect(),
		)
	} else if text.width() == 0 {
		std::borrow::Cow::Owned(format!("◌{text}"))
	} else {
		std::borrow::Cow::Borrowed(text)
	}
}

pub fn display_width(text: &str) -> usize {
	display_grapheme(text).width().max(1)
}

pub fn glyphs(line: &str) -> impl Iterator<Item = Glyph<'_>> {
	let mut column = 0;
	line.grapheme_indices(true).map(move |(byte, text)| {
		let width = if text == "\t" {
			TAB_STOP - column % TAB_STOP
		} else {
			display_width(text)
		};
		let glyph = Glyph {
			byte,
			column,
			width,
			text,
		};
		column += width;
		glyph
	})
}

pub fn column_at(line: &str, byte: usize) -> usize {
	glyphs(line)
		.take_while(|glyph| glyph.byte < byte)
		.map(|glyph| glyph.width)
		.sum()
}

pub fn byte_at_column(line: &str, column: usize) -> usize {
	// A click inside a tab or wide grapheme lands at its leading boundary.
	glyphs(line)
		.find(|glyph| column < glyph.column + glyph.width)
		.map_or(line.len(), |glyph| glyph.byte)
}

pub fn sanitize(text: &str) -> String {
	let mut result = String::new();
	let mut chars = text.chars().peekable();
	while let Some(ch) = chars.next() {
		match ch {
			'\r' => {
				if chars.peek() == Some(&'\n') {
					chars.next();
				}
				result.push('\n');
			}
			'\n' | '\t' => result.push(ch),
			ch if ch.is_control() => result.extend(ch.escape_default()),
			ch => result.push(ch),
		}
	}
	result
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ambiguous_clusters_have_a_display_only_ascii_fallback() {
		for text in ["👩🏽‍💻", "🇺🇳", "🏽", "❤️", "1️⃣", "\u{200d}"] {
			let display = display_grapheme(text);
			if text != "🏽" {
				assert!(display.is_ascii(), "{text:?}: {display:?}");
				assert!(display.starts_with("\\u{"));
			}
			assert_eq!(display_width(text), display.width().max(1));
		}
		for text in ["é", "e\u{301}", "界", "👩", "a"] {
			assert_eq!(display_grapheme(text), text);
		}
		assert_eq!(display_grapheme("\u{301}"), "◌\u{301}");
	}

	#[test]
	fn display_mapping_handles_tabs_wide_and_combining_text() {
		let line = "a\t界e\u{301}";
		assert_eq!(column_at(line, line.len()), 7);
		assert_eq!(byte_at_column(line, 3), 1);
		assert_eq!(byte_at_column(line, 5), 2);
		assert_eq!(byte_at_column(line, 6), 5);
		assert_eq!(byte_at_column(line, 100), line.len());
		for glyph in glyphs(line) {
			assert_eq!(
				byte_at_column(line, column_at(line, glyph.byte)),
				glyph.byte
			);
		}
	}

	#[test]
	fn paste_normalizes_newlines_and_makes_controls_visible() {
		assert_eq!(
			sanitize("a\r\nb\rc\t\n\x1b[31m\0\u{85}"),
			"a\nb\nc\t\n\\u{1b}[31m\\u{0}\\u{85}"
		);
	}
}
