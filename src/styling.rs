use crate::document::Document;
use ratatui::style::{Color, Modifier, Style};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Ink {
	#[default]
	Default,
	Ansi(u8),
	Indexed(u8),
	Rgb(u8, u8, u8),
}

impl Ink {
	pub fn render(self) -> Color {
		match self {
			Self::Default => Color::Reset,
			Self::Ansi(n) | Self::Indexed(n) => Color::Indexed(n),
			Self::Rgb(r, g, b) => Color::Rgb(r, g, b),
		}
	}
	fn code(self, background: bool) -> Option<String> {
		let base = if background { 48 } else { 38 };
		match self {
			Self::Default => None, // Each transition begins with SGR 0.
			Self::Ansi(n) => Some(
				(if n < 8 {
					base - 8 + n
				} else {
					base + 52 + n - 8
				})
				.to_string(),
			),
			Self::Indexed(n) => Some(format!("{base};5;{n}")),
			Self::Rgb(r, g, b) => Some(format!("{base};2;{r};{g};{b}")),
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
	Bold,
	Italic,
	Underline,
	Dim,
	Strike,
	Clear,
	Foreground(Ink),
	Background(Ink),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextStyle {
	pub bold: bool,
	pub italic: bool,
	pub underline: bool,
	pub dim: bool,
	pub strike: bool,
	pub foreground: Ink,
	pub background: Ink,
}

impl TextStyle {
	pub fn matches(self, action: Action) -> bool {
		match action {
			Action::Bold => self.bold,
			Action::Italic => self.italic,
			Action::Underline => self.underline,
			Action::Dim => self.dim,
			Action::Strike => self.strike,
			Action::Clear => self == Self::default(),
			Action::Foreground(ink) => self.foreground == ink,
			Action::Background(ink) => self.background == ink,
		}
	}
	pub fn set(&mut self, action: Action, enabled: bool) {
		match action {
			Action::Bold => self.bold = enabled,
			Action::Italic => self.italic = enabled,
			Action::Underline => self.underline = enabled,
			Action::Dim => self.dim = enabled,
			Action::Strike => self.strike = enabled,
			Action::Clear => *self = Self::default(),
			Action::Foreground(ink) => self.foreground = ink,
			Action::Background(ink) => self.background = ink,
		}
	}
	pub fn render(self) -> Style {
		let mut style = Style::default()
			.fg(self.foreground.render())
			.bg(self.background.render());
		for (enabled, modifier) in [
			(self.bold, Modifier::BOLD),
			(self.italic, Modifier::ITALIC),
			(self.underline, Modifier::UNDERLINED),
			(self.dim, Modifier::DIM),
			(self.strike, Modifier::CROSSED_OUT),
		] {
			if enabled {
				style = style.add_modifier(modifier);
			}
		}
		style
	}
	fn ansi(self) -> String {
		let mut codes = vec!["0".to_owned()];
		for (enabled, code) in [
			(self.bold, "1"),
			(self.dim, "2"),
			(self.italic, "3"),
			(self.underline, "4"),
			(self.strike, "9"),
		] {
			if enabled {
				codes.push(code.into());
			}
		}
		codes.extend(self.foreground.code(false));
		codes.extend(self.background.code(true));
		format!("\x1b[{}m", codes.join(";"))
	}
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
	pub start: usize,
	pub end: usize,
	pub style: TextStyle,
}

/// Contiguous, normalized byte ranges covering the document (including newlines).
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Styles {
	pub runs: Vec<Run>,
}

impl Styles {
	pub fn at(&self, byte: usize) -> TextStyle {
		self.runs
			.iter()
			.find(|run| run.start <= byte && byte < run.end)
			.map_or(TextStyle::default(), |run| run.style)
	}
	pub fn push(&mut self, start: usize, end: usize, style: TextStyle) {
		if start >= end {
			return;
		}
		if let Some(last) = self.runs.last_mut()
			&& last.end == start
			&& last.style == style
		{
			last.end = end;
			return;
		}
		self.runs.push(Run { start, end, style });
	}
	pub fn insert(&mut self, at: usize, count: usize, style: TextStyle) {
		let mut next = Self::default();
		for run in &self.runs {
			next.push(run.start, run.end.min(at), run.style);
		}
		next.push(at, at + count, style);
		for run in &self.runs {
			if run.end > at {
				next.push(run.start.max(at) + count, run.end + count, run.style);
			}
		}
		*self = next;
	}
	pub fn remove(&mut self, start: usize, end: usize) {
		let mut next = Self::default();
		for run in &self.runs {
			next.push(run.start, run.end.min(start), run.style);
			if run.end > end {
				next.push(
					run.start.max(end) - (end - start),
					run.end - (end - start),
					run.style,
				);
			}
		}
		*self = next;
	}
	pub fn apply(&mut self, start: usize, end: usize, change: impl Fn(TextStyle) -> TextStyle) {
		let mut next = Self::default();
		for run in &self.runs {
			next.push(run.start, run.end.min(start), run.style);
			next.push(run.start.max(start), run.end.min(end), change(run.style));
			next.push(run.start.max(end), run.end, run.style);
		}
		*self = next;
	}
}

pub fn ansi(document: &Document) -> String {
	let text = document.lines.join("\n");
	let mut output = String::from("\x1b[0m");
	let mut previous = TextStyle::default();
	for run in &document.styles.runs {
		if run.style != previous {
			output.push_str(&run.style.ansi());
			previous = run.style;
		}
		output.push_str(&text[run.start..run.end]);
	}
	output.push_str("\x1b[0m");
	output
}

pub fn clipboard_text(document: &Document) -> String {
	code(document).replace('\n', "")
}

pub fn code(document: &Document) -> String {
	let mut output = String::new();
	for ch in ansi(document).chars() {
		match ch {
			'\x1b' => output.push_str("\\x1b"),
			'\n' => output.push_str("\\n\n"),
			'\t' => output.push_str("\\t"),
			'\\' => output.push_str("\\\\"),
			ch => output.push(ch),
		}
	}
	output
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::document::Position;

	#[test]
	fn all_color_modes_and_attribute_resets_generate_correct_sgr() {
		for (ink, fg, bg) in [
			(Ink::Default, "", ""),
			(Ink::Ansi(0), ";30", ";40"),
			(Ink::Ansi(7), ";37", ";47"),
			(Ink::Ansi(8), ";90", ";100"),
			(Ink::Ansi(15), ";97", ";107"),
			(Ink::Indexed(0), ";38;5;0", ";48;5;0"),
			(Ink::Indexed(255), ";38;5;255", ";48;5;255"),
			(Ink::Rgb(0, 128, 255), ";38;2;0;128;255", ";48;2;0;128;255"),
		] {
			let mut style = TextStyle {
				foreground: ink,
				background: ink,
				bold: true,
				dim: true,
				strike: true,
				..Default::default()
			};
			assert_eq!(style.ansi(), format!("\x1b[0;1;2;9{fg}{bg}m"));
			style.set(Action::Bold, false);
			assert_eq!(style.ansi(), format!("\x1b[0;2;9{fg}{bg}m"));
			style.set(Action::Bold, true);
			style.set(Action::Dim, false);
			assert_eq!(style.ansi(), format!("\x1b[0;1;9{fg}{bg}m"));
			style.set(Action::Foreground(Ink::Default), true);
			assert_eq!(style.ansi(), format!("\x1b[0;1;9{bg}m"));
			style.set(Action::Clear, true);
			assert_eq!(style, TextStyle::default());
		}
	}

	#[test]
	fn clipboard_text_has_escaped_newlines_without_presentation_rows() {
		let mut doc = Document::default();
		doc.insert(Position::default(), "a\n\tb\\n\n");
		assert_eq!(clipboard_text(&doc), "\\x1b[0ma\\n\\tb\\\\n\\n\\x1b[0m");
		assert!(!clipboard_text(&doc).contains('\n'));
		assert_eq!(code(&doc).lines().count(), 3);
	}

	#[test]
	fn ansi_transitions_reset_and_preserve_newlines_and_literal_escapes() {
		let mut doc = Document::default();
		doc.insert(Position::default(), "Hi\n\t\\n");
		doc.styles.apply(0, 2, |mut style| {
			style.bold = true;
			style.foreground = Ink::Ansi(6);
			style
		});
		assert_eq!(ansi(&doc), "\x1b[0m\x1b[0;1;36mHi\x1b[0m\n\t\\n\x1b[0m");
		assert_eq!(
			code(&doc),
			"\\x1b[0m\\x1b[0;1;36mHi\\x1b[0m\\n\n\\t\\\\n\\x1b[0m"
		);
	}

	#[test]
	fn ranges_remain_contiguous_and_merge_after_overlapping_operations() {
		let mut styles = Styles::default();
		styles.insert(0, 10, TextStyle::default());
		styles.apply(2, 8, |mut style| {
			style.bold = true;
			style
		});
		styles.apply(4, 10, |mut style| {
			style.italic = true;
			style
		});
		assert!(styles.at(5).bold && styles.at(5).italic);
		styles.remove(1, 9);
		assert_eq!(styles.runs.last().unwrap().end, 2);
		styles.apply(0, 2, |_| TextStyle::default());
		assert_eq!(styles.runs.len(), 1);
		styles.insert(
			1,
			3,
			TextStyle {
				underline: true,
				..Default::default()
			},
		);
		assert_eq!(
			styles
				.runs
				.iter()
				.map(|run| (run.start, run.end))
				.collect::<Vec<_>>(),
			[(0, 1), (1, 4), (4, 5)]
		);
	}
}
