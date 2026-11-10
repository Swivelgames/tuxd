//! A buffer owns its session and both projections; workspace layout stays shared.
use crate::{
	features::{code::Code, editor::Editor},
	libraries::ansi::Document,
	session::{DocumentSession, SessionId},
};
use std::path::{Path, PathBuf};

pub(crate) struct Buffer {
	pub session: DocumentSession,
	pub editor: Editor,
	pub code: Code,
	pub clipboard: crate::features::editor::clipboard::Clipboard,
	untitled: usize,
}

impl Buffer {
	pub fn new(document: Document, path: Option<PathBuf>, imported: bool, untitled: usize) -> Self {
		let mut editor = Editor::default();
		editor.surface.viewport.wrap = true;
		Self {
			session: DocumentSession::new(document, path, imported),
			editor,
			code: Code::default(),
			clipboard: Default::default(),
			untitled,
		}
	}

	fn is_initial_placeholder(&self) -> bool {
		self.untitled == 1
			&& self.session.path.is_none()
			&& self.session.origin.is_none()
			&& self.session.editing.document.is_empty()
			&& !self.session.dirty()
			&& self.session.editing.pending.is_none()
	}

	pub fn name(&self) -> String {
		use unicode_segmentation::UnicodeSegmentation;
		let name = self
			.session
			.directory_source()
			.and_then(Path::file_name)
			.map(|name| crate::libraries::text::display::sanitize(&name.to_string_lossy()))
			.unwrap_or_else(|| format!("untitled {}", self.untitled))
			.replace('\n', "\\n")
			.replace('\t', "\\t");
		name.graphemes(true)
			.map(crate::libraries::text::display::display_grapheme)
			.collect()
	}
}

impl super::Workspace {
	pub(crate) fn buffers(&self) -> impl Iterator<Item = &Buffer> {
		self.inactive[..self.active_index]
			.iter()
			.chain(std::iter::once(&self.active))
			.chain(self.inactive[self.active_index..].iter())
	}

	pub(crate) fn buffers_mut(&mut self) -> impl Iterator<Item = &mut Buffer> {
		let (before, after) = self.inactive.split_at_mut(self.active_index);
		before
			.iter_mut()
			.chain(std::iter::once(&mut self.active))
			.chain(after.iter_mut())
	}

	pub(crate) fn select_buffer(&mut self, index: usize) {
		if index == self.active_index || index > self.inactive.len() {
			return;
		}
		self.cancel_pointer_interactions();
		self.active.session.editing.break_group();

		// The active buffer occupies a logical tab position, but is not in inactive.
		let stored_index = if index > self.active_index {
			index - 1
		} else {
			index
		};
		let next = self.inactive.remove(stored_index);
		let previous = std::mem::replace(&mut self.active, next);

		// Removing the selected tab shifts the old active insertion point when moving left.
		let previous_index = if index < self.active_index {
			self.active_index - 1
		} else {
			self.active_index
		};
		self.inactive.insert(previous_index, previous);
		self.active_index = index;

		self.active.editor.surface.hide();
		self.active.code.surface.hide();
	}

	pub(crate) fn select_session(&mut self, id: SessionId) -> bool {
		let index = self.buffers().position(|buffer| buffer.session.id == id);
		if let Some(index) = index {
			self.select_buffer(index);
		}
		index.is_some()
	}

	pub(crate) fn cycle_buffer(&mut self, forward: bool) {
		let count = self.inactive.len() + 1;
		let index = (self.active_index + if forward { 1 } else { count - 1 }) % count;
		self.select_buffer(index);
	}

	pub(crate) fn add_buffer(&mut self, document: Document, path: Option<PathBuf>, imported: bool) {
		self.cancel_pointer_interactions();
		self.active.session.editing.break_group();
		let next = Buffer::new(document, path, imported, self.next_untitled);
		self.next_untitled += 1;
		let previous = std::mem::replace(&mut self.active, next);
		self.inactive.insert(self.active_index, previous);
		self.active_index = self.inactive.len();
		self.editor_open = true;
		self.focus = super::Pane::Editor;
	}

	pub(crate) fn open_document(
		&mut self,
		document: Document,
		path: Option<PathBuf>,
		imported: bool,
	) {
		if self.active.is_initial_placeholder() {
			self.cancel_pointer_interactions();
			self.active = Buffer::new(document, path, imported, 1);
			self.editor_open = true;
			self.focus = super::Pane::Editor;
		} else {
			self.add_buffer(document, path, imported);
		}
	}

	pub(crate) fn close_buffer(&mut self) {
		self.cancel_pointer_interactions();
		if self.inactive.is_empty() {
			self.active = Buffer::new(Document::default(), None, false, self.next_untitled);
			self.next_untitled += 1;
		} else {
			self.active_index = self.active_index.min(self.inactive.len() - 1);
			self.active = self.inactive.remove(self.active_index);
		}
		self.tab_start = self.tab_start.min(self.active_index);
	}

	pub(crate) fn native_buffer(&self, path: &Path) -> Option<SessionId> {
		let identity = path_identity(path);
		self.buffers()
			.find(|buffer| {
				buffer
					.session
					.path
					.as_ref()
					.is_some_and(|candidate| path_identity(candidate) == identity)
			})
			.map(|buffer| buffer.session.id)
	}
}

fn path_identity(path: &Path) -> PathBuf {
	std::fs::canonicalize(path).unwrap_or_else(|_| {
		let parent = path
			.parent()
			.filter(|parent| !parent.as_os_str().is_empty())
			.unwrap_or(Path::new("."));
		match (std::fs::canonicalize(parent), path.file_name()) {
			(Ok(parent), Some(name)) => parent.join(name),
			_ => path.to_owned(),
		}
	})
}
