//! Application configuration location; codecs never discover paths.
use std::path::PathBuf;
pub fn directory() -> Result<PathBuf, String> {
	if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME").filter(|s| !s.is_empty()) {
		let dir = PathBuf::from(dir);
		if !dir.is_absolute() {
			return Err("XDG_CONFIG_HOME must be absolute".into());
		}
		return Ok(dir.join("tuxd"));
	}
	std::env::var_os("HOME")
		.filter(|s| !s.is_empty())
		.map(|p| PathBuf::from(p).join(".config/tuxd"))
		.ok_or_else(|| "Cannot locate user config directory (HOME is unset)".into())
}

pub mod commit;

/// Configuration codecs validate their schema before requesting publication.
pub fn write_json(path: &std::path::Path, value: &impl serde::Serialize) -> Result<(), String> {
	let mut bytes = Vec::new();
	value
		.serialize(&mut serde_json::Serializer::with_formatter(
			&mut bytes,
			serde_json::ser::PrettyFormatter::with_indent(b"\t"),
		))
		.map_err(|e| e.to_string())?;
	bytes.push(b'\n');
	let parent = path
		.parent()
		.ok_or("Configuration has no parent directory")?;
	std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
	crate::libraries::storage::publish(path, &bytes, true, None).map_err(|e| e.to_string())
}
