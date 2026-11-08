//! Session, pane composition, geometry and focus. Modal and runtime policy belongs to App.
mod mouse;
mod panes;
pub mod shortcuts;
mod tools;
mod view;

pub use tools::Effect;
pub use view::Chrome;

use crate::{libraries::ansi::Document, panel::Panel};
use ratatui::layout::Rect;

/// Logical focus is independent of the wide/slim pane arrangement.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum Pane {
	#[default]
	Editor,
	Tools,
	Code,
}

impl Pane {
	fn next(self) -> Self {
		match self {
			Self::Editor => Self::Tools,
			Self::Tools => Self::Code,
			Self::Code => Self::Editor,
		}
	}
}

/// Only document panes support collapse and wrapping.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum TextPane {
	Editor,
	Code,
}

pub struct Workspace {
	pub(crate) session: crate::session::DocumentSession,
	pub(crate) editor: crate::features::editor::Editor,
	pub(crate) code: crate::features::code::Code,
	pub(crate) panel: Panel,
	pub(crate) editor_open: bool,
	pub(crate) code_open: bool,
	pub(crate) slim: bool,
	pub(crate) slim_tab: usize,
	pane_toggles: [Rect; 2],
	pub(crate) slim_tabs: Vec<(Rect, usize)>,
	editor_wrap_toggle: Rect,
	code_wrap_toggle: Rect,
	pub(crate) focus: Pane,
	code_toggle: Rect,
	dragging: bool,
}

impl Default for Workspace {
	fn default() -> Self {
		let mut editor = crate::features::editor::Editor::default();
		editor.surface.viewport.wrap = true;
		Self {
			session: Default::default(),
			editor,
			code: Default::default(),
			panel: Default::default(),
			editor_open: true,
			code_open: true,
			slim: false,
			slim_tab: 0,
			pane_toggles: [Rect::default(); 2],
			slim_tabs: Vec::new(),
			editor_wrap_toggle: Default::default(),
			code_wrap_toggle: Default::default(),
			focus: Default::default(),
			code_toggle: Default::default(),
			dragging: Default::default(),
		}
	}
}

impl Workspace {
	pub(crate) fn cancel_pointer_interactions(&mut self) {
		self.dragging = false;
		self.panel.scrollbar_drag = None;
		self.editor.surface.cancel_drag();
		self.code.surface.cancel_drag();
		self.panel.color.gauge_drag = None;
	}

	pub fn replace(
		&mut self,
		document: Document,
		path: Option<std::path::PathBuf>,
		imported: bool,
	) {
		self.session = crate::session::DocumentSession::new(document, path, imported);
		self.editor_open = true;
		self.editor.surface.viewport.reset();
		self.code.surface.viewport.top = 0;
		self.code.surface.viewport.left = 0;
		self.focus = Pane::Editor;
		self.cancel_pointer_interactions();
	}

	pub fn input_scope(&self) -> crate::shortcuts::InputScope {
		use crate::shortcuts::InputScope as S;
		match self.focus {
			Pane::Editor => S::Editor,
			Pane::Code => S::Code,
			Pane::Tools => self.panel.input_scope(self.panel.selected),
		}
	}
}

#[cfg(test)]
mod tests;
