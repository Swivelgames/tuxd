//! Ordered two-file publication with best-effort restoration, not a crash-atomic transaction.
use crate::{shortcuts, theme};
use std::path::PathBuf;

/// Borrowed validated-page data; the coordinator never owns UI state.
pub struct Draft<'a> {
	pub shortcuts: &'a shortcuts::Config,
	pub theme: &'a theme::Config,
	pub original_theme: &'a theme::Config,
}
pub fn save(
	draft: Draft<'_>,
	publish_shortcuts: impl FnOnce(&shortcuts::Config) -> Result<(), String>,
	theme_path: &Result<PathBuf, String>,
) -> Result<(), String> {
	draft.shortcuts.build()?;
	draft.theme.resolve()?;
	let path = theme_path.as_ref().map_err(Clone::clone)?;
	let changed = draft.theme != draft.original_theme;
	let previous = if changed {
		match std::fs::read(path) {
			Ok(bytes) => Some(bytes),
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
			Err(e) => return Err(e.to_string()),
		}
	} else {
		None
	};
	if changed {
		draft.theme.write(path)?;
	}
	if let Err(error) = publish_shortcuts(draft.shortcuts) {
		if changed {
			let restored = if let Some(bytes) = previous {
				crate::libraries::storage::publish(path, &bytes, true, None)
			} else {
				std::fs::remove_file(path)
			};
			if let Err(restore_error) = restored {
				return Err(format!(
					"{error}; could not restore theme file: {restore_error}"
				));
			}
		}
		return Err(error);
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn theme_publication_failure_never_publishes_shortcuts() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("theme.json");
		std::fs::create_dir(&path).unwrap();
		let shortcuts = shortcuts::Config::default();
		let original = theme::Config::default();
		let mut changed = original.clone();
		changed.preset = "light".into();
		let result = save(
			Draft {
				shortcuts: &shortcuts,
				theme: &changed,
				original_theme: &original,
			},
			|_| panic!("must not publish shortcuts"),
			&Ok(path),
		);
		assert!(result.is_err());
	}

	#[test]
	fn successful_commit_uses_supplied_theme_destination_and_activates_shortcuts() {
		let dir = tempfile::tempdir().unwrap();
		let theme_path = dir.path().join("appearance/theme.json");
		let mut runtime = shortcuts::Runtime::default();
		runtime.path = Ok(dir.path().join("bindings/shortcuts.json"));
		let mut shortcuts = runtime.config.clone();
		shortcuts.timeout_ms = 300;
		let original = theme::Config::default();
		let mut theme = original.clone();
		theme.preset = "one-dark".into();
		save(
			Draft {
				shortcuts: &shortcuts,
				theme: &theme,
				original_theme: &original,
			},
			|config| runtime.save(config.clone()),
			&Ok(theme_path.clone()),
		)
		.unwrap();
		assert_eq!(runtime.config, shortcuts);
		assert_eq!(theme::Config::read(&theme_path).unwrap(), theme);
		assert_eq!(
			shortcuts::Config::read(runtime.path.as_ref().unwrap()).unwrap(),
			shortcuts
		);
	}

	#[test]
	fn invalid_drafts_have_no_effect_and_restoration_failure_is_reported() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("theme.json");
		let shortcuts = shortcuts::Config::default();
		let original = theme::Config::default();
		let mut theme = original.clone();
		theme.preset = "missing".into();
		assert!(
			save(
				Draft {
					shortcuts: &shortcuts,
					theme: &theme,
					original_theme: &original
				},
				|_| panic!("invalid draft"),
				&Ok(path.clone())
			)
			.is_err()
		);
		assert!(!path.exists());

		theme.preset = "light".into();
		original.write(&path).unwrap();
		let result = save(
			Draft {
				shortcuts: &shortcuts,
				theme: &theme,
				original_theme: &original,
			},
			|_| {
				// Model a filesystem change between publication and rollback.
				std::fs::remove_file(&path).unwrap();
				std::fs::create_dir(&path).unwrap();
				Err("shortcut write failed".into())
			},
			&Ok(path.clone()),
		);
		let error = result.unwrap_err();
		assert!(error.starts_with("shortcut write failed; could not restore theme file:"));
	}
}
