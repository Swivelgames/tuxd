//! User-facing action catalogue; ordering and grouping are independent of dispatch.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InputScope {
	Editor,
	Code,
	Tools,
	Radio,
	ToolTabs,
	Grid,
	Hex,
	Channel,
	Explorer,
	Location,
	Filename,
	Format,
	FileButtons,
	Menu,
	Prompt,
	ReviewContent,
	ReviewButtons,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
	Timeout,
	Workspace,
	Editor,
	Code,
	Tools,
	Dialogs,
	Explorer,
}
impl Category {
	pub fn label(self) -> &'static str {
		match self {
			Self::Timeout => "General",
			Self::Workspace => "Workspace",
			Self::Editor => "Editor",
			Self::Code => "Code",
			Self::Tools => "Tools",
			Self::Dialogs => "Dialogs",
			Self::Explorer => "File Explorer",
		}
	}
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Subsection {
	Files,
	Panes,
	FieldActions,
	FieldNavigation,
	TextFields,
	RangeControls,
	Tools,
	Movement,
	Selection,
	Editing,
	Rgb,
	General,
	Locations,
	Navigation,
	Display,
}
impl Subsection {
	pub fn label(self) -> &'static str {
		match self {
			Self::Files => "Files",
			Self::Panes => "Panes",
			Self::FieldActions => "Field Actions",
			Self::FieldNavigation => "Field Navigation",
			Self::TextFields => "Text Fields",
			Self::RangeControls => "Range Controls",
			Self::Tools => "Tools",
			Self::Movement => "Movement",
			Self::Selection => "Selection",
			Self::Editing => "Editing",
			Self::Rgb => "RGB",
			Self::General => "General Actions",
			Self::Locations => "Locations",
			Self::Navigation => "Navigation",
			Self::Display => "Display",
		}
	}
}
impl InputScope {
	pub fn section(self) -> Category {
		use InputScope::*;
		match self {
			Editor => Category::Editor,
			Code => Category::Code,
			Tools | Radio | ToolTabs | Grid | Hex | Channel => Category::Tools,
			Explorer | Location | Filename | Format | FileButtons => Category::Explorer,
			Menu | Prompt | ReviewContent | ReviewButtons => Category::Dialogs,
		}
	}
	pub fn text(self) -> bool {
		matches!(
			self,
			Self::Editor | Self::Location | Self::Filename | Self::Hex | Self::Channel
		)
	}
	pub fn label(self) -> &'static str {
		match self {
			Self::Editor => "Editor",
			Self::Code => "Code",
			Self::Tools => "Tool buttons",
			Self::Radio => "Tool radios",
			Self::ToolTabs => "Tool tabs",
			Self::Grid => "Color and character grids",
			Self::Hex => "Hex field",
			Self::Channel => "RGB ranges",
			Self::Explorer => "Explorer list",
			Self::Location => "Location field",
			Self::Filename => "Filename field",
			Self::Format => "ANSI format",
			Self::FileButtons => "File buttons",
			Self::Menu => "Menus",
			Self::Prompt => "Prompts",
			Self::ReviewContent => "Review content",
			Self::ReviewButtons => "Review buttons",
		}
	}
}
use InputScope::*;
pub(crate) const WORKSPACE: &[InputScope] =
	&[Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel];
pub(crate) const PICKER: &[InputScope] = &[Explorer, Location, Filename, Format, FileButtons];
pub(crate) const FOCUSABLE: &[InputScope] = &[
	Tools,
	Radio,
	ToolTabs,
	Grid,
	Hex,
	Channel,
	Explorer,
	Location,
	Filename,
	Format,
	FileButtons,
	Menu,
	Prompt,
	ReviewContent,
	ReviewButtons,
];
pub(crate) const DIRECTIONAL: &[InputScope] = &[
	Tools,
	Radio,
	ToolTabs,
	Grid,
	Hex,
	Location,
	Filename,
	Format,
	FileButtons,
	Menu,
	Prompt,
	ReviewContent,
	ReviewButtons,
];
pub(crate) const VERTICAL: &[InputScope] = &[
	Tools,
	Radio,
	ToolTabs,
	Grid,
	Hex,
	Channel,
	Format,
	FileButtons,
	Menu,
	Prompt,
	ReviewContent,
	ReviewButtons,
];
pub(crate) const PAGED: &[InputScope] = &[Tools, Grid, ReviewContent, ReviewButtons];
pub(crate) const BOUNDARY: &[InputScope] = &[
	Tools,
	Radio,
	ToolTabs,
	Grid,
	Hex,
	Location,
	Filename,
	Menu,
	ReviewContent,
	ReviewButtons,
];
pub(crate) const ACTIVATABLE: &[InputScope] = &[
	Tools,
	Radio,
	ToolTabs,
	Grid,
	Explorer,
	Format,
	FileButtons,
	Menu,
	Prompt,
	ReviewButtons,
];
pub(crate) const TEXT_FIELDS: &[InputScope] = &[Location, Filename, Hex, Channel];
pub(crate) const RESPONSES: &[InputScope] = &[Prompt, ReviewContent, ReviewButtons, FileButtons];

#[derive(Clone)]
pub struct ActionSpec {
	pub id: ActionId,
	pub label: &'static str,
	pub section: Category,
	pub subsection: Option<Subsection>,
	pub order: u16,
	pub scopes: &'static [InputScope],
	pub defaults: [&'static str; 3],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionId {
	New,
	Open,
	Save,
	SaveAs,
	Import,
	Export,
	Quit,
	FileMenu,
	ToggleCode,
	ToggleEditor,
	EditorWrap,
	Bold,
	Italic,
	Underline,
	Undo,
	Redo,
	MoveLeft,
	SelectLeft,
	MoveRight,
	SelectRight,
	MoveUp,
	SelectUp,
	MoveDown,
	SelectDown,
	MoveHome,
	SelectHome,
	MoveEnd,
	SelectEnd,
	MovePageUp,
	SelectPageUp,
	MovePageDown,
	SelectPageDown,
	MoveStart,
	SelectStart,
	MoveFinish,
	SelectFinish,
	MoveWordLeft,
	SelectWordLeft,
	MoveWordRight,
	SelectWordRight,
	DeleteBefore,
	DeleteAfter,
	DeleteWord,
	InsertNewline,
	InsertTab,
	ClearSelection,
	CyclePanes,
	FocusEditor,
	FocusCode,
	FocusTools,
	PreviousTab,
	NextTab,
	CodeWrap,
	CodeUp,
	CodeDown,
	CodeLeft,
	CodeRight,
	CodePageUp,
	CodePageDown,
	CodeHome,
	CodeEnd,
	CodeStart,
	CodeFinish,
	RgbApply,
	PickerLocation,
	PickerCwd,
	PickerHidden,
	PickerFilter,
	ExplorerParent,
	ExplorerEnter,
	ExplorerUp,
	ExplorerDown,
	ExplorerFirst,
	ExplorerLast,
	ExplorerMiddle,
	ExplorerPageUp,
	ExplorerPageDown,
	FieldDeleteAfter,
	Positive,
	Negative,
	Abort,
	Close,
	NextField,
	PreviousField,
	FieldLeft,
	FieldRight,
	FieldUp,
	FieldDown,
	FieldPageUp,
	FieldPageDown,
	FieldHome,
	FieldEnd,
	Activate,
	Submit,
	RangeDecrease,
	RangeIncrease,
	RangeDecrease10,
	RangeIncrease10,
	RangeMinimum,
	RangeMaximum,
	FieldDeleteBefore,
	Copy,
	Paste,
	Cut,
	PastePlain,
	PasteLiteral,
	SelectAll,
}

/// Immutable catalog assembled independently of live or visible panel instances.
pub static ACTIONS: std::sync::LazyLock<Vec<ActionSpec>> = std::sync::LazyLock::new(|| {
	[
		crate::files::shortcuts::ACTIONS,
		crate::workspace::shortcuts::ACTIONS,
		crate::features::editor::shortcuts::ACTIONS,
		crate::features::code::shortcuts::ACTIONS,
		crate::panel::shortcuts::ACTIONS,
		crate::files::picker::shortcuts::ACTIONS,
		crate::components::shortcuts::ACTIONS,
	]
	.into_iter()
	.flatten()
	.cloned()
	.collect()
});
pub fn catalog()
-> &'static crate::libraries::shortcuts::Catalog<ActionId, InputScope, super::ShortcutKey> {
	use crate::libraries::shortcuts::{Catalog, Context, Descriptor};
	static CATALOG: std::sync::LazyLock<Catalog<ActionId, InputScope, super::ShortcutKey>> =
		std::sync::LazyLock::new(|| {
			let contexts = [
				Editor,
				Code,
				Tools,
				Radio,
				ToolTabs,
				Grid,
				Hex,
				Channel,
				Explorer,
				Location,
				Filename,
				Format,
				FileButtons,
				Menu,
				Prompt,
				ReviewContent,
				ReviewButtons,
			];
			Catalog::new(
				ACTIONS.iter().map(|spec| Descriptor {
					id: spec.id,
					label: spec.label,
					category: spec.section.label(),
					scopes: spec.scopes.to_vec(),
					defaults: super::default_slots(spec.id),
				}),
				contexts.map(|id| Context {
					id,
					editable: id.text(),
				}),
				["F1", "Ctrl+/", "Ctrl+,"]
					.into_iter()
					.map(|s| (super::key::defaults(s).remove(0), "Settings"))
					.collect(),
			)
			.expect("complete unique shortcut catalog")
		});
	&CATALOG
}

pub fn category_actions(section: Category) -> Vec<&'static ActionSpec> {
	let mut specs: Vec<_> = ACTIONS.iter().filter(|s| s.section == section).collect();
	specs.sort_by_key(|s| (s.subsection, s.order));
	specs
}
impl ActionId {
	/// Presentation-only spacing within a subsection; never a focusable row.
	pub fn starts_visual_group(self) -> bool {
		use ActionId::*;
		matches!(
			self,
			FieldLeft
				| FieldPageUp
				| Save | Import
				| FocusEditor
				| PreviousTab
				| MoveHome | MoveWordLeft
				| MovePageUp | SelectHome
				| SelectWordLeft
				| SelectPageUp
				| ClearSelection
				| InsertNewline
				| DeleteBefore
				| CodePageUp | CodeHome
				| ExplorerParent
				| ExplorerFirst
				| ExplorerPageUp
		)
	}

	pub fn applicability(self) -> String {
		let scopes = self.spec().scopes;
		if scopes == WORKSPACE {
			return "Editor, Code and Tools".into();
		}
		if scopes == PICKER {
			return "File dialogs".into();
		}
		if scopes == TEXT_FIELDS {
			return "Path, Hex and RGB fields".into();
		}
		if scopes.len() > 8 {
			return "Tool controls, file dialogs and prompts".into();
		}
		scopes
			.iter()
			.map(|s| s.label())
			.collect::<Vec<_>>()
			.join(", ")
	}

	pub fn spec(self) -> &'static ActionSpec {
		ACTIONS
			.iter()
			.find(|s| s.id == self)
			.expect("registered action")
	}
	pub fn description(self) -> String {
		use ActionId::*;
		match self {
			Cut => "Copy the Editor selection as escaped ANSI, then delete after successful forwarding. Undo restores the cut.".into(),
			PastePlain => "Decode ANSI and paste its text using the current typing style.".into(),
			PasteLiteral => "Paste clipboard text literally without decoding ANSI escape notation.".into(),
			Copy => "Copy selected field text, or the entire escaped ANSI output in Code. Editor selections are copied as escaped ANSI.".into(),
			Paste => "Paste plain or styled ANSI text into the Editor, or text into a path or RGB field. In the explorer list, paste fills File/path.".into(),
			SelectAll => "Select the focused field's contents for copying or replacement.".into(),
			NextField | PreviousField => "Move focus between fields. Radios, tabs, and grids are each one focus stop.".into(),
			FieldLeft | FieldRight | FieldUp | FieldDown => "Navigate the focused control. Radios/tabs select immediately; grids move the highlight without applying it.".into(),
			Activate => "Activate the focused button, menu item, radio, tab, swatch, or character.".into(),
			Submit => "Confirm the focused form field: navigate Location, submit a file, or apply staged RGB.".into(),
			RgbApply => "Apply the staged RGB color from an RGB field. Submit already provides Enter by default.".into(),
			ClearSelection => "Clear the document selection and return focus to Editor.".into(),
			InsertTab => "Insert a literal tab at the cursor, replacing any selection.".into(),
			ToggleCode => "Expand or collapse Code (the shared lower pane in slim layout).".into(),
			ToggleEditor => "Expand or collapse Editor; the other pane fills the available space.".into(),
			_ => format!("{} / {}", self.spec().section.label(), self.spec().subsection.map_or("General", Subsection::label)),
		}
	}
}
impl std::fmt::Display for ActionId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} / {}", self.spec().section.label(), self.spec().label)
	}
}
