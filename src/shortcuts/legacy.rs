//! Version 1 action IDs/defaults, retained only for loss-aware configuration loading.
use super::{ActionId as A, Config, Slots, key};
use serde::Deserialize;
use std::collections::BTreeMap;
struct Legacy {
	id: &'static str,
	defaults: [&'static str; 3],
	targets: &'static [A],
}
const LEGACY: &[Legacy] = &[
	Legacy {
		id: "new",
		defaults: ["Ctrl+n", "", ""],
		targets: &[A::New],
	},
	Legacy {
		id: "open",
		defaults: ["Ctrl+o", "", ""],
		targets: &[A::Open],
	},
	Legacy {
		id: "save",
		defaults: ["Ctrl+s", "", ""],
		targets: &[A::Save],
	},
	Legacy {
		id: "save_as",
		defaults: ["Ctrl+Shift+s", "", ""],
		targets: &[A::SaveAs],
	},
	Legacy {
		id: "import",
		defaults: ["", "", ""],
		targets: &[A::Import],
	},
	Legacy {
		id: "export",
		defaults: ["Ctrl+e", "", ""],
		targets: &[A::Export],
	},
	Legacy {
		id: "quit",
		defaults: ["Ctrl+q", "", ""],
		targets: &[A::Quit],
	},
	Legacy {
		id: "file_menu",
		defaults: ["F4", "", ""],
		targets: &[A::FileMenu],
	},
	Legacy {
		id: "toggle_code",
		defaults: ["F2", "", ""],
		targets: &[A::ToggleCode],
	},
	Legacy {
		id: "toggle_editor",
		defaults: ["F5", "", ""],
		targets: &[A::ToggleEditor],
	},
	Legacy {
		id: "editor_wrap",
		defaults: ["F3", "", ""],
		targets: &[A::EditorWrap],
	},
	Legacy {
		id: "bold",
		defaults: ["Ctrl+b", "", ""],
		targets: &[A::Bold],
	},
	Legacy {
		id: "italic",
		defaults: ["Ctrl+i", "", ""],
		targets: &[A::Italic],
	},
	Legacy {
		id: "underline",
		defaults: ["Ctrl+u", "", ""],
		targets: &[A::Underline],
	},
	Legacy {
		id: "undo",
		defaults: ["Ctrl+z", "", ""],
		targets: &[A::Undo],
	},
	Legacy {
		id: "redo",
		defaults: ["Ctrl+y", "Ctrl+Shift+z", ""],
		targets: &[A::Redo],
	},
	Legacy {
		id: "move_left",
		defaults: ["Left", "", ""],
		targets: &[A::MoveLeft],
	},
	Legacy {
		id: "select_left",
		defaults: ["Shift+Left", "", ""],
		targets: &[A::SelectLeft],
	},
	Legacy {
		id: "move_right",
		defaults: ["Right", "", ""],
		targets: &[A::MoveRight],
	},
	Legacy {
		id: "select_right",
		defaults: ["Shift+Right", "", ""],
		targets: &[A::SelectRight],
	},
	Legacy {
		id: "move_up",
		defaults: ["Up", "", ""],
		targets: &[A::MoveUp],
	},
	Legacy {
		id: "select_up",
		defaults: ["Shift+Up", "", ""],
		targets: &[A::SelectUp],
	},
	Legacy {
		id: "move_down",
		defaults: ["Down", "", ""],
		targets: &[A::MoveDown],
	},
	Legacy {
		id: "select_down",
		defaults: ["Shift+Down", "", ""],
		targets: &[A::SelectDown],
	},
	Legacy {
		id: "move_home",
		defaults: ["Home", "", ""],
		targets: &[A::MoveHome],
	},
	Legacy {
		id: "select_home",
		defaults: ["Shift+Home", "", ""],
		targets: &[A::SelectHome],
	},
	Legacy {
		id: "move_end",
		defaults: ["End", "", ""],
		targets: &[A::MoveEnd],
	},
	Legacy {
		id: "select_end",
		defaults: ["Shift+End", "", ""],
		targets: &[A::SelectEnd],
	},
	Legacy {
		id: "move_page_up",
		defaults: ["PageUp", "", ""],
		targets: &[A::MovePageUp],
	},
	Legacy {
		id: "select_page_up",
		defaults: ["Shift+PageUp", "", ""],
		targets: &[A::SelectPageUp],
	},
	Legacy {
		id: "move_page_down",
		defaults: ["PageDown", "", ""],
		targets: &[A::MovePageDown],
	},
	Legacy {
		id: "select_page_down",
		defaults: ["Shift+PageDown", "", ""],
		targets: &[A::SelectPageDown],
	},
	Legacy {
		id: "move_start",
		defaults: ["Ctrl+Home", "", ""],
		targets: &[A::MoveStart],
	},
	Legacy {
		id: "select_start",
		defaults: ["Shift+Ctrl+Home", "", ""],
		targets: &[A::SelectStart],
	},
	Legacy {
		id: "move_finish",
		defaults: ["Ctrl+End", "", ""],
		targets: &[A::MoveFinish],
	},
	Legacy {
		id: "select_finish",
		defaults: ["Shift+Ctrl+End", "", ""],
		targets: &[A::SelectFinish],
	},
	Legacy {
		id: "move_word_left",
		defaults: ["Ctrl+Left", "Alt+Left", ""],
		targets: &[A::MoveWordLeft],
	},
	Legacy {
		id: "select_word_left",
		defaults: ["Shift+Ctrl+Left", "Shift+Alt+Left", ""],
		targets: &[A::SelectWordLeft],
	},
	Legacy {
		id: "move_word_right",
		defaults: ["Ctrl+Right", "Alt+Right", ""],
		targets: &[A::MoveWordRight],
	},
	Legacy {
		id: "select_word_right",
		defaults: ["Shift+Ctrl+Right", "Shift+Alt+Right", ""],
		targets: &[A::SelectWordRight],
	},
	Legacy {
		id: "delete_before",
		defaults: ["Backspace", "", ""],
		targets: &[A::DeleteBefore],
	},
	Legacy {
		id: "delete_after",
		defaults: ["Delete", "", ""],
		targets: &[A::DeleteAfter],
	},
	Legacy {
		id: "delete_word",
		defaults: ["Ctrl+Backspace", "Alt+Backspace", ""],
		targets: &[A::DeleteWord],
	},
	Legacy {
		id: "insert_newline",
		defaults: ["Enter", "", ""],
		targets: &[A::InsertNewline],
	},
	Legacy {
		id: "insert_tab",
		defaults: ["Tab", "", ""],
		targets: &[A::InsertTab],
	},
	Legacy {
		id: "clear_selection",
		defaults: ["Esc", "", ""],
		targets: &[A::ClearSelection],
	},
	Legacy {
		id: "cycle_panes",
		defaults: ["F6", "", ""],
		targets: &[A::CyclePanes],
	},
	Legacy {
		id: "focus_editor",
		defaults: ["", "", ""],
		targets: &[A::FocusEditor],
	},
	Legacy {
		id: "focus_code",
		defaults: ["", "", ""],
		targets: &[A::FocusCode],
	},
	Legacy {
		id: "focus_tools",
		defaults: ["", "", ""],
		targets: &[A::FocusTools],
	},
	Legacy {
		id: "previous_tab",
		defaults: ["Alt+Left", "", ""],
		targets: &[A::PreviousTab],
	},
	Legacy {
		id: "next_tab",
		defaults: ["Alt+Right", "", ""],
		targets: &[A::NextTab],
	},
	Legacy {
		id: "code_wrap",
		defaults: ["F3", "", ""],
		targets: &[A::CodeWrap],
	},
	Legacy {
		id: "copy_code",
		defaults: ["Ctrl+c", "", ""],
		targets: &[A::Copy],
	},
	Legacy {
		id: "code_up",
		defaults: ["Up", "", ""],
		targets: &[A::CodeUp],
	},
	Legacy {
		id: "code_down",
		defaults: ["Down", "", ""],
		targets: &[A::CodeDown],
	},
	Legacy {
		id: "code_left",
		defaults: ["Left", "", ""],
		targets: &[A::CodeLeft],
	},
	Legacy {
		id: "code_right",
		defaults: ["Right", "", ""],
		targets: &[A::CodeRight],
	},
	Legacy {
		id: "code_page_up",
		defaults: ["PageUp", "", ""],
		targets: &[A::CodePageUp],
	},
	Legacy {
		id: "code_page_down",
		defaults: ["PageDown", "", ""],
		targets: &[A::CodePageDown],
	},
	Legacy {
		id: "code_home",
		defaults: ["Home", "", ""],
		targets: &[A::CodeHome],
	},
	Legacy {
		id: "code_end",
		defaults: ["End", "", ""],
		targets: &[A::CodeEnd],
	},
	Legacy {
		id: "code_start",
		defaults: ["Ctrl+Home", "", ""],
		targets: &[A::CodeStart],
	},
	Legacy {
		id: "code_finish",
		defaults: ["Ctrl+End", "", ""],
		targets: &[A::CodeFinish],
	},
	Legacy {
		id: "tool_next",
		defaults: ["Tab", "", ""],
		targets: &[A::NextField],
	},
	Legacy {
		id: "tool_previous",
		defaults: ["Shift+Tab", "", ""],
		targets: &[A::PreviousField],
	},
	Legacy {
		id: "tool_right",
		defaults: ["Right", "", ""],
		targets: &[A::FieldRight],
	},
	Legacy {
		id: "tool_left",
		defaults: ["Left", "", ""],
		targets: &[A::FieldLeft],
	},
	Legacy {
		id: "tool_first",
		defaults: ["Home", "", ""],
		targets: &[A::FieldHome],
	},
	Legacy {
		id: "tool_last",
		defaults: ["End", "", ""],
		targets: &[A::FieldEnd],
	},
	Legacy {
		id: "tool_activate",
		defaults: ["Enter", "Space", ""],
		targets: &[A::Activate],
	},
	Legacy {
		id: "rgb_apply",
		defaults: ["Enter", "", ""],
		targets: &[A::RgbApply],
	},
	Legacy {
		id: "rgb_delete",
		defaults: ["Backspace", "", ""],
		targets: &[A::FieldDeleteBefore],
	},
	Legacy {
		id: "rgb_select",
		defaults: ["Ctrl+a", "", ""],
		targets: &[A::SelectAll],
	},
	Legacy {
		id: "tool_up",
		defaults: ["Up", "", ""],
		targets: &[A::FieldUp],
	},
	Legacy {
		id: "indexed_up",
		defaults: ["Up", "", ""],
		targets: &[A::FieldUp],
	},
	Legacy {
		id: "tool_down",
		defaults: ["Down", "", ""],
		targets: &[A::FieldDown],
	},
	Legacy {
		id: "indexed_down",
		defaults: ["Down", "", ""],
		targets: &[A::FieldDown],
	},
	Legacy {
		id: "tool_page_up",
		defaults: ["PageUp", "", ""],
		targets: &[A::FieldPageUp],
	},
	Legacy {
		id: "indexed_page_up",
		defaults: ["PageUp", "", ""],
		targets: &[A::FieldPageUp],
	},
	Legacy {
		id: "tool_page_down",
		defaults: ["PageDown", "", ""],
		targets: &[A::FieldPageDown],
	},
	Legacy {
		id: "indexed_page_down",
		defaults: ["PageDown", "", ""],
		targets: &[A::FieldPageDown],
	},
	Legacy {
		id: "rgb_decrease",
		defaults: ["Left", "", ""],
		targets: &[A::RangeDecrease],
	},
	Legacy {
		id: "rgb_increase",
		defaults: ["Right", "", ""],
		targets: &[A::RangeIncrease],
	},
	Legacy {
		id: "rgb_decrease10",
		defaults: ["Shift+Left", "", ""],
		targets: &[A::RangeDecrease10],
	},
	Legacy {
		id: "rgb_increase10",
		defaults: ["Shift+Right", "", ""],
		targets: &[A::RangeIncrease10],
	},
	Legacy {
		id: "rgb_minimum",
		defaults: ["Home", "", ""],
		targets: &[A::RangeMinimum],
	},
	Legacy {
		id: "rgb_maximum",
		defaults: ["End", "", ""],
		targets: &[A::RangeMaximum],
	},
	Legacy {
		id: "picker_next",
		defaults: ["Tab", "", ""],
		targets: &[A::NextField],
	},
	Legacy {
		id: "picker_previous",
		defaults: ["Shift+Tab", "", ""],
		targets: &[A::PreviousField],
	},
	Legacy {
		id: "picker_activate",
		defaults: ["Enter", "", ""],
		targets: &[A::Activate, A::Submit],
	},
	Legacy {
		id: "picker_location",
		defaults: ["Ctrl+l", "", ""],
		targets: &[A::PickerLocation],
	},
	Legacy {
		id: "picker_cwd",
		defaults: ["Ctrl+g", "", ""],
		targets: &[A::PickerCwd],
	},
	Legacy {
		id: "picker_hidden",
		defaults: ["Ctrl+h", "Ctrl+.", ""],
		targets: &[A::PickerHidden],
	},
	Legacy {
		id: "picker_filter",
		defaults: ["Ctrl+f", "", ""],
		targets: &[A::PickerFilter],
	},
	Legacy {
		id: "picker_paste",
		defaults: ["Ctrl+v", "", ""],
		targets: &[A::Paste],
	},
	Legacy {
		id: "explorer_parent",
		defaults: ["Left", "Backspace", "h"],
		targets: &[A::ExplorerParent],
	},
	Legacy {
		id: "explorer_enter",
		defaults: ["Right", "l", ""],
		targets: &[A::ExplorerEnter],
	},
	Legacy {
		id: "explorer_up",
		defaults: ["Up", "k", ""],
		targets: &[A::ExplorerUp],
	},
	Legacy {
		id: "explorer_down",
		defaults: ["Down", "j", ""],
		targets: &[A::ExplorerDown],
	},
	Legacy {
		id: "explorer_first",
		defaults: ["Home", "g g", ""],
		targets: &[A::ExplorerFirst],
	},
	Legacy {
		id: "explorer_last",
		defaults: ["End", "G", ""],
		targets: &[A::ExplorerLast],
	},
	Legacy {
		id: "explorer_middle",
		defaults: ["M", "", ""],
		targets: &[A::ExplorerMiddle],
	},
	Legacy {
		id: "explorer_page_up",
		defaults: ["PageUp", "", ""],
		targets: &[A::ExplorerPageUp],
	},
	Legacy {
		id: "explorer_page_down",
		defaults: ["PageDown", "", ""],
		targets: &[A::ExplorerPageDown],
	},
	Legacy {
		id: "field_left",
		defaults: ["Left", "", ""],
		targets: &[A::FieldLeft],
	},
	Legacy {
		id: "field_right",
		defaults: ["Right", "", ""],
		targets: &[A::FieldRight],
	},
	Legacy {
		id: "field_home",
		defaults: ["Home", "", ""],
		targets: &[A::FieldHome],
	},
	Legacy {
		id: "field_end",
		defaults: ["End", "", ""],
		targets: &[A::FieldEnd],
	},
	Legacy {
		id: "field_delete_before",
		defaults: ["Backspace", "", ""],
		targets: &[A::FieldDeleteBefore],
	},
	Legacy {
		id: "field_delete_after",
		defaults: ["Delete", "", ""],
		targets: &[A::FieldDeleteAfter],
	},
	Legacy {
		id: "field_select_all",
		defaults: ["Ctrl+a", "", ""],
		targets: &[A::SelectAll],
	},
	Legacy {
		id: "format_raw",
		defaults: ["Left", "", ""],
		targets: &[A::FieldLeft],
	},
	Legacy {
		id: "format_escaped",
		defaults: ["Right", "", ""],
		targets: &[A::FieldRight],
	},
	Legacy {
		id: "format_toggle",
		defaults: ["Space", "", ""],
		targets: &[A::Activate],
	},
	Legacy {
		id: "dialog_next",
		defaults: ["Tab", "Right", "Down"],
		targets: &[A::NextField],
	},
	Legacy {
		id: "dialog_previous",
		defaults: ["Shift+Tab", "Left", "Up"],
		targets: &[A::PreviousField],
	},
	Legacy {
		id: "dialog_activate",
		defaults: ["Enter", "", ""],
		targets: &[A::Activate],
	},
	Legacy {
		id: "positive",
		defaults: ["s", "y", ""],
		targets: &[A::Positive],
	},
	Legacy {
		id: "negative",
		defaults: ["d", "n", ""],
		targets: &[A::Negative],
	},
	Legacy {
		id: "abort",
		defaults: ["a", "c", ""],
		targets: &[A::Abort],
	},
	Legacy {
		id: "close",
		defaults: ["Esc", "", ""],
		targets: &[A::Close],
	},
	Legacy {
		id: "review_next",
		defaults: ["Tab", "", ""],
		targets: &[A::NextField],
	},
	Legacy {
		id: "review_previous",
		defaults: ["Shift+Tab", "", ""],
		targets: &[A::PreviousField],
	},
	Legacy {
		id: "review_up",
		defaults: ["Up", "", ""],
		targets: &[A::FieldUp],
	},
	Legacy {
		id: "review_down",
		defaults: ["Down", "", ""],
		targets: &[A::FieldDown],
	},
	Legacy {
		id: "review_page_up",
		defaults: ["PageUp", "", ""],
		targets: &[A::FieldPageUp],
	},
	Legacy {
		id: "review_page_down",
		defaults: ["PageDown", "", ""],
		targets: &[A::FieldPageDown],
	},
	Legacy {
		id: "review_first",
		defaults: ["Home", "", ""],
		targets: &[A::FieldHome],
	},
	Legacy {
		id: "review_last",
		defaults: ["End", "", ""],
		targets: &[A::FieldEnd],
	},
];
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Version1 {
	version: u32,
	timeout_ms: u64,
	bindings: BTreeMap<String, Slots>,
}
pub fn upgrade(bytes: &[u8]) -> Result<Config, String> {
	let old: Version1 = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
	if old.version != 1 {
		return Err("Expected shortcuts version 1".into());
	}
	let mut config = Config {
		timeout_ms: old.timeout_ms,
		..Config::default()
	};
	let mut changes: BTreeMap<A, Vec<(String, Slots)>> = BTreeMap::new();
	for (id, slots) in old.bindings {
		let spec = LEGACY
			.iter()
			.find(|s| s.id == id)
			.ok_or_else(|| format!("Unknown legacy action: {id}"))?;
		let defaults = spec.defaults.map(|s| {
			if s.is_empty() {
				None
			} else {
				Some(key::defaults(s))
			}
		});
		if slots == defaults {
			continue;
		}
		for target in spec.targets {
			changes
				.entry(*target)
				.or_default()
				.push((id.clone(), slots.clone()));
		}
	}
	for (target, sources) in changes {
		let first = &sources[0].1;
		let merged = if sources.iter().all(|(_, slots)| slots == first) {
			first.clone()
		} else {
			if sources
				.iter()
				.any(|(_, slots)| slots.iter().all(Option::is_none))
			{
				return Err(format!(
					"Cannot merge legacy shortcuts for {target}: explicit unset conflicts with another customization ({})",
					sources
						.iter()
						.map(|(s, _)| s.as_str())
						.collect::<Vec<_>>()
						.join(", ")
				));
			}
			let mut unique = Vec::new();
			for (_, slots) in &sources {
				for keys in slots.iter().flatten() {
					if !unique.contains(keys) {
						unique.push(keys.clone());
					}
				}
			}
			if unique.len() > 3 {
				return Err(format!(
					"Cannot merge legacy shortcuts for {target}: {} distinct bindings exceed three slots",
					unique.len()
				));
			}
			std::array::from_fn(|i| unique.get(i).cloned())
		};
		config.bindings.insert(target, merged);
	}
	config.build().map_err(|e| {
		format!("Legacy shortcuts need adjustment: {e}. Original file was not changed.")
	})?;
	Ok(config)
}

#[cfg(test)]
mod tests {
	use super::*;
	fn slots(sequence: &str) -> Slots {
		[Some(key::defaults(sequence)), None, None]
	}
	fn fixture() -> serde_json::Value {
		let bindings: BTreeMap<_, Slots> = LEGACY
			.iter()
			.map(|s| {
				(
					s.id,
					s.defaults.map(|key| {
						if key.is_empty() {
							None
						} else {
							Some(key::defaults(key))
						}
					}),
				)
			})
			.collect();
		serde_json::json!({"version":1,"timeout_ms":1000,"bindings":bindings})
	}
	fn convert(value: &serde_json::Value) -> Result<Config, String> {
		upgrade(&serde_json::to_vec(value).unwrap())
	}
	#[test]
	fn original_defaults_upgrade_to_consolidated_defaults() {
		assert_eq!(convert(&fixture()).unwrap(), Config::default());
	}
	#[test]
	fn custom_slots_unsets_and_unique_merged_bindings_are_preserved() {
		let mut value = fixture();
		value["bindings"]["toggle_code"] = serde_json::to_value(slots("F8")).unwrap();
		value["bindings"]["undo"] = serde_json::json!([null, null, null]);
		value["bindings"]["tool_next"] = serde_json::to_value(slots("Ctrl+8")).unwrap();
		value["bindings"]["picker_next"] = serde_json::to_value(slots("Ctrl+9")).unwrap();
		value["timeout_ms"] = serde_json::json!(250);
		let config = convert(&value).unwrap();
		assert_eq!(config.bindings[&A::ToggleCode], slots("F8"));
		assert_eq!(config.bindings[&A::Undo], [None, None, None]);
		let merged = &config.bindings[&A::NextField];
		assert!(merged.contains(&slots("Ctrl+8")[0]));
		assert!(merged.contains(&slots("Ctrl+9")[0]));
		assert_eq!(config.timeout_ms, 250);
	}
	#[test]
	fn incompatible_legacy_merges_are_reported_without_rewriting_disk() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("shortcuts.json");
		for unset in [false, true] {
			let mut value = fixture();
			value["bindings"]["tool_next"] = serde_json::to_value(if unset {
				[None, None, None]
			} else {
				[
					Some(key::defaults("F8")),
					Some(key::defaults("F9")),
					Some(key::defaults("F10")),
				]
			})
			.unwrap();
			value["bindings"]["picker_next"] = serde_json::to_value(slots("F11")).unwrap();
			let bytes = serde_json::to_vec(&value).unwrap();
			std::fs::write(&path, &bytes).unwrap();
			let error = Config::read(&path).unwrap_err();
			assert!(
				error.contains(if unset {
					"explicit unset"
				} else {
					"exceed three slots"
				}),
				"{error}"
			);
			assert_eq!(std::fs::read(&path).unwrap(), bytes);
		}
		let mut value = fixture();
		value["bindings"]["picker_activate"] = serde_json::to_value(slots("Space")).unwrap();
		assert!(convert(&value).unwrap_err().contains("ordinary text"));
	}
}
