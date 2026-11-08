//! In-memory UI appearance. No application, terminal, filesystem or document dependencies.
mod catalog;
mod configuration;
mod presets;
pub use catalog::{Group, Role, Theme};
pub use configuration::Config;
pub use presets::PRESETS;

/// UI color values; terminal-default appearance is independent of authored ANSI ink.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
	Reset,
	Indexed(u8),
	Rgb(u8, u8, u8),
	DarkGray,
	White,
}
