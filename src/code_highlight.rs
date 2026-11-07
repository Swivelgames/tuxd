//! View-only tokens for our escaped ANSI representation, never export markup.
use std::ops::Range;

use ratatui::style::{Color, Modifier, Style};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
	Sequence,
	Parameter,
	Reset,
	Separator,
	Whitespace,
}

#[derive(Debug)]
struct Token {
	bytes: Range<usize>,
	kind: Kind,
}

#[derive(Default, Debug)]
pub struct Highlight(Vec<Token>);

impl Highlight {
	pub fn new(line: &str) -> Self {
		let bytes = line.as_bytes();
		let mut tokens = Vec::new();
		let mut at = 0;
		while at < bytes.len() {
			if bytes[at] != b'\\' {
				at += 1;
				continue;
			}
			// Literal backslashes are doubled by the serializer. Consume both so
			// document text such as "\\x1b[31m" cannot masquerade as a style sequence.
			if bytes.get(at + 1) == Some(&b'\\') {
				at += 2;
				continue;
			}
			if matches!(bytes.get(at + 1), Some(b'n' | b't')) {
				tokens.push(Token {
					bytes: at..at + 2,
					kind: Kind::Whitespace,
				});
				at += 2;
				continue;
			}
			if bytes[at..].starts_with(b"\\x1b[") {
				let start = at + 5;
				let mut end = start;
				while bytes
					.get(end)
					.is_some_and(|b| b.is_ascii_digit() || *b == b';')
				{
					end += 1;
				}
				if end > start && bytes.get(end) == Some(&b'm') {
					tokens.push(Token {
						bytes: at..start,
						kind: Kind::Sequence,
					});
					let mut parameter = start;
					while parameter < end {
						let mut next = parameter;
						while next < end && bytes[next] != b';' {
							next += 1;
						}
						if next > parameter {
							let kind = if parameter == start && &bytes[parameter..next] == b"0" {
								Kind::Reset
							} else {
								Kind::Parameter
							};
							tokens.push(Token {
								bytes: parameter..next,
								kind,
							});
						}
						if next < end {
							tokens.push(Token {
								bytes: next..next + 1,
								kind: Kind::Separator,
							});
						}
						parameter = next + 1;
					}
					tokens.push(Token {
						bytes: end..end + 1,
						kind: Kind::Sequence,
					});
					at = end + 1;
					continue;
				}
			}
			at += 1;
		}
		Self(tokens)
	}

	pub fn style_at(&self, byte: usize) -> Style {
		let index = self.0.partition_point(|token| token.bytes.end <= byte);
		let Some(token) = self
			.0
			.get(index)
			.filter(|token| token.bytes.contains(&byte))
		else {
			return Style::default();
		};
		match token.kind {
			Kind::Sequence => Style::default().fg(Color::Cyan),
			Kind::Parameter => Style::default().fg(Color::Yellow),
			Kind::Reset => Style::default()
				.fg(Color::Magenta)
				.add_modifier(Modifier::BOLD),
			Kind::Separator => Style::default().fg(Color::DarkGray),
			Kind::Whitespace => Style::default().fg(Color::Blue),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn literal_escape_like_text_stays_plain() {
		let line = r"\\x1b[31m \\n \\t \\u{1f469} café 界";
		let highlight = Highlight::new(line);
		assert!(highlight.0.is_empty());
		for byte in 0..line.len() {
			assert_eq!(highlight.style_at(byte), Style::default());
		}
	}

	#[test]
	fn reset_is_distinct_from_rgb_zero_and_tokens_use_byte_offsets() {
		let line = "界\\x1b[0;38;2;0;128;255mtext\\n\\t";
		let highlight = Highlight::new(line);
		assert_eq!(
			highlight.0.iter().filter(|t| t.kind == Kind::Reset).count(),
			1
		);
		assert_eq!(highlight.style_at(3).fg, Some(Color::Cyan));
		assert_eq!(
			highlight.style_at(line.find(";0;").unwrap() + 1).fg,
			Some(Color::Yellow)
		);
		assert_eq!(
			highlight.style_at(line.find("text").unwrap()),
			Style::default()
		);
		assert_eq!(highlight.style_at(line.len() - 1).fg, Some(Color::Blue));
		for token in &highlight.0 {
			assert!(line.is_char_boundary(token.bytes.start));
			assert!(line.is_char_boundary(token.bytes.end));
		}
	}

	#[test]
	fn malformed_or_non_sgr_text_is_not_highlighted() {
		for line in [r"\x1b[2J", r"\x1b[38;2", r"\x1b[watm"] {
			assert!(Highlight::new(line).0.is_empty());
		}
	}
}
