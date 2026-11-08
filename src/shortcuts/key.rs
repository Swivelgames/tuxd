//! Lossless decoded key adapter. Event kind/state are not part of a binding.
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
use ratatui_which_key::Key;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShortcutKey {
	pub code: KeyCode,
	pub modifiers: M,
}
impl ShortcutKey {
	pub fn leader() -> Self {
		Self {
			code: KeyCode::Null,
			modifiers: M::empty(),
		}
	}
	pub fn is_leader(&self) -> bool {
		*self == Self::leader()
	}

	pub fn new(code: KeyCode, modifiers: M) -> Self {
		Self { code, modifiers }.normalized()
	}
	pub fn normalized(mut self) -> Self {
		if self.code == KeyCode::BackTab {
			self.code = KeyCode::Tab;
			self.modifiers.insert(M::SHIFT);
		}
		if let KeyCode::Char(mut ch) = self.code {
			if self
				.modifiers
				.intersects(M::CONTROL | M::ALT | M::SUPER | M::HYPER | M::META)
			{
				if ch.is_ascii_uppercase() {
					ch = ch.to_ascii_lowercase();
					self.modifiers.insert(M::SHIFT);
				}
			} else {
				// Terminals report printable Shift either as a modifier or in the character.
				if self.modifiers.contains(M::SHIFT) && ch.is_ascii_lowercase() {
					ch = ch.to_ascii_uppercase();
				}
				self.modifiers.remove(M::SHIFT);
			}
			if self.modifiers == M::CONTROL && matches!(ch, '7' | '_') {
				ch = '/'; // Crossterm's legacy 0x1f representation.
			}
			self.code = KeyCode::Char(ch);
		}
		self
	}
	pub fn opener(&self) -> bool {
		(self.code == KeyCode::F(1) && self.modifiers.is_empty())
			|| (self.modifiers == M::CONTROL && matches!(self.code, KeyCode::Char('/' | ',')))
	}
	pub fn printable(&self) -> Option<char> {
		if self.modifiers.is_empty()
			&& let KeyCode::Char(ch) = self.code
			&& !ch.is_control()
		{
			Some(ch)
		} else {
			None
		}
	}
	pub fn label(&self) -> String {
		if self.is_leader() {
			return "<leader>".into();
		}
		let mut parts = Vec::new();
		for (modifier, name) in [
			(M::CONTROL, "Ctrl"),
			(M::ALT, "Alt"),
			(M::SHIFT, "Shift"),
			(M::SUPER, "Super"),
			(M::HYPER, "Hyper"),
			(M::META, "Meta"),
		] {
			if self.modifiers.contains(modifier) {
				parts.push(name.to_owned());
			}
		}
		parts.push(match self.code {
			KeyCode::Char(' ') => "Space".into(),
			KeyCode::Char(ch) if ch.is_control() => format!("U+{:04X}", ch as u32),
			KeyCode::Char(ch) => {
				if self.modifiers.is_empty() {
					ch.to_string()
				} else {
					ch.to_uppercase().collect()
				}
			}
			KeyCode::F(n) => format!("F{n}"),
			other => format!("{other:?}"),
		});
		parts.join("+")
	}
	// The library parses strings; an internal hex token avoids ambiguities for
	// spaces, angle brackets, arbitrary Unicode and combined modifiers.
	pub fn token(&self) -> String {
		let bytes = serde_json::to_vec(self).expect("key serialization");
		format!(
			"<key:{}>",
			bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
		)
	}
}
impl From<KeyEvent> for ShortcutKey {
	fn from(key: KeyEvent) -> Self {
		Self::new(key.code, key.modifiers)
	}
}
impl Key for ShortcutKey {
	fn display(&self) -> String {
		self.label()
	}
	fn is_backspace(&self) -> bool {
		self.code == KeyCode::Backspace && self.modifiers.is_empty()
	}
	fn space() -> Self {
		Self::new(KeyCode::Char(' '), M::NONE)
	}
	fn from_char(ch: char) -> Option<Self> {
		Some(Self::new(KeyCode::Char(ch), M::NONE))
	}
	fn from_special_name(name: &str) -> Option<Self> {
		let hex = name.strip_prefix("key:")?;
		if !hex.is_ascii() || !hex.len().is_multiple_of(2) {
			return None;
		}
		let bytes: Option<Vec<u8>> = (0..hex.len())
			.step_by(2)
			.map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
			.collect();
		serde_json::from_slice(&bytes?).ok()
	}
}

/// Parses trusted registry defaults, not user input. User configuration is structured.
pub fn defaults(text: &str) -> Vec<ShortcutKey> {
	text.split_whitespace()
		.map(|part| {
			let mut modifiers = M::NONE;
			let mut code = part;
			while let Some((prefix, rest)) = code.split_once('+') {
				modifiers |= match prefix {
					"Ctrl" => M::CONTROL,
					"Alt" => M::ALT,
					"Shift" => M::SHIFT,
					_ => panic!("Invalid default: {part}"),
				};
				code = rest;
			}
			let code = match code {
				"Left" => KeyCode::Left,
				"Right" => KeyCode::Right,
				"Up" => KeyCode::Up,
				"Down" => KeyCode::Down,
				"Home" => KeyCode::Home,
				"End" => KeyCode::End,
				"PageUp" => KeyCode::PageUp,
				"PageDown" => KeyCode::PageDown,
				"Backspace" => KeyCode::Backspace,
				"Delete" => KeyCode::Delete,
				"Enter" => KeyCode::Enter,
				"Tab" => KeyCode::Tab,
				"Esc" => KeyCode::Esc,
				"Space" => KeyCode::Char(' '),
				s if s.starts_with('F') && s.len() > 1 => KeyCode::F(s[1..].parse().unwrap()),
				s => {
					assert_eq!(s.chars().count(), 1);
					KeyCode::Char(s.chars().next().unwrap())
				}
			};
			ShortcutKey::new(code, modifiers)
		})
		.collect()
}

impl crate::libraries::shortcuts::BindingKey for ShortcutKey {
	fn leader() -> Self {
		Self::leader()
	}
	fn is_leader(&self) -> bool {
		self.is_leader()
	}
	fn normalized(self) -> Self {
		self.normalized()
	}
	fn token(&self) -> String {
		self.token()
	}
	fn label(&self) -> String {
		self.label()
	}
	fn leader_allowed(&self) -> bool {
		self.code != KeyCode::Esc
	}
	fn escape(&self) -> bool {
		self.code == KeyCode::Esc && self.modifiers.is_empty()
	}
	fn is_printable(&self) -> bool {
		self.printable().is_some()
	}
}
