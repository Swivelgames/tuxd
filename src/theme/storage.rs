//! Theme file adapter; resolution and schema belong to the in-memory library.
use crate::libraries::themes;
use serde::{Deserialize, Serialize};
use std::{
	ops::{Deref, DerefMut},
	path::Path,
};
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Config(themes::Config);
impl Deref for Config {
	type Target = themes::Config;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl DerefMut for Config {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
impl Config {
	pub fn read(path: &Path) -> Result<Self, String> {
		let bytes = match (crate::libraries::storage::ReadPolicy {
			limit: 65536,
			overflow: "Theme configuration exceeds 64 KiB",
			special_file: "Source must be a regular file",
			error_kind: std::io::ErrorKind::InvalidData,
		})
		.file(path)
		{
			Ok(bytes) => bytes,
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
				return Ok(Self::default());
			}
			Err(error) => return Err(error.to_string()),
		};
		let config: Self = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
		config.resolve()?;
		Ok(config)
	}
	/// Validated atomic save; loading never writes.
	pub fn write(&self, path: &Path) -> Result<(), String> {
		self.resolve()?;
		crate::configuration::write_json(path, self)
	}
}
