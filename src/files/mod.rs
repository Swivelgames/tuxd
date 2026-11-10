mod input;
pub mod picker;
pub mod shortcuts;
mod view;
pub use input::Effect;
pub use view::draw;
// File actions and modal state. Candidate documents are validated before replacement.
use crate::components::scrollbar::Bar;
use ratatui::layout::Rect;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
	New,
	Open,
	OpenDirectory,
	Save,
	SaveAs,
	SaveAll,
	CloseBuffer,
	ImportRaw,
	ImportEscaped,
	ExportRaw,
	ExportEscaped,
	Quit,
}
impl Action {
	pub fn shortcut(self) -> crate::shortcuts::ActionId {
		use crate::shortcuts::ActionId as A;
		match self {
			Self::New => A::New,
			Self::Open => A::Open,
			Self::OpenDirectory => A::OpenDirectory,
			Self::Save => A::Save,
			Self::SaveAs => A::SaveAs,
			Self::SaveAll => A::SaveAll,
			Self::CloseBuffer => A::CloseBuffer,
			Self::ImportRaw | Self::ImportEscaped => A::Import,
			Self::ExportRaw | Self::ExportEscaped => A::Export,
			Self::Quit => A::Quit,
		}
	}
}
pub const MENU: [(Action, &str); 10] = [
	(Action::New, "New document"),
	(Action::Open, "Open document"),
	(Action::OpenDirectory, "Open directory"),
	(Action::CloseBuffer, "Close document"),
	(Action::Save, "Save"),
	(Action::SaveAs, "Save as"),
	(Action::SaveAll, "Save all"),
	(Action::ImportEscaped, "Import"),
	(Action::ExportEscaped, "Export"),
	(Action::Quit, "Quit"),
];

pub enum Dialog {
	Menu {
		index: usize,
	},
	Path {
		picker: Box<crate::files::picker::Picker>,
	},
	Dirty,
	Overwrite {
		action: Action,
		path: PathBuf,
	},
}

pub struct Files {
	pub dialog: Option<Dialog>,
	pub error: Option<String>,
	pub(crate) document_name: String,
	scroll: usize,
	viewport: usize,
	bar: Option<Bar>,
	buttons: Vec<(Rect, usize)>,
	choice: usize,
	drag: Option<usize>,
}
impl Files {
	pub fn poll(&mut self) -> bool {
		match &mut self.dialog {
			Some(Dialog::Path { picker }) => picker.poll_paste(),
			_ => false,
		}
	}
	pub fn open(&mut self, dialog: Dialog) {
		self.choice = if matches!(dialog, Dialog::Dirty) {
			2
		} else {
			0
		};
		self.dialog = Some(dialog);
		self.error = None;
		self.scroll = 0;
		self.bar = None;
		self.drag = None;
	}
	pub fn cancel(&mut self) {
		self.dialog = None;
		self.error = None;
		self.drag = None;
	}
	pub fn reveal_error(&mut self) {
		self.scroll = usize::MAX;
	}
}
impl Default for Files {
	fn default() -> Self {
		Self {
			dialog: None,
			error: None,
			document_name: String::new(),
			scroll: 0,
			viewport: 1,
			bar: None,
			buttons: Vec::new(),
			choice: 0,
			drag: None,
		}
	}
}

#[cfg(test)]
mod tests;

impl Files {
	pub fn input_scope(&self) -> Option<crate::shortcuts::InputScope> {
		use crate::shortcuts::InputScope as S;
		Some(match self.dialog.as_ref()? {
			Dialog::Path { picker, .. } => picker.input_scope(),
			Dialog::Menu { .. } => S::Menu,
			_ => S::Prompt,
		})
	}
}
