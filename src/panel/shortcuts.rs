//! Stable shortcut definitions owned by this capability.
use crate::shortcuts::registry::{ActionId, ActionSpec, Category, InputScope, Subsection};
use InputScope::*;
pub static ACTIONS: &[ActionSpec] = &[
	ActionSpec {
		id: ActionId::RgbApply,
		label: "Apply RGB",
		section: Category::Tools,
		subsection: Some(Subsection::Rgb),
		order: 0,
		scopes: &[Hex, Channel],
		defaults: ["", "", ""],
	},
	ActionSpec {
		id: ActionId::CharacterSearch,
		label: "Search characters",
		section: Category::Tools,
		subsection: None,
		order: 0,
		scopes: &[Characters, CharacterSearch],
		defaults: ["Ctrl+f", "", ""],
	},
];
