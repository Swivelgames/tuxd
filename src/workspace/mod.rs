//! Session, pane composition, geometry and focus. Modal and runtime policy belongs to App.
mod buffers;
mod layout;
mod mouse;
mod panes;
pub mod shortcuts;
mod tabs;
mod tools;
mod view;

pub use view::Chrome;

use crate::{libraries::ansi::Document, panel::Panel};
use ratatui::layout::Rect;

/// Requests that require application-owned services or modal workflows.
pub enum Effect {
	None,
	Copy(String),
	Paste(usize),
	PasteCharacters,
	Notice(String),
	CloseBuffer(crate::session::SessionId),
}

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
	pub(crate) active: buffers::Buffer,
	// Inactive buffers retain tab order; active_index is the gap occupied by active.
	inactive: Vec<buffers::Buffer>,
	active_index: usize,
	next_untitled: usize,
	tab_hits: Vec<(Rect, usize)>,
	tab_close_hits: Vec<(Rect, crate::session::SessionId)>,
	tab_start: usize,
	pub(crate) panel: Panel,
	pub(crate) editor_open: bool,
	pub(crate) code_open: bool,
	pub(crate) slim: bool,
	pub(crate) slim_tab: usize,
	pane_toggles: [Rect; 2],
	pub(crate) slim_tabs: Vec<(Rect, usize)>,
	code_wrap_toggle: Rect,
	pub(crate) focus: Pane,
	dragging: bool,
}

impl Default for Workspace {
	fn default() -> Self {
		Self {
			active: buffers::Buffer::new(Document::default(), None, false, 1),
			inactive: Vec::new(),
			active_index: 0,
			next_untitled: 2,
			tab_hits: Vec::new(),
			tab_close_hits: Vec::new(),
			tab_start: 0,
			panel: Default::default(),
			editor_open: true,
			code_open: true,
			slim: false,
			slim_tab: 0,
			pane_toggles: [Rect::default(); 2],
			slim_tabs: Vec::new(),
			code_wrap_toggle: Default::default(),
			focus: Default::default(),
			dragging: Default::default(),
		}
	}
}

impl Workspace {
	pub(crate) fn cancel_pointer_interactions(&mut self) {
		self.dragging = false;
		self.panel.scrollbar_drag = None;
		self.active.editor.surface.cancel_drag();
		self.active.code.surface.cancel_drag();
		self.panel.color.gauge_drag = None;
	}

	pub fn replace(
		&mut self,
		document: Document,
		path: Option<std::path::PathBuf>,
		imported: bool,
	) {
		self.active.session = crate::session::DocumentSession::new(document, path, imported);
		self.editor_open = true;
		self.active.editor.surface.viewport.reset();
		self.active.code.surface.viewport.top = 0;
		self.active.code.surface.viewport.left = 0;
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
