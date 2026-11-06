//! Contextual shortcut routing, validation and transactional configuration.
pub mod key;
mod legacy;
pub mod registry;
pub mod settings;
pub use key::ShortcutKey;
use ratatui_which_key::{Keymap, NodeResult};
pub use registry::{ACTIONS, ActionId, InputScope, SettingsSection};
use serde::{Deserialize, Serialize};
use std::{
	collections::BTreeMap,
	io::{Read, Write},
	path::{Path, PathBuf},
	time::{Duration, Instant},
};

pub type Slots = [Option<Vec<ShortcutKey>>; 3];
pub type Map = Keymap<ShortcutKey, InputScope, ActionId, SettingsSection>;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
	pub version: u32,
	pub timeout_ms: u64,
	pub bindings: BTreeMap<ActionId, Slots>,
}
impl Default for Config {
	fn default() -> Self {
		Self {
			version: 2,
			timeout_ms: 1000,
			bindings: ACTIONS
				.iter()
				.map(|s| (s.id, default_slots(s.id)))
				.collect(),
		}
	}
}
pub fn default_slots(action: ActionId) -> Slots {
	action.spec().defaults.map(|text| {
		if text.is_empty() {
			None
		} else {
			Some(key::defaults(text))
		}
	})
}
pub fn sequence_label(keys: &[ShortcutKey]) -> String {
	keys.iter()
		.map(ShortcutKey::label)
		.collect::<Vec<_>>()
		.join(" ")
}
impl Config {
	pub fn build(&self) -> Result<Map, String> {
		if self.version != 2 {
			return Err(format!(
				"Unsupported shortcuts version {} (expected 2)",
				self.version
			));
		}
		if !(1..=60_000).contains(&self.timeout_ms) {
			return Err("Sequence timeout must be 1–60000 milliseconds".into());
		}
		let mut entries = Vec::new();
		for spec in ACTIONS {
			let slots = self
				.bindings
				.get(&spec.id)
				.ok_or_else(|| format!("Missing action: {}", spec.label))?;
			for (slot, keys) in slots.iter().enumerate() {
				let Some(keys) = keys else {
					continue;
				};
				if keys.is_empty() || keys.len() > 9 {
					return Err(format!(
						"{} slot {}: use 1–9 keys or unset",
						spec.label,
						slot + 1
					));
				}
				for key in keys {
					if *key != key.clone().normalized() {
						return Err(format!(
							"{} slot {}: noncanonical key {}",
							spec.label,
							slot + 1,
							key.label()
						));
					}
					if key.opener() {
						return Err(format!("{} is reserved for Shortcut Settings", key.label()));
					}
				}
				if keys.len() > 1
					&& keys
						.iter()
						.any(|k| k.code == crossterm::event::KeyCode::Esc && k.modifiers.is_empty())
				{
					return Err("Esc cancels sequences and cannot be part of one".into());
				}
				if keys[0].printable().is_some() && spec.scopes.iter().any(|scope| scope.text()) {
					return Err(format!(
						"{} would consume ordinary text in an editable context ({})",
						spec.label,
						keys[0].label()
					));
				}
				entries.push((spec, slot, keys));
			}
		}
		for (i, (a, slot_a, keys_a)) in entries.iter().enumerate() {
			for (b, slot_b, keys_b) in &entries[i + 1..] {
				if !(keys_a.starts_with(keys_b) || keys_b.starts_with(keys_a)) {
					continue;
				}
				let scopes: Vec<_> = a.scopes.iter().filter(|s| b.scopes.contains(s)).collect();
				if !scopes.is_empty() {
					return Err(format!(
						"{} slot {} ({}) conflicts with {} slot {} ({}) in {:?}",
						a.id,
						slot_a + 1,
						sequence_label(keys_a),
						b.id,
						slot_b + 1,
						sequence_label(keys_b),
						scopes
					));
				}
			}
		}
		let mut map = Map::new();
		for (spec, _, keys) in &entries {
			let token: String = keys.iter().map(ShortcutKey::token).collect();
			for scope in spec.scopes {
				map.bind(&token, spec.id, spec.section, *scope);
				// Name groups from the sections of their effective descendants.
				for length in 1..keys.len() {
					let prefix = &keys[..length];
					let same_section = entries
						.iter()
						.filter(|(s, _, seq)| s.scopes.contains(scope) && seq.starts_with(prefix))
						.all(|(s, _, _)| s.section == spec.section);
					map.describe_group_for_scope(
						&prefix.iter().map(ShortcutKey::token).collect::<String>(),
						if same_section {
							spec.section.label()
						} else {
							"Actions"
						},
						*scope,
					);
				}
			}
		}
		Ok(map)
	}
	pub fn read(path: &Path) -> Result<Self, String> {
		let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
		if !metadata.is_file() {
			return Err("Shortcuts configuration must be a regular file".into());
		}
		let mut bytes = Vec::new();
		std::fs::File::open(path)
			.map_err(|e| e.to_string())?
			.take(1024 * 1024 + 1)
			.read_to_end(&mut bytes)
			.map_err(|e| e.to_string())?;
		if bytes.len() > 1024 * 1024 {
			return Err("Shortcuts file exceeds 1 MiB".into());
		}
		let version = serde_json::from_slice::<serde_json::Value>(&bytes)
			.map_err(|e| e.to_string())?
			.get("version")
			.and_then(|v| v.as_u64());
		if version == Some(1) {
			return legacy::upgrade(&bytes);
		}
		let mut config: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
		// Missing known actions inherit defaults; unknown IDs/fields/versions are errors.
		for spec in ACTIONS {
			config
				.bindings
				.entry(spec.id)
				.or_insert_with(|| default_slots(spec.id));
		}
		config.build()?;
		Ok(config)
	}
	pub fn write(&self, path: &Path) -> Result<(), String> {
		self.build()?;
		let parent = path
			.parent()
			.ok_or("Configuration has no parent directory")?;
		std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
		let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
		let formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");
		self.serialize(&mut serde_json::Serializer::with_formatter(
			&mut file, formatter,
		))
		.map_err(|e| e.to_string())?;
		file.write_all(b"\n")
			.and_then(|_| file.as_file().sync_all())
			.map_err(|e| e.to_string())?;
		file.persist(path).map_err(|e| e.to_string())?;
		Ok(())
	}
	pub fn hint(&self, action: ActionId) -> String {
		self.bindings
			.get(&action)
			.and_then(|slots| slots.iter().flatten().next())
			.map(|s| sequence_label(s))
			.unwrap_or_default()
	}
}
pub fn config_path() -> Result<PathBuf, String> {
	if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME").filter(|s| !s.is_empty()) {
		let dir = PathBuf::from(dir);
		if !dir.is_absolute() {
			return Err("XDG_CONFIG_HOME must be absolute".into());
		}
		return Ok(dir.join("tuxd/shortcuts.json"));
	}
	std::env::var_os("HOME")
		.filter(|s| !s.is_empty())
		.map(|p| PathBuf::from(p).join(".config/tuxd/shortcuts.json"))
		.ok_or_else(|| "Cannot locate user config directory (HOME is unset)".into())
}
pub enum Dispatch {
	Action(ActionId),
	Pending,
	Consumed,
	Unbound,
}
pub struct Runtime {
	pub config: Config,
	map: Map,
	pub path: Result<PathBuf, String>,
	pending: Vec<ShortcutKey>,
	scope: Option<InputScope>,
	last: Option<Instant>,
}
impl Default for Runtime {
	fn default() -> Self {
		let config = Config::default();
		let map = config.build().expect("valid default shortcuts");
		Self {
			config,
			map,
			path: config_path(),
			pending: Vec::new(),
			scope: None,
			last: None,
		}
	}
}
impl Runtime {
	pub fn load(&mut self) -> Result<(), String> {
		let path = self.path.as_ref().map_err(Clone::clone)?;
		match std::fs::metadata(path) {
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
			Err(e) => return Err(e.to_string()),
			Ok(_) => (),
		}
		let config = Config::read(path)?;
		self.map = config.build()?;
		self.config = config;
		Ok(())
	}
	pub fn save(&mut self, config: Config) -> Result<(), String> {
		let map = config.build()?;
		config.write(self.path.as_ref().map_err(Clone::clone)?)?;
		self.config = config;
		self.map = map;
		self.cancel();
		Ok(())
	}
	pub fn cancel(&mut self) {
		self.pending.clear();
		self.last = None;
		self.scope = None;
	}
	pub fn expire(&mut self, now: Instant) -> bool {
		if self.last.is_some_and(|last| {
			now.saturating_duration_since(last) >= Duration::from_millis(self.config.timeout_ms)
		}) {
			self.cancel();
			true
		} else {
			false
		}
	}
	pub fn pending_label(&self) -> Option<String> {
		(!self.pending.is_empty()).then(|| format!("{} …", sequence_label(&self.pending)))
	}
	pub fn dispatch(&mut self, key: ShortcutKey, scope: InputScope, now: Instant) -> Dispatch {
		if self.scope != Some(scope) {
			self.cancel();
		}
		self.expire(now);
		let had_prefix = !self.pending.is_empty();
		if had_prefix && key.code == crossterm::event::KeyCode::Esc && key.modifiers.is_empty() {
			self.cancel();
			return Dispatch::Consumed;
		}
		self.scope = Some(scope);
		self.pending.push(key);
		match self.map.navigate(&self.pending, &scope) {
			Some(NodeResult::Leaf { action }) => {
				self.cancel();
				Dispatch::Action(action)
			}
			Some(NodeResult::Branch { .. }) => {
				self.last = Some(now);
				Dispatch::Pending
			}
			None => {
				self.cancel();
				if had_prefix {
					Dispatch::Consumed
				} else {
					Dispatch::Unbound
				}
			}
		}
	}
}

#[cfg(test)]
mod tests;
