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
pub enum SettingsSection {
	ThemeGroup,
	Appearance,
	ShortcutsGroup,
	Timeout,
	Workspace,
	Editor,
	Code,
	Tools,
	Dialogs,
	Explorer,
}
impl SettingsSection {
	pub const ALL: [Self; 10] = [
		Self::ThemeGroup,
		Self::Appearance,
		Self::ShortcutsGroup,
		Self::Timeout,
		Self::Workspace,
		Self::Editor,
		Self::Code,
		Self::Tools,
		Self::Dialogs,
		Self::Explorer,
	];
	pub fn label(self) -> &'static str {
		match self {
			Self::ThemeGroup => "Theme",
			Self::Appearance => "Appearance",
			Self::ShortcutsGroup => "Shortcuts",
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
	pub fn section(self) -> SettingsSection {
		use InputScope::*;
		match self {
			Editor => SettingsSection::Editor,
			Code => SettingsSection::Code,
			Tools | Radio | ToolTabs | Grid | Hex | Channel => SettingsSection::Tools,
			Explorer | Location | Filename | Format | FileButtons => SettingsSection::Explorer,
			Menu | Prompt | ReviewContent | ReviewButtons => SettingsSection::Dialogs,
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
const WORKSPACE: &[InputScope] = &[Editor, Code, Tools, Radio, ToolTabs, Grid, Hex, Channel];
const PICKER: &[InputScope] = &[Explorer, Location, Filename, Format, FileButtons];
const FOCUSABLE: &[InputScope] = &[
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
const DIRECTIONAL: &[InputScope] = &[
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
const VERTICAL: &[InputScope] = &[
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
const PAGED: &[InputScope] = &[Tools, Grid, ReviewContent, ReviewButtons];
const BOUNDARY: &[InputScope] = &[
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
const ACTIVATABLE: &[InputScope] = &[
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
const TEXT_FIELDS: &[InputScope] = &[Location, Filename, Hex, Channel];
const RESPONSES: &[InputScope] = &[Prompt, ReviewContent, ReviewButtons, FileButtons];

pub struct ActionSpec {
	pub id: ActionId,
	pub label: &'static str,
	pub section: SettingsSection,
	pub subsection: Option<Subsection>,
	pub order: u16,
	pub scopes: &'static [InputScope],
	pub defaults: [&'static str; 3],
}
macro_rules! catalogue {
	($( $id:ident, $label:literal, $section:ident, $subsection:expr, $order:literal, $scopes:expr, $defaults:expr; )*) => {
		#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
		#[serde(rename_all = "snake_case")]
		pub enum ActionId { $( $id, )* }
		pub static ACTIONS: &[ActionSpec] = &[$(ActionSpec { id: ActionId::$id, label: $label,
			section: SettingsSection::$section, subsection: $subsection, order: $order, scopes: $scopes, defaults: $defaults,
		},)*];
	};
}
catalogue! {
	New, "New document", Workspace, Some(Subsection::Files), 2, WORKSPACE, ["Ctrl+n", "", ""];
	Open, "Open document", Workspace, Some(Subsection::Files), 3, WORKSPACE, ["Ctrl+o", "", ""];
	Save, "Save", Workspace, Some(Subsection::Files), 4, WORKSPACE, ["Ctrl+s", "", ""];
	SaveAs, "Save as", Workspace, Some(Subsection::Files), 5, WORKSPACE, ["Ctrl+Shift+s", "", ""];
	Import, "Import", Workspace, Some(Subsection::Files), 6, WORKSPACE, ["", "", ""];
	Export, "Export", Workspace, Some(Subsection::Files), 7, WORKSPACE, ["Ctrl+e", "", ""];
	Quit, "Quit", Workspace, None, 0, WORKSPACE, ["Ctrl+q", "", ""];
	FileMenu, "Files menu", Workspace, Some(Subsection::Files), 1, WORKSPACE, ["F4", "", ""];
	ToggleCode, "Toggle Code", Code, None, 0, WORKSPACE, ["F2", "", ""];
	ToggleEditor, "Toggle Editor", Editor, None, 0, WORKSPACE, ["F5", "", ""];
	EditorWrap, "Toggle wrapping", Editor, None, 1, &[Editor], ["F3", "", ""];
	Bold, "Toggle bold", Editor, Some(Subsection::Tools), 0, WORKSPACE, ["Ctrl+b", "", ""];
	Italic, "Toggle italic", Editor, Some(Subsection::Tools), 1, WORKSPACE, ["Ctrl+i", "", ""];
	Underline, "Toggle underline", Editor, Some(Subsection::Tools), 2, WORKSPACE, ["Ctrl+u", "", ""];
	Undo, "Undo", Editor, Some(Subsection::Editing), 0, WORKSPACE, ["Ctrl+z", "", ""];
	Redo, "Redo", Editor, Some(Subsection::Editing), 1, WORKSPACE, ["Ctrl+y", "Ctrl+Shift+z", ""];
	MoveLeft, "Left", Editor, Some(Subsection::Movement), 0, &[Editor], ["Left", "", ""];
	SelectLeft, "Select left", Editor, Some(Subsection::Selection), 0, &[Editor], ["Shift+Left", "", ""];
	MoveRight, "Right", Editor, Some(Subsection::Movement), 1, &[Editor], ["Right", "", ""];
	SelectRight, "Select right", Editor, Some(Subsection::Selection), 1, &[Editor], ["Shift+Right", "", ""];
	MoveUp, "Up", Editor, Some(Subsection::Movement), 2, &[Editor], ["Up", "", ""];
	SelectUp, "Select up", Editor, Some(Subsection::Selection), 2, &[Editor], ["Shift+Up", "", ""];
	MoveDown, "Down", Editor, Some(Subsection::Movement), 3, &[Editor], ["Down", "", ""];
	SelectDown, "Select down", Editor, Some(Subsection::Selection), 3, &[Editor], ["Shift+Down", "", ""];
	MoveHome, "Line beginning", Editor, Some(Subsection::Movement), 4, &[Editor], ["Home", "", ""];
	SelectHome, "Select line beginning", Editor, Some(Subsection::Selection), 4, &[Editor], ["Shift+Home", "", ""];
	MoveEnd, "Line end", Editor, Some(Subsection::Movement), 5, &[Editor], ["End", "", ""];
	SelectEnd, "Select line end", Editor, Some(Subsection::Selection), 5, &[Editor], ["Shift+End", "", ""];
	MovePageUp, "Page up", Editor, Some(Subsection::Movement), 10, &[Editor], ["PageUp", "", ""];
	SelectPageUp, "Select page up", Editor, Some(Subsection::Selection), 10, &[Editor], ["Shift+PageUp", "", ""];
	MovePageDown, "Page down", Editor, Some(Subsection::Movement), 11, &[Editor], ["PageDown", "", ""];
	SelectPageDown, "Select page down", Editor, Some(Subsection::Selection), 11, &[Editor], ["Shift+PageDown", "", ""];
	MoveStart, "Document beginning", Editor, Some(Subsection::Movement), 6, &[Editor], ["Ctrl+Home", "", ""];
	SelectStart, "Select document beginning", Editor, Some(Subsection::Selection), 6, &[Editor], ["Shift+Ctrl+Home", "", ""];
	MoveFinish, "Document end", Editor, Some(Subsection::Movement), 7, &[Editor], ["Ctrl+End", "", ""];
	SelectFinish, "Select document end", Editor, Some(Subsection::Selection), 7, &[Editor], ["Shift+Ctrl+End", "", ""];
	MoveWordLeft, "Previous word", Editor, Some(Subsection::Movement), 8, &[Editor], ["Ctrl+Left", "Alt+Left", ""];
	SelectWordLeft, "Select previous word", Editor, Some(Subsection::Selection), 8, &[Editor], ["Shift+Ctrl+Left", "Shift+Alt+Left", ""];
	MoveWordRight, "Next word", Editor, Some(Subsection::Movement), 9, &[Editor], ["Ctrl+Right", "Alt+Right", ""];
	SelectWordRight, "Select next word", Editor, Some(Subsection::Selection), 9, &[Editor], ["Shift+Ctrl+Right", "Shift+Alt+Right", ""];
	DeleteBefore, "Backspace", Editor, Some(Subsection::Editing), 4, &[Editor], ["Backspace", "", ""];
	DeleteAfter, "Delete", Editor, Some(Subsection::Editing), 5, &[Editor], ["Delete", "", ""];
	DeleteWord, "Delete previous word", Editor, Some(Subsection::Editing), 6, &[Editor], ["Ctrl+Backspace", "Alt+Backspace", ""];
	InsertNewline, "Newline", Editor, Some(Subsection::Editing), 2, &[Editor], ["Enter", "", ""];
	InsertTab, "Indent", Editor, Some(Subsection::Editing), 3, &[Editor], ["Tab", "", ""];
	ClearSelection, "Clear selection", Editor, Some(Subsection::Selection), 99, WORKSPACE, ["Esc", "", ""];
	CyclePanes, "Cycle panes", Workspace, Some(Subsection::Panes), 0, WORKSPACE, ["F6", "", ""];
	FocusEditor, "Focus Editor", Workspace, Some(Subsection::Panes), 1, WORKSPACE, ["", "", ""];
	FocusCode, "Focus Code", Workspace, Some(Subsection::Panes), 2, WORKSPACE, ["", "", ""];
	FocusTools, "Focus Tools", Workspace, Some(Subsection::Panes), 3, WORKSPACE, ["", "", ""];
	PreviousTab, "Previous slim tab", Workspace, Some(Subsection::Panes), 4, &[Code, Tools, Radio, ToolTabs, Grid, Hex, Channel], ["Alt+Left", "", ""];
	NextTab, "Next slim tab", Workspace, Some(Subsection::Panes), 5, &[Code, Tools, Radio, ToolTabs, Grid, Hex, Channel], ["Alt+Right", "", ""];
	CodeWrap, "Toggle wrapping", Code, None, 1, &[Code], ["F3", "", ""];
	CodeUp, "Up", Code, Some(Subsection::Movement), 2, &[Code], ["Up", "", ""];
	CodeDown, "Down", Code, Some(Subsection::Movement), 3, &[Code], ["Down", "", ""];
	CodeLeft, "Left", Code, Some(Subsection::Movement), 0, &[Code], ["Left", "", ""];
	CodeRight, "Right", Code, Some(Subsection::Movement), 1, &[Code], ["Right", "", ""];
	CodePageUp, "Page up", Code, Some(Subsection::Movement), 4, &[Code], ["PageUp", "", ""];
	CodePageDown, "Page down", Code, Some(Subsection::Movement), 5, &[Code], ["PageDown", "", ""];
	CodeHome, "Line beginning", Code, Some(Subsection::Movement), 6, &[Code], ["Home", "", ""];
	CodeEnd, "Line end", Code, Some(Subsection::Movement), 7, &[Code], ["End", "", ""];
	CodeStart, "Document beginning", Code, Some(Subsection::Movement), 8, &[Code], ["Ctrl+Home", "", ""];
	CodeFinish, "Document end", Code, Some(Subsection::Movement), 9, &[Code], ["Ctrl+End", "", ""];
	RgbApply, "Apply RGB", Tools, Some(Subsection::Rgb), 0, &[Hex, Channel], ["", "", ""];
	PickerLocation, "Focus Location", Explorer, Some(Subsection::Locations), 0, PICKER, ["Ctrl+l", "", ""];
	PickerCwd, "Go to CWD", Explorer, Some(Subsection::Locations), 1, PICKER, ["Ctrl+g", "", ""];
	PickerHidden, "Toggle hidden files", Explorer, Some(Subsection::Display), 0, PICKER, ["Ctrl+h", "Ctrl+.", ""];
	PickerFilter, "Toggle filter", Explorer, Some(Subsection::Display), 1, PICKER, ["Ctrl+f", "", ""];
	ExplorerParent, "Parent directory", Explorer, Some(Subsection::Locations), 2, &[Explorer], ["Left", "Backspace", "h"];
	ExplorerEnter, "Enter directory", Explorer, Some(Subsection::Locations), 3, &[Explorer], ["Right", "l", ""];
	ExplorerUp, "Previous entry", Explorer, Some(Subsection::Navigation), 0, &[Explorer], ["Up", "k", ""];
	ExplorerDown, "Next entry", Explorer, Some(Subsection::Navigation), 1, &[Explorer], ["Down", "j", ""];
	ExplorerFirst, "First entry", Explorer, Some(Subsection::Navigation), 2, &[Explorer], ["Home", "g g", ""];
	ExplorerLast, "Last entry", Explorer, Some(Subsection::Navigation), 3, &[Explorer], ["End", "G", ""];
	ExplorerMiddle, "Middle visible entry", Explorer, Some(Subsection::Navigation), 4, &[Explorer], ["M", "", ""];
	ExplorerPageUp, "Previous page", Explorer, Some(Subsection::Navigation), 5, &[Explorer], ["PageUp", "", ""];
	ExplorerPageDown, "Next page", Explorer, Some(Subsection::Navigation), 6, &[Explorer], ["PageDown", "", ""];
	FieldDeleteAfter, "Delete", Workspace, Some(Subsection::TextFields), 51, &[Location, Filename], ["Delete", "", ""];
	Positive, "Positive action", Dialogs, Some(Subsection::General), 0, RESPONSES, ["s", "y", ""];
	Negative, "Negative action", Dialogs, Some(Subsection::General), 1, RESPONSES, ["d", "n", ""];
	Abort, "Abort action", Dialogs, Some(Subsection::General), 2, RESPONSES, ["a", "c", ""];
	Close, "Close / dismiss", Dialogs, Some(Subsection::General), 3, &[Menu, Prompt, ReviewContent, ReviewButtons, Explorer, Location, Filename, Format, FileButtons], ["Esc", "", ""];
	NextField, "Next field", Workspace, Some(Subsection::FieldNavigation), 0, FOCUSABLE, ["Tab", "", ""];
	PreviousField, "Previous field", Workspace, Some(Subsection::FieldNavigation), 1, FOCUSABLE, ["Shift+Tab", "", ""];
	FieldLeft, "Left", Workspace, Some(Subsection::FieldNavigation), 2, DIRECTIONAL, ["Left", "", ""];
	FieldRight, "Right", Workspace, Some(Subsection::FieldNavigation), 3, DIRECTIONAL, ["Right", "", ""];
	FieldUp, "Up", Workspace, Some(Subsection::FieldNavigation), 4, VERTICAL, ["Up", "", ""];
	FieldDown, "Down", Workspace, Some(Subsection::FieldNavigation), 5, VERTICAL, ["Down", "", ""];
	FieldPageUp, "Page up", Workspace, Some(Subsection::FieldNavigation), 6, PAGED, ["PageUp", "", ""];
	FieldPageDown, "Page down", Workspace, Some(Subsection::FieldNavigation), 7, PAGED, ["PageDown", "", ""];
	FieldHome, "Beginning", Workspace, Some(Subsection::FieldNavigation), 8, BOUNDARY, ["Home", "", ""];
	FieldEnd, "End", Workspace, Some(Subsection::FieldNavigation), 9, BOUNDARY, ["End", "", ""];
	Activate, "Activate", Workspace, Some(Subsection::FieldActions), 10, ACTIVATABLE, ["Enter", "Space", ""];
	Submit, "Submit", Workspace, Some(Subsection::FieldActions), 11, TEXT_FIELDS, ["Enter", "", ""];
	RangeDecrease, "Decrease", Workspace, Some(Subsection::RangeControls), 30, &[Channel], ["Left", "", ""];
	RangeIncrease, "Increase", Workspace, Some(Subsection::RangeControls), 31, &[Channel], ["Right", "", ""];
	RangeDecrease10, "Decrease by 10", Workspace, Some(Subsection::RangeControls), 32, &[Channel], ["Shift+Left", "", ""];
	RangeIncrease10, "Increase by 10", Workspace, Some(Subsection::RangeControls), 33, &[Channel], ["Shift+Right", "", ""];
	RangeMinimum, "Minimum", Workspace, Some(Subsection::RangeControls), 34, &[Channel], ["Home", "", ""];
	RangeMaximum, "Maximum", Workspace, Some(Subsection::RangeControls), 35, &[Channel], ["End", "", ""];
	FieldDeleteBefore, "Backspace", Workspace, Some(Subsection::TextFields), 50, TEXT_FIELDS, ["Backspace", "", ""];
	Copy, "Copy", Workspace, Some(Subsection::TextFields), 60, &[Editor, Code, Location, Filename, Hex, Channel], ["Ctrl+c", "", ""];
	Paste, "Paste", Workspace, Some(Subsection::TextFields), 61, &[Editor, Explorer, Location, Filename, Format, FileButtons, Hex, Channel], ["Ctrl+v", "", ""];
	Cut, "Cut", Workspace, Some(Subsection::TextFields), 63, &[Editor], ["Ctrl+x", "", ""];
	PastePlain, "Paste without formatting", Workspace, Some(Subsection::TextFields), 64, &[Editor], ["", "", ""];
	PasteLiteral, "Paste literal text", Workspace, Some(Subsection::TextFields), 65, &[Editor], ["", "", ""];
	SelectAll, "Select all", Workspace, Some(Subsection::TextFields), 62, &[Editor, Location, Filename, Hex, Channel], ["Ctrl+a", "", ""];
}
pub fn category_actions(section: SettingsSection) -> Vec<&'static ActionSpec> {
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
