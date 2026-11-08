//! Contextual shortcut routing, validation and transactional configuration.
pub mod key;
mod legacy;
pub mod registry;
mod storage;
pub use key::ShortcutKey;
#[cfg(test)]
use ratatui_which_key::NodeResult;
#[cfg(test)]
use registry::ACTIONS;
pub use registry::{ActionId, InputScope};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Instant};

pub type Slots = engine::Slots<ShortcutKey>;
pub type Map = engine::Map<ActionId, InputScope, ShortcutKey>;
use crate::libraries::shortcuts as engine;
pub use engine::sequence_label;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Config(engine::Configuration<ActionId, ShortcutKey>);
impl std::ops::Deref for Config {
	type Target = engine::Configuration<ActionId, ShortcutKey>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl std::ops::DerefMut for Config {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
impl Default for Config {
	fn default() -> Self {
		Self(registry::catalog().defaults())
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
impl Config {
	pub fn build(&self) -> Result<Map, String> {
		registry::catalog().compile(&self.0)
	}
}
pub use storage::config_path;
pub type Dispatch = engine::Dispatch<ActionId>;
pub struct Runtime {
	pub config: Config,
	map: Map,
	pub path: Result<PathBuf, String>,
	resolver: engine::Resolver<InputScope, ShortcutKey>,
}
impl Default for Runtime {
	fn default() -> Self {
		let config = Config::default();
		let map = config.build().expect("valid default shortcuts");
		Self {
			config,
			map,
			path: config_path(),
			resolver: Default::default(),
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
		self.resolver.cancel();
	}
	pub fn expire(&mut self, now: Instant) -> bool {
		self.resolver.expire(now, self.config.timeout_ms)
	}
	pub fn pending_label(&self) -> Option<String> {
		self.resolver.pending_label()
	}
	pub fn dispatch(&mut self, key: ShortcutKey, scope: InputScope, now: Instant) -> Dispatch {
		self.resolver
			.dispatch(&self.map, self.config.timeout_ms, key, scope, now)
	}
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod leader_tests {
	use super::*;
	#[test]
	fn resolved_leader_dispatches_without_delaying_standalone_actions() {
		let mut runtime = Runtime::default();
		let leader = key::defaults("Ctrl+y")[0].clone();
		runtime.config.change_leader(Some(leader.clone()), false);
		runtime
			.config
			.bindings
			.get_mut(&ActionId::FocusTools)
			.unwrap()[0] = Some(vec![
			ShortcutKey::leader(),
			key::defaults("Right")[0].clone(),
		]);
		runtime.map = runtime.config.build().unwrap();
		let now = Instant::now();
		assert!(matches!(
			runtime.dispatch(leader, InputScope::Editor, now),
			Dispatch::Pending
		));
		assert!(matches!(
			runtime.dispatch(key::defaults("Right")[0].clone(), InputScope::Editor, now),
			Dispatch::Action(ActionId::FocusTools)
		));
		assert!(matches!(
			runtime.dispatch(key::defaults("Ctrl+z")[0].clone(), InputScope::Editor, now),
			Dispatch::Action(ActionId::Undo)
		));
	}

	#[test]
	fn leader_conversion_and_literal_preservation() {
		let mut config = Config::default();
		let key = key::defaults("Ctrl+y")[0].clone();
		config.change_leader(Some(key.clone()), false);
		assert!(config.bindings[&ActionId::Redo][0].is_none());
		config.build().unwrap();
		config.bindings.get_mut(&ActionId::Redo).unwrap()[0] =
			Some(vec![ShortcutKey::leader(), key::defaults("g")[0].clone()]);
		config.build().unwrap();
		let roundtrip: Config =
			serde_json::from_str(&serde_json::to_string(&config).unwrap()).unwrap();
		assert_eq!(config, roundtrip);
		config.change_leader(None, false);
		assert_eq!(
			config.bindings[&ActionId::Redo][0].as_ref().unwrap()[0],
			key
		);
		config.build().unwrap();
	}
	#[test]
	fn resolved_conflicts_and_remove() {
		let mut config = Config::default();
		config.change_leader(Some(key::defaults("Ctrl+y")[0].clone()), false);
		let sequence = vec![ShortcutKey::leader(), key::defaults("g")[0].clone()];
		config.bindings.get_mut(&ActionId::Redo).unwrap()[0] = Some(sequence.clone());
		config.bindings.get_mut(&ActionId::Undo).unwrap()[0] = Some(sequence);
		assert!(config.build().unwrap_err().contains("conflicts"));
		config.change_leader(None, true);
		assert!(config.bindings[&ActionId::Redo][0].is_none());
		config.build().unwrap();
	}
}
