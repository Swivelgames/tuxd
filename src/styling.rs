use crate::document::Document;
use ratatui::style::{Color, Modifier, Style};

pub const LABELS: [&str; 12] = [
	"Bold",
	"Italic",
	"Underline",
	"Default",
	"Black",
	"Red",
	"Green",
	"Yellow",
	"Blue",
	"Magenta",
	"Cyan",
	"White",
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextStyle {
	pub bold: bool,
	pub italic: bool,
	pub underline: bool,
	pub foreground: Option<u8>,
}

impl TextStyle {
	pub fn matches(self, tool: usize) -> bool {
		match tool {
			0 => self.bold,
			1 => self.italic,
			2 => self.underline,
			3 => self.foreground.is_none(),
			_ => self.foreground == Some((tool - 4) as u8),
		}
	}
	pub fn set(&mut self, tool: usize, enabled: bool) {
		match tool {
			0 => self.bold = enabled,
			1 => self.italic = enabled,
			2 => self.underline = enabled,
			3 => self.foreground = None,
			_ => self.foreground = Some((tool - 4) as u8),
		}
	}
	pub fn render(self) -> Style {
		let mut style = Style::default();
		if self.bold {
			style = style.add_modifier(Modifier::BOLD);
		}
		if self.italic {
			style = style.add_modifier(Modifier::ITALIC);
		}
		if self.underline {
			style = style.add_modifier(Modifier::UNDERLINED);
		}
		if let Some(color) = self.foreground {
			style = style.fg([
				Color::Black,
				Color::Red,
				Color::Green,
				Color::Yellow,
				Color::Blue,
				Color::Magenta,
				Color::Cyan,
				Color::Gray,
			][color as usize]);
		}
		style
	}
	fn ansi(self) -> String {
		let mut codes = vec!["0".to_owned()];
		if self.bold {
			codes.push("1".into());
		}
		if self.italic {
			codes.push("3".into());
		}
		if self.underline {
			codes.push("4".into());
		}
		if let Some(color) = self.foreground {
			codes.push((30 + color).to_string());
		}
		format!("\x1b[{}m", codes.join(";"))
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
	pub start: usize,
	pub end: usize,
	pub style: TextStyle,
}

/// Contiguous, normalized byte ranges covering the document (including newlines).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
			style.foreground = Some(6);
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
