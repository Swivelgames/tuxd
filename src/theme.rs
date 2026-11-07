//! Runtime application themes; Midnight is the default. Document formatting and swatch colors are content,
//! and must never be substituted with these UI roles.
use ratatui::{
	style::{Color, Modifier, Style},
	widgets::{Block, BorderType},
};

use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::BTreeMap, io::Write, path::Path};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Theme {
	pub accent: Color,
	pub surface: Color,
	pub text: Color,
	pub border: Color,
	pub muted: Color,
	pub field: Color,
	pub selected: Color,
	pub selected_text: Color,
	pub focus: Color,
	pub inactive_indicator: Color,
	pub error: Color,
	pub selection_overlay: Color,
	pub range_text: Color,
	pub syntax_sequence: Color,
	pub syntax_parameter: Color,
	pub syntax_escape: Color,
	pub syntax_separator: Color,
	pub syntax_whitespace: Color,
	pub rgb_red: Color,
	pub rgb_green: Color,
	pub rgb_blue: Color,
}
impl Theme {
	pub fn color(&self, role: &str) -> Color {
		match role {
			"accent" => self.accent,
			"surface" => self.surface,
			"text" => self.text,
			"border" => self.border,
			"muted" => self.muted,
			"field" => self.field,
			"selected" => self.selected,
			"selected_text" => self.selected_text,
			"focus" => self.focus,
			"inactive_indicator" => self.inactive_indicator,
			"error" => self.error,
			"selection_overlay" => self.selection_overlay,
			"range_text" => self.range_text,
			"syntax_sequence" => self.syntax_sequence,
			"syntax_parameter" => self.syntax_parameter,
			"syntax_escape" => self.syntax_escape,
			"syntax_separator" => self.syntax_separator,
			"syntax_whitespace" => self.syntax_whitespace,
			"rgb_red" => self.rgb_red,
			"rgb_green" => self.rgb_green,
			"rgb_blue" => self.rgb_blue,
			_ => Color::Reset,
		}
	}
}
impl Default for Theme {
	fn default() -> Self {
		let rgb = rgb;
		Self {
			accent: rgb(0x8bd5ca),
			surface: rgb(0x161923),
			text: rgb(0xe4e9f2),
			border: rgb(0x738299),
			muted: rgb(0xa0adc2),
			field: rgb(0x242b3b),
			selected: rgb(0x354660),
			selected_text: rgb(0xf4f7ff),
			focus: rgb(0x435778),
			inactive_indicator: rgb(0x738299),
			error: rgb(0xff8c9b),
			syntax_sequence: rgb(0x8bd5ca),
			syntax_parameter: rgb(0xecc98b),
			syntax_escape: rgb(0xd6a6ed),
			syntax_separator: rgb(0xa0adc2),
			syntax_whitespace: rgb(0x91bcf0),
			selection_overlay: Color::DarkGray,
			range_text: Color::White,
			rgb_red: rgb(0x732d37),
			rgb_green: rgb(0x235f41),
			rgb_blue: rgb(0x284682),
		}
	}
}
fn rgb(hex: u32) -> Color {
	Color::Rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}
// Upstream palette sources and role adaptations: .work/initiatives/themes/presets.md.
fn palette(
	[
		surface,
		field,
		text,
		muted,
		selected,
		focus,
		accent,
		parameter,
		escape,
		error,
	]: [u32; 10],
) -> Theme {
	Theme {
		surface: rgb(surface),
		field: rgb(field),
		text: rgb(text),
		muted: rgb(muted),
		border: rgb(muted),
		selected: rgb(selected),
		selected_text: rgb(text),
		focus: rgb(focus),
		inactive_indicator: rgb(muted),
		accent: rgb(accent),
		error: rgb(error),
		syntax_sequence: rgb(accent),
		syntax_parameter: rgb(parameter),
		syntax_escape: rgb(escape),
		syntax_separator: rgb(muted),
		syntax_whitespace: rgb(accent),
		..Theme::default()
	}
}
pub const PRESETS: &[(&str, &str)] = &[
	("midnight", "Midnight"),
	("light", "Light"),
	("one-dark", "One Dark"),
	("dracula", "Dracula"),
	("nord", "Nord"),
	("gruvbox-dark", "Gruvbox Dark"),
	("gruvbox-light", "Gruvbox Light"),
	("solarized-dark", "Solarized Dark"),
	("solarized-light", "Solarized Light"),
	("tokyo-night", "Tokyo Night"),
	("catppuccin-latte", "Catppuccin Latte"),
	("catppuccin-frappe", "Catppuccin Frappé"),
	("catppuccin-macchiato", "Catppuccin Macchiato"),
	("catppuccin-mocha", "Catppuccin Mocha"),
];
impl Theme {
	pub fn preset(id: &str) -> Result<Self, String> {
		Ok(match id {
			"midnight" => Self::default(),
			"light" => Self {
				accent: rgb(0x005a80),
				surface: rgb(0xf4f6fa),
				text: rgb(0x202938),
				border: rgb(0x68758a),
				muted: rgb(0x505e73),
				field: rgb(0xe4eaf2),
				selected: rgb(0xc2d9ee),
				selected_text: rgb(0x162b40),
				focus: rgb(0xb0cde6),
				inactive_indicator: rgb(0x68758a),
				error: rgb(0xa21b32),
				syntax_sequence: rgb(0x005a80),
				syntax_parameter: rgb(0x775000),
				syntax_escape: rgb(0x79358b),
				syntax_separator: rgb(0x505e73),
				syntax_whitespace: rgb(0x215da8),
				..Self::default()
			},
			"one-dark" => palette([
				0x282c34, 0x21252b, 0xabb2bf, 0xabb2bf, 0x3e4451, 0x4b5263, 0x61afef, 0xe5c07b,
				0xc678dd, 0xe06c75,
			]),
			"dracula" => palette([
				0x282a36, 0x282a36, 0xf8f8f2, 0xbd93f9, 0x44475a, 0x44475a, 0x8be9fd, 0xf1fa8c,
				0xbd93f9, 0xff5555,
			]),
			"nord" => palette([
				0x2e3440, 0x3b4252, 0xeceff4, 0xd8dee9, 0x434c5e, 0x4c566a, 0x88c0d0, 0xebcb8b,
				0xb48ead, 0xbf616a,
			]),
			"gruvbox-dark" => palette([
				0x282828, 0x3c3836, 0xebdbb2, 0xbdae93, 0x504945, 0x665c54, 0x8ec07c, 0xfabd2f,
				0xd3869b, 0xfb4934,
			]),
			"gruvbox-light" => palette([
				0xfbf1c7, 0xebdbb2, 0x3c3836, 0x665c54, 0xd5c4a1, 0xbdae93, 0x76678, 0xb57614,
				0x8f3f71, 0x9d0006,
			]),
			"solarized-dark" => palette([
				0x2b36, 0x73642, 0x93a1a1, 0x839496, 0x73642, 0x73642, 0x2aa198, 0xb58900,
				0x6c71c4, 0xdc322f,
			]),
			"solarized-light" => palette([
				0xfdf6e3, 0xeee8d5, 0x586e75, 0x657b83, 0xeee8d5, 0xeee8d5, 0x268bd2, 0xb58900,
				0x6c71c4, 0xdc322f,
			]),
			"tokyo-night" => palette([
				0x1a1b26, 0x16161e, 0xc0caf5, 0xa9b1d6, 0x292e42, 0x394b70, 0x7aa2f7, 0xe0af68,
				0xbb9af7, 0xf7768e,
			]),
			"catppuccin-latte" => palette([
				0xeff1f5, 0xe6e9ef, 0x4c4f69, 0x6c6f85, 0xccd0da, 0xbcc0cc, 0x1e66f5, 0xdf8e1d,
				0x8839ef, 0xd20f39,
			]),
			"catppuccin-frappe" => palette([
				0x303446, 0x292c3c, 0xc6d0f5, 0xa5adce, 0x414559, 0x51576d, 0x8caaee, 0xe5c890,
				0xca9ee6, 0xe78284,
			]),
			"catppuccin-macchiato" => palette([
				0x24273a, 0x1e2030, 0xcad3f5, 0xa5adcb, 0x363a4f, 0x494d64, 0x8aadf4, 0xeed49f,
				0xc6a0f6, 0xed8796,
			]),
			"catppuccin-mocha" => palette([
				0x1e1e2e, 0x181825, 0xcdd6f4, 0xa6adc8, 0x313244, 0x45475a, 0x89b4fa, 0xf9e2af,
				0xcba6f7, 0xf38ba8,
			]),
			_ => return Err(format!("Unknown theme preset {id}")),
		})
	}
}
pub const ROLES: &[(&str, &str)] = &[
	("accent", "Accent"),
	("surface", "Background"),
	("text", "Text"),
	("border", "Border"),
	("muted", "Muted text"),
	("field", "Field background"),
	("selected", "Selected background"),
	("selected_text", "Selected text"),
	("focus", "Focused background"),
	("inactive_indicator", "Inactive indicator"),
	("error", "Error"),
	("selection_overlay", "Selection overlay"),
	("range_text", "Range text"),
	("syntax_sequence", "Sequences"),
	("syntax_parameter", "Parameters"),
	("syntax_escape", "Escapes"),
	("syntax_separator", "Separators"),
	("syntax_whitespace", "Whitespace"),
	("rgb_red", "Red"),
	("rgb_green", "Green"),
	("rgb_blue", "Blue"),
];
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
	ACTIVE.with(|theme| theme.borrow().accent)
}
pub fn surface_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().surface)
}
pub fn text_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().text)
}
pub fn border_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().border)
}
pub fn muted_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().muted)
}
pub fn field_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().field)
}
pub fn selected_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().selected)
}
pub fn selected_text_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().selected_text)
}
pub fn focus_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().focus)
}
pub fn inactive_indicator_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().inactive_indicator)
}
pub fn error_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().error)
}
pub fn selection_overlay_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().selection_overlay)
}
pub fn range_text_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().range_text)
}
pub fn syntax_sequence_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().syntax_sequence)
}
pub fn syntax_parameter_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().syntax_parameter)
}
pub fn syntax_escape_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().syntax_escape)
}
pub fn syntax_separator_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().syntax_separator)
}
pub fn syntax_whitespace_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().syntax_whitespace)
}
pub fn rgb_red_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().rgb_red)
}
pub fn rgb_green_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().rgb_green)
}
pub fn rgb_blue_color() -> Color {
	ACTIVE.with(|theme| theme.borrow().rgb_blue)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
	pub version: u32,
	pub preset: String,
	#[serde(default)]
	pub colors: BTreeMap<String, String>,
}
impl Default for Config {
	fn default() -> Self {
		Self {
			version: 1,
			preset: "midnight".into(),
			colors: BTreeMap::new(),
		}
	}
}
impl Config {
	pub fn resolve(&self) -> Result<Theme, String> {
		if self.version != 1 {
			return Err(format!("Unsupported theme version {}", self.version));
		}
		let mut theme = Theme::preset(&self.preset)?;
		for (role, value) in &self.colors {
			let color = parse_color(value)?;
			match role.as_str() {
				"accent" => theme.accent = color,
				"surface" => theme.surface = color,
				"text" => theme.text = color,
				"border" => theme.border = color,
				"muted" => theme.muted = color,
				"field" => theme.field = color,
				"selected" => theme.selected = color,
				"selected_text" => theme.selected_text = color,
				"focus" => theme.focus = color,
				"inactive_indicator" => theme.inactive_indicator = color,
				"error" => theme.error = color,
				"selection_overlay" => theme.selection_overlay = color,
				"range_text" => theme.range_text = color,
				"syntax_sequence" => theme.syntax_sequence = color,
				"syntax_parameter" => theme.syntax_parameter = color,
				"syntax_escape" => theme.syntax_escape = color,
				"syntax_separator" => theme.syntax_separator = color,
				"syntax_whitespace" => theme.syntax_whitespace = color,
				"rgb_red" => theme.rgb_red = color,
				"rgb_green" => theme.rgb_green = color,
				"rgb_blue" => theme.rgb_blue = color,
				_ => return Err(format!("Unknown theme role {role}")),
			}
		}
		Ok(theme)
	}
	pub fn read(path: &Path) -> Result<Self, String> {
		let bytes = match crate::persistence::read_file(path) {
			Ok(bytes) => bytes,
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
				return Ok(Self::default());
			}
			Err(error) => return Err(error.to_string()),
		};
		if bytes.len() > 65536 {
			return Err("Theme configuration exceeds 64 KiB".into());
		}
		let config: Self = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
		config.resolve()?;
		Ok(config)
	}
	/// Validated atomic save; loading never writes.
	pub fn write(&self, path: &Path) -> Result<(), String> {
		self.resolve()?;
		let parent = path
			.parent()
			.ok_or("Theme configuration has no parent directory")?;
		std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
		let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
		self.serialize(&mut serde_json::Serializer::with_formatter(
			&mut file,
			serde_json::ser::PrettyFormatter::with_indent(b"\t"),
		))
		.map_err(|e| e.to_string())?;
		file.write_all(b"\n")
			.and_then(|_| file.as_file().sync_all())
			.map_err(|e| e.to_string())?;
		file.persist(path).map_err(|e| e.to_string())?;
		Ok(())
	}
}
fn parse_color(value: &str) -> Result<Color, String> {
	if value == "default" {
		return Ok(Color::Reset);
	}
	if let Some(index) = value.strip_prefix("ansi:") {
		return index
			.parse::<u8>()
			.map(Color::Indexed)
			.map_err(|_| format!("Invalid indexed color {value}"));
	}
	if let Some(hex) = value.strip_prefix('#')
		&& hex.len() == 6
		&& hex.is_ascii()
		&& let Ok(rgb) = u32::from_str_radix(hex, 16)
	{
		return Ok(Color::Rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8));
	}
	Err(format!(
		"Invalid color {value}; use default, ansi:0–255, or #RRGGBB"
	))
}
pub fn load() -> Result<Config, String> {
	let path = crate::shortcuts::config_path()?.with_file_name("theme.json");
	Config::read(&path)
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
		fn luminance(color: Color) -> f64 {
			let Color::Rgb(r, g, b) = color else {
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
			let config = Config {
				preset: (*id).into(),
				..Config::default()
			};
			let theme = config.resolve().unwrap();
			let roundtrip: Config =
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
		assert_eq!(theme.accent, Color::Rgb(18, 52, 86));
		assert_eq!(theme.surface, Color::Indexed(17));
		assert_eq!(theme.border, Theme::default().border);
		let saved = std::fs::read(&path).unwrap();
		config.colors.insert("unknown".into(), "default".into());
		assert!(config.write(&path).is_err());
		assert_eq!(std::fs::read(&path).unwrap(), saved);
	}
	#[test]
	fn invalid_configuration_and_scoped_activation() {
		for value in ["ansi:256", "#12345", "#zzzzzz", "red", "", "#１２３"] {
			assert!(parse_color(value).is_err());
		}
		let base = accent_color();
		let custom = Theme {
			accent: Color::Green,
			..Theme::default()
		};
		with(&custom, || {
			assert_eq!(accent_color(), Color::Green);
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
