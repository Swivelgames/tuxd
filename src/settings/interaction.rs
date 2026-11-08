//! Exactly one interaction owns input at a time; page drafts outlive these modes.
use super::{Capture, ShortcutKey};
use crate::libraries::controls::Field;
#[derive(Clone, Default)]
pub(super) enum Mode {
	#[default]
	Browse,
	Selector,
	Preset,
	Color(Field),
	Timeout(Field),
	Capture(Capture),
	LeaderCapture,
	LeaderChange(Option<ShortcutKey>),
	Discard(bool),
}
impl Mode {
	pub fn color_input(&self) -> Option<&str> {
		if let Self::Color(text) = self {
			Some(text.text())
		} else {
			None
		}
	}
	pub fn timeout_input(&self) -> Option<&str> {
		if let Self::Timeout(text) = self {
			Some(text.text())
		} else {
			None
		}
	}
	#[cfg(test)]
	pub fn capture(&self) -> Option<&Capture> {
		if let Self::Capture(capture) = self {
			Some(capture)
		} else {
			None
		}
	}
}
