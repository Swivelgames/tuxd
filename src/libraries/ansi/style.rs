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

	pub(super) fn ansi(self) -> String {
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

/// Serializable style ranges. Document construction validates coverage and boundaries.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Styles {
	pub runs: Vec<Run>,
}

impl Styles {
	pub(super) fn at(&self, byte: usize) -> TextStyle {
		self.runs
			.iter()
			.find(|run| run.start <= byte && byte < run.end)
			.map_or(TextStyle::default(), |run| run.style)
	}

	pub(super) fn push(&mut self, start: usize, end: usize, style: TextStyle) {
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

	pub(super) fn apply(
		&mut self,
		start: usize,
		end: usize,
		change: impl Fn(TextStyle) -> TextStyle,
	) {
		let mut next = Self::default();
		for run in &self.runs {
			next.push(run.start, run.end.min(start), run.style);
			next.push(run.start.max(start), run.end.min(end), change(run.style));
			next.push(run.start.max(end), run.end, run.style);
		}
		*self = next;
	}
}

impl TextStyle {
	pub(super) fn validate(self) -> Result<(), super::Error> {
		if [self.foreground, self.background]
			.iter()
			.any(|ink| matches!(ink, Ink::Ansi(n) if *n > 15))
		{
			return Err(super::Error::InvalidColor);
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
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
}
