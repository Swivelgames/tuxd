//! Import styled text, never terminal screen operations. All input is inert data.
use crate::{
	document::Document,
	styling::{Ink, Styles, TextStyle},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
	Raw,
	Escaped,
}

#[derive(Debug)]
pub struct Issue {
	pub start: usize,
	pub end: usize,
	pub line: usize,
	pub column: usize,
	pub notation: String,
	pub reason: String,
}

pub struct Import {
	pub source: String,
	pub document: Document,
	pub issues: Vec<Issue>,
}

#[derive(Clone, Copy)]
struct Unit {
	ch: char,
	start: usize,
	end: usize,
}

fn issue(source: &str, start: usize, end: usize, reason: impl Into<String>) -> Issue {
	Issue {
		start,
		end,
		line: 0,
		column: 0,
		notation: source[start..end]
			.chars()
			.flat_map(char::escape_default)
			.collect(),
		reason: reason.into(),
	}
}

fn units(source: &str, mode: Mode, issues: &mut Vec<Issue>) -> Vec<Unit> {
	let mut output = Vec::new();
	let mut at = 0;
	while at < source.len() {
		let start = at;
		let ch = source[at..].chars().next().unwrap();
		at += ch.len_utf8();
		if mode == Mode::Raw || ch != '\\' {
			output.push(Unit { ch, start, end: at });
			continue;
		}
		let next = source[at..].chars().next();
		let decoded = match next {
			Some('\\') => Some('\\'),
			Some('n') => Some('\n'),
			Some('t') => Some('\t'),
			Some('x') => {
				at += 1;
				let hex_start = at;
				while at < source.len()
					&& at - hex_start < 2
					&& source.as_bytes()[at].is_ascii_hexdigit()
				{
					at += 1;
				}
				if at - hex_start == 2 && &source[hex_start..at].to_ascii_lowercase() == "1b" {
					output.push(Unit {
						ch: '\x1b',
						start,
						end: at,
					});
				} else {
					issues.push(issue(source, start, at, "Expected \\x1b for ESC"));
				}
				continue;
			}
			_ => None,
		};
		if let Some(next) = next {
			at += next.len_utf8();
		}
		if let Some(ch) = decoded {
			output.push(Unit { ch, start, end: at });
		} else {
			issues.push(issue(
				source,
				start,
				at,
				"Unsupported or incomplete escaped notation",
			));
		}
	}
	output
}

fn sgr(parameters: &str, original: TextStyle) -> Result<TextStyle, String> {
	let values: Vec<u16> = parameters
		.split(';')
		.map(|part| {
			if part.is_empty() {
				Ok(0)
			} else {
				part.parse::<u16>()
			}
		})
		.collect::<Result<_, _>>()
		.map_err(|_| "Malformed SGR parameters".to_owned())?;
	let mut style = original;
	let mut i = 0;
	while i < values.len() {
		let value = values[i];
		i += 1;
		match value {
			0 => style = TextStyle::default(),
			1 => style.bold = true,
			2 => style.dim = true,
			3 => style.italic = true,
			4 => style.underline = true,
			9 => style.strike = true,
			22 => {
				style.bold = false;
				style.dim = false;
			}
			23 => style.italic = false,
			24 => style.underline = false,
			29 => style.strike = false,
			30..=37 => style.foreground = Ink::Ansi((value - 30) as u8),
			40..=47 => style.background = Ink::Ansi((value - 40) as u8),
			90..=97 => style.foreground = Ink::Ansi((value - 90 + 8) as u8),
			100..=107 => style.background = Ink::Ansi((value - 100 + 8) as u8),
			39 => style.foreground = Ink::Default,
			49 => style.background = Ink::Default,
			38 | 48 => {
				let count = match values.get(i) {
					Some(5) => 1,
					Some(2) => 3,
					_ => return Err("Unsupported or incomplete extended color".into()),
				};
				i += 1;
				let channels = values
					.get(i..i + count)
					.ok_or("Incomplete extended color")?;
				if channels.iter().any(|n| *n > 255) {
					return Err("Color components must be 0–255".into());
				}
				let ink = if count == 1 {
					Ink::Indexed(channels[0] as u8)
				} else {
					Ink::Rgb(channels[0] as u8, channels[1] as u8, channels[2] as u8)
				};
				i += count;
				if value == 38 {
					style.foreground = ink;
				} else {
					style.background = ink;
				}
			}
			_ => {
				return Err(format!(
					"Unsupported SGR attribute {value}; strip removes this entire SGR sequence"
				));
			}
		}
	}
	Ok(style)
}

pub fn parse(source: &str, mode: Mode) -> Result<Import, String> {
	let mut issues = Vec::new();
	let input = units(source, mode, &mut issues);
	let mut text = String::new();
	let mut styles = Styles::default();
	let mut style = TextStyle::default();
	let mut at = 0;
	while at < input.len() {
		let first = at;
		let ch = input[at].ch;
		at += 1;
		if ch == '\r' && input.get(at).is_some_and(|u| u.ch == '\n') {
			continue;
		}
		if !ch.is_control() || matches!(ch, '\n' | '\t') {
			let start = text.len();
			text.push(ch);
			styles.push(start, text.len(), style);
			continue;
		}
		let mut reason = "Unsupported control character".to_owned();
		let kind = if ch == '\x1b' {
			input.get(at).map(|u| u.ch)
		} else {
			match ch {
				'\u{9b}' => Some('['),
				'\u{9d}' => Some(']'),
				'\u{90}' => Some('P'),
				'\u{98}' => Some('X'),
				'\u{9e}' => Some('^'),
				'\u{9f}' => Some('_'),
				_ => None,
			}
		};
		if let Some(kind) = kind {
			if ch == '\x1b' && (' '..='~').contains(&input[at].ch) {
				at += 1;
			}
			match kind {
				'[' => {
					let params_start = at;
					while at < input.len() && (' '..='?').contains(&input[at].ch) {
						at += 1;
					}
					if at < input.len() && ('@'..='~').contains(&input[at].ch) {
						let final_byte = input[at].ch;
						at += 1;
						if final_byte == 'm' {
							let parameters: String =
								input[params_start..at - 1].iter().map(|u| u.ch).collect();
							match sgr(&parameters, style) {
								Ok(next) => {
									style = next;
									continue;
								}
								Err(error) => reason = error,
							}
						} else {
							reason =
								"Unsupported CSI operation (only SGR styling is imported)".into();
						}
					} else {
						reason = "Incomplete or malformed CSI; stopped before the next text/control boundary".into();
					}
				}
				']' | 'P' | 'X' | '^' | '_' => {
					let mut terminated = false;
					while at < input.len() {
						if input[at].ch == '\u{9c}' || kind == ']' && input[at].ch == '\x07' {
							at += 1;
							terminated = true;
							break;
						}
						if input[at].ch == '\x1b' && input.get(at + 1).is_some_and(|u| u.ch == '\\')
						{
							at += 2;
							terminated = true;
							break;
						}
						if input[at].ch == '\x1b' {
							break;
						}
						at += 1;
					}
					if !terminated {
						return Err(format!(
							"Unterminated control string at source byte {}. Its end cannot be identified safely; import aborted.",
							input[first].start
						));
					}
					reason = "Unsupported control string (OSC/DCS/SOS/PM/APC)".into();
				}
				_ if ch == '\x1b' => {
					if (' '..='/').contains(&kind) {
						while at < input.len() && (' '..='/').contains(&input[at].ch) {
							at += 1;
						}
						if at < input.len() && ('0'..='~').contains(&input[at].ch) {
							at += 1;
						}
					}
					reason = "Unsupported or malformed ESC operation".into();
				}
				_ => (),
			}
		}
		issues.push(issue(source, input[first].start, input[at - 1].end, reason));
	}
	issues.sort_by_key(|item| item.start);
	let mut positions = source.char_indices().peekable();
	let (mut line, mut column) = (1, 1);
	for issue in &mut issues {
		while positions
			.peek()
			.is_some_and(|(offset, _)| *offset < issue.start)
		{
			let (_, ch) = positions.next().unwrap();
			if ch == '\n' {
				line += 1;
				column = 1;
			} else {
				column += 1;
			}
		}
		issue.line = line;
		issue.column = column;
	}
	let mut document = Document {
		lines: text.split('\n').map(str::to_owned).collect(),
		styles,
	};
	document.normalize_styles();
	Ok(Import {
		source: source.to_owned(),
		document,
		issues,
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{document::Position, styling};
	#[test]
	fn malformed_escape_recovery_preserves_unicode_and_generated_inputs_stay_valid() {
		assert_eq!(
			parse("a\x1b(界\x1b🦀b", Mode::Raw).unwrap().document.lines,
			["a界🦀b"]
		);
		let alphabet = [
			'\x1b', '[', ']', 'P', ';', ':', '0', '9', 'm', '\\', 'x', '1', 'b', '\n', '\t', '\r',
			'\u{9b}', '\u{301}', '界', 'a', '\x07',
		];
		let mut seed = 12345usize;
		for _ in 0..100 {
			let source: String = (0..150)
				.map(|_| {
					seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
					alphabet[seed % alphabet.len()]
				})
				.collect();
			for mode in [Mode::Raw, Mode::Escaped] {
				if let Ok(import) = parse(&source, mode) {
					assert!(crate::persistence::encode(&import.document).is_ok());
					for issue in import.issues {
						assert!(source.get(issue.start..issue.end).is_some());
						assert!(!issue.notation.chars().any(char::is_control));
					}
				}
			}
		}
	}

	#[test]
	fn raw_and_escaped_exports_roundtrip_every_color_kind() {
		for ink in [
			Ink::Default,
			Ink::Ansi(15),
			Ink::Indexed(255),
			Ink::Rgb(0, 128, 255),
		] {
			let mut document = Document::default();
			document.insert(Position::default(), "e\u{301}界\n\t\\x1b[31m👩‍💻\n");
			document.styles.apply(0, 6, |_| TextStyle {
				bold: true,
				dim: true,
				italic: true,
				underline: true,
				strike: true,
				foreground: ink,
				background: ink,
			});
			for (source, mode) in [
				(styling::ansi(&document), Mode::Raw),
				(styling::clipboard_text(&document), Mode::Escaped),
			] {
				let parsed = parse(&source, mode).unwrap();
				assert!(parsed.issues.is_empty());
				assert_eq!(parsed.document, document);
			}
		}
	}
	#[test]
	fn reset_semantics_and_strip_preserve_surrounding_text() {
		let source = "\x1b[1;2;3;4;9;91;48;5;42mA\x1b[22;23;24;29;39;49mB\x1b[2JC\x1b[31\nD\x07界";
		let parsed = parse(source, Mode::Raw).unwrap();
		assert_eq!(parsed.document.lines, ["ABC", "D界"]);
		assert_eq!(parsed.document.styles.at(1), TextStyle::default());
		assert_eq!(parsed.issues.len(), 3);
		assert!(parsed.issues.iter().all(|i| !i.notation.contains('\x1b')));
	}
	#[test]
	fn escaped_diagnostics_point_to_source_and_unknown_sgr_is_atomic() {
		let source = r"a\x1b[1;999mb\q尾";
		let parsed = parse(source, Mode::Escaped).unwrap();
		assert_eq!(parsed.document.lines, ["ab尾"]);
		assert_eq!(parsed.document.styles.at(1), TextStyle::default());
		assert_eq!(
			&source[parsed.issues[0].start..parsed.issues[0].end],
			r"\x1b[1;999m"
		);
		assert_eq!(parsed.issues.len(), 2);
		assert!(parse("\x1b]unclosed title", Mode::Raw).is_err());
		assert_eq!(
			parse("a\x1b]title\x07b", Mode::Raw).unwrap().document.lines,
			["ab"]
		);
	}
}
