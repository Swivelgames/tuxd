use crate::shortcuts::registry::Category;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Section {
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
impl Section {
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

impl From<Category> for Section {
	fn from(category: Category) -> Self {
		match category {
			Category::Timeout => Self::Timeout,
			Category::Workspace => Self::Workspace,
			Category::Editor => Self::Editor,
			Category::Code => Self::Code,
			Category::Tools => Self::Tools,
			Category::Dialogs => Self::Dialogs,
			Category::Explorer => Self::Explorer,
		}
	}
}
impl Section {
	pub fn shortcut_category(self) -> Option<Category> {
		Some(match self {
			Self::Timeout => Category::Timeout,
			Self::Workspace => Category::Workspace,
			Self::Editor => Category::Editor,
			Self::Code => Category::Code,
			Self::Tools => Category::Tools,
			Self::Dialogs => Category::Dialogs,
			Self::Explorer => Category::Explorer,
			_ => return None,
		})
	}
}
