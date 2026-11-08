//! Ratatui appearance adapter. `with` supplies only resolved colors for one render scope.
mod storage;
use crate::libraries::themes::Color as UiColor;
pub use crate::libraries::themes::{Group, PRESETS, Role, Theme};
use ratatui::{
	style::{Color, Modifier, Style},
	widgets::{Block, BorderType},
};
use std::cell::RefCell;
pub use storage::Config;

pub fn color(value: UiColor) -> Color {
	match value {
		UiColor::Reset => Color::Reset,
		UiColor::Indexed(n) => Color::Indexed(n),
		UiColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
		UiColor::DarkGray => Color::DarkGray,
		UiColor::White => Color::White,
	}
}
thread_local! { static ACTIVE: RefCell<Theme> = RefCell::new(Theme::default()); }
/// Apply a theme for one render scope, restoring it even if rendering unwinds.
pub fn with<R>(theme: &Theme, render: impl FnOnce() -> R) -> R {
	struct Restore(Theme);
	impl Drop for Restore {
		fn drop(&mut self) {
			ACTIVE.with(|active| *active.borrow_mut() = self.0.clone());
		}
	}
	let _restore = Restore(ACTIVE.with(|active| active.replace(theme.clone())));
	render()
}
pub fn accent_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().accent))
}
pub fn surface_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().surface))
}
pub fn text_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().text))
}
pub fn border_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().border))
}
pub fn muted_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().muted))
}
pub fn field_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().field))
}
pub fn selected_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().selected))
}
pub fn selected_text_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().selected_text))
}
pub fn focus_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().focus))
}
pub fn inactive_indicator_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().inactive_indicator))
}
pub fn error_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().error))
}
pub fn selection_overlay_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().selection_overlay))
}
pub fn range_text_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().range_text))
}
pub fn syntax_sequence_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().syntax_sequence))
}
pub fn syntax_parameter_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().syntax_parameter))
}
pub fn syntax_escape_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().syntax_escape))
}
pub fn syntax_separator_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().syntax_separator))
}
pub fn syntax_whitespace_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().syntax_whitespace))
}
pub fn rgb_red_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().rgb_red))
}
pub fn rgb_green_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().rgb_green))
}
pub fn rgb_blue_color() -> Color {
	ACTIVE.with(|theme| color(theme.borrow().rgb_blue))
}

pub fn accent() -> Style {
	Style::default()
		.fg(accent_color())
		.add_modifier(Modifier::BOLD)
}
pub fn muted() -> Style {
	Style::default().fg(muted_color())
}
pub fn block<'a>() -> Block<'a> {
	Block::bordered()
		.style(Style::default().fg(text_color()).bg(surface_color()))
		.border_type(BorderType::Rounded)
		.border_style(Style::default().fg(border_color()))
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn midnight_is_default_and_baseline_is_not_supported() {
		assert_eq!(Config::default().preset, "midnight");
		assert_eq!(Config::default().resolve().unwrap(), Theme::default());
		assert!(!PRESETS.iter().any(|(id, _)| *id == "baseline"));
		assert!(Theme::preset("baseline").is_err());
	}

	#[test]
	fn presets_persist_and_explicit_palette_text_has_contrast() {
		fn luminance(color: UiColor) -> f64 {
			let UiColor::Rgb(r, g, b) = color else {
				panic!("explicit preset color expected")
			};
			let channel = |n: u8| {
				let s = n as f64 / 255.0;
				if s <= 0.04045 {
					s / 12.92
				} else {
					((s + 0.055) / 1.055).powf(2.4)
				}
			};
			0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
		}
		for (id, _) in PRESETS {
			let config = crate::libraries::themes::Config {
				preset: (*id).into(),
				..Default::default()
			};
			let theme = config.resolve().unwrap();
			let roundtrip: crate::libraries::themes::Config =
				serde_json::from_str(&serde_json::to_string(&config).unwrap()).unwrap();
			assert_eq!(roundtrip.resolve().unwrap(), theme);
			if !matches!(*id, "midnight" | "light") {
				continue;
			}
			for (fg, bg) in [
				(theme.text, theme.surface),
				(theme.text, theme.field),
				(theme.selected_text, theme.selected),
				(theme.muted, theme.surface),
				(theme.accent, theme.selected),
				(theme.error, theme.surface),
			] {
				let (a, b) = (luminance(fg), luminance(bg));
				assert!(
					(a.max(b) + 0.05) / (a.min(b) + 0.05) >= 4.5,
					"{id}: {fg:?} on {bg:?}"
				);
			}
		}
	}

	#[test]
	fn configuration_roundtrip_and_failed_save_preserve_file() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("theme.json");
		assert_eq!(Config::read(&path).unwrap(), Config::default());
		assert!(!path.exists());
		let mut config = Config::default();
		config.colors.insert("accent".into(), "#123456".into());
		config.colors.insert("surface".into(), "ansi:17".into());
		config.write(&path).unwrap();
		assert_eq!(Config::read(&path).unwrap(), config);
		let theme = config.resolve().unwrap();
		assert_eq!(theme.accent, UiColor::Rgb(18, 52, 86));
		assert_eq!(theme.surface, UiColor::Indexed(17));
		assert_eq!(theme.border, Theme::default().border);
		let saved = std::fs::read(&path).unwrap();
		config.colors.insert("unknown".into(), "default".into());
		assert!(config.write(&path).is_err());
		assert_eq!(std::fs::read(&path).unwrap(), saved);
	}
	#[test]
	fn invalid_configuration_and_scoped_activation() {
		for value in ["ansi:256", "#12345", "#zzzzzz", "red", "", "#１２３"] {
			assert!(
				crate::libraries::themes::Config {
					colors: [("accent".into(), value.into())].into(),
					..Default::default()
				}
				.resolve()
				.is_err()
			);
		}
		let base = accent_color();
		let custom = Theme {
			accent: UiColor::Indexed(2),
			..Theme::default()
		};
		with(&custom, || {
			assert_eq!(accent_color(), Color::Indexed(2));
		});
		assert_eq!(accent_color(), base);
		let _ = std::panic::catch_unwind(|| with(&custom, || panic!("render failure")));
		assert_eq!(accent_color(), base);
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("theme.json");
		for input in [
			r#"{"version":2,"preset":"baseline"}"#,
			r#"{"version":1,"preset":"missing"}"#,
			r#"{"version":1,"preset":"baseline","oops":1}"#,
			"{",
		] {
			std::fs::write(&path, input).unwrap();
			assert!(Config::read(&path).is_err());
			assert_eq!(std::fs::read_to_string(&path).unwrap(), input);
		}
	}
}
