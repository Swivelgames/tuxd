//! Stable shortcut definitions owned by this capability.
use crate::shortcuts::registry::{
	ActionId, ActionSpec, Category, InputScope, Subsection, WORKSPACE,
};
use InputScope::*;
pub static ACTIONS: &[ActionSpec] = &[
	ActionSpec {
		id: ActionId::CyclePanes,
		label: "Cycle panes",
		section: Category::Workspace,
		subsection: Some(Subsection::Panes),
		order: 0,
		scopes: WORKSPACE,
		defaults: ["F6", "", ""],
	},
	ActionSpec {
		id: ActionId::FocusEditor,
		label: "Focus Editor",
		section: Category::Workspace,
		subsection: Some(Subsection::Panes),
		order: 1,
		scopes: WORKSPACE,
		defaults: ["", "", ""],
	},
	ActionSpec {
		id: ActionId::FocusCode,
		label: "Focus Code",
		section: Category::Workspace,
		subsection: Some(Subsection::Panes),
		order: 2,
		scopes: WORKSPACE,
		defaults: ["", "", ""],
	},
	ActionSpec {
		id: ActionId::FocusTools,
		label: "Focus Tools",
		section: Category::Workspace,
		subsection: Some(Subsection::Panes),
		order: 3,
		scopes: WORKSPACE,
		defaults: ["", "", ""],
	},
	ActionSpec {
		id: ActionId::PreviousTab,
		label: "Previous slim tab",
		section: Category::Workspace,
		subsection: Some(Subsection::Panes),
		order: 4,
		scopes: &[Code, Tools, Radio, ToolTabs, Grid, Hex, Channel],
		defaults: ["Alt+Left", "", ""],
	},
	ActionSpec {
		id: ActionId::NextTab,
		label: "Next slim tab",
		section: Category::Workspace,
		subsection: Some(Subsection::Panes),
		order: 5,
		scopes: &[Code, Tools, Radio, ToolTabs, Grid, Hex, Channel],
		defaults: ["Alt+Right", "", ""],
	},
];
