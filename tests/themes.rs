use std::collections::{BTreeMap, BTreeSet};
use terminal_ux_designer::libraries::themes::{Color, Config, Group, PRESETS, Role, Theme};

#[test]
fn every_preset_and_role_matches_the_pre_refactor_palette() {
	let expected: BTreeMap<String, BTreeMap<String, String>> =
		serde_json::from_str(include_str!("fixtures/themes-v1.json")).unwrap();
	assert_eq!(expected.len(), PRESETS.len());
	for (id, _) in PRESETS {
		let theme = Theme::preset(id).unwrap();
		let actual: BTreeMap<_, _> = Role::ALL
			.iter()
			.map(|role| (role.id().to_owned(), format!("{:?}", theme.color(*role))))
			.collect();
		assert_eq!(actual, expected[*id], "{id}");
	}
}

#[test]
fn catalog_is_complete_unique_and_roundtrips_every_override_in_memory() {
	let ids: BTreeSet<_> = Role::ALL.iter().map(|role| role.id()).collect();
	assert_eq!(ids.len(), 21);
	assert_eq!(ids.len(), Role::ALL.len());
	for group in Group::ALL {
		assert!(Role::ALL.iter().any(|role| role.group() == *group));
	}
	for &role in Role::ALL {
		assert_eq!(Role::from_id(role.id()).unwrap(), role);
		assert!(!role.label().is_empty());
		let config = Config {
			colors: [(role.id().into(), "#123456".into())].into(),
			..Config::default()
		};
		let saved = serde_json::to_vec(&config).unwrap();
		let decoded: Config = serde_json::from_slice(&saved).unwrap();
		assert_eq!(decoded, config);
		let theme = decoded.resolve().unwrap();
		for &other in Role::ALL {
			assert_eq!(
				theme.color(other),
				if role == other {
					Color::Rgb(18, 52, 86)
				} else {
					Theme::default().color(other)
				}
			);
		}
	}
}

#[test]
fn validation_is_independent_of_paths_and_rejects_unknown_contract_values() {
	assert_eq!(Config::default().preset, "midnight");
	for value in ["ansi:256", "#12345", "#zzzzzz", "red", "", "#１２３"] {
		let config = Config {
			colors: [("accent".into(), value.into())].into(),
			..Config::default()
		};
		assert!(config.resolve().is_err());
	}
	for (value, expected) in [
		("default", Color::Reset),
		("ansi:255", Color::Indexed(255)),
		("#ff00Aa", Color::Rgb(255, 0, 170)),
	] {
		let config = Config {
			colors: [("accent".into(), value.into())].into(),
			..Config::default()
		};
		assert_eq!(config.resolve().unwrap().color(Role::Accent), expected);
	}
	for input in [
		r#"{"version":2,"preset":"midnight"}"#,
		r#"{"version":1,"preset":"baseline"}"#,
		r#"{"version":1,"preset":"midnight","colors":{"unknown":"default"}}"#,
	] {
		assert!(
			serde_json::from_str::<Config>(input)
				.unwrap()
				.resolve()
				.is_err()
		);
	}
	assert!(
		serde_json::from_str::<Config>(r#"{"version":1,"preset":"midnight","oops":1}"#).is_err()
	);
}
