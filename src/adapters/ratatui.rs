//! Translate authored styles into terminal rendering; never used for serialization.
use crate::libraries::ansi::{Ink, TextStyle};
use ratatui::style::{Color, Modifier, Style};

pub fn ink_color(ink: Ink) -> Color {
	match ink {
		Ink::Default => Color::Reset,
		Ink::Ansi(n) | Ink::Indexed(n) => Color::Indexed(n),
		Ink::Rgb(r, g, b) => Color::Rgb(r, g, b),
	}
}

pub fn text_style(value: TextStyle) -> Style {
	let mut style = Style::default()
		.fg(ink_color(value.foreground))
		.bg(ink_color(value.background));
	for (enabled, modifier) in [
		(value.bold, Modifier::BOLD),
		(value.italic, Modifier::ITALIC),
		(value.underline, Modifier::UNDERLINED),
		(value.dim, Modifier::DIM),
		(value.strike, Modifier::CROSSED_OUT),
	] {
		if enabled {
			style = style.add_modifier(modifier);
		}
	}
	style
}
