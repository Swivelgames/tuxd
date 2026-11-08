//! Application configuration paths and atomic publication.
use super::{Config, legacy, registry};
use std::path::{Path, PathBuf};
impl Config {
	pub fn read(path: &Path) -> Result<Self, String> {
		let bytes = crate::libraries::storage::ReadPolicy {
			limit: 1048576,
			overflow: "Shortcuts file exceeds 1 MiB",
			special_file: "Shortcuts configuration must be a regular file",
			error_kind: std::io::ErrorKind::InvalidData,
		}
		.file(path)
		.map_err(|e| e.to_string())?;
		let version = serde_json::from_slice::<serde_json::Value>(&bytes)
			.map_err(|e| e.to_string())?
			.get("version")
			.and_then(|v| v.as_u64());
		if version == Some(1) {
			return legacy::upgrade(&bytes);
		}
		let mut config: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
		// Missing known actions inherit defaults; unknown IDs/fields/versions are errors.
		registry::catalog().inherit_defaults(&mut config.0);
		config.build()?;
		Ok(config)
	}
	pub fn write(&self, path: &Path) -> Result<(), String> {
		self.build()?;
		crate::configuration::write_json(path, self)
	}
}
pub fn config_path() -> Result<PathBuf, String> {
	Ok(crate::configuration::directory()?.join("shortcuts.json"))
}
