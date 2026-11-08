use super::{Color, Role, Theme};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
			theme.set(Role::from_id(role)?, color);
		}
		Ok(theme)
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
