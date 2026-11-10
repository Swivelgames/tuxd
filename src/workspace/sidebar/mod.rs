//! Directory browsing and Open Files; document operations remain application effects.
mod input;
mod tree;
mod view;

use crate::{components::scrollbar::Bar, session::SessionId};
use ratatui::layout::Rect;
use std::path::{Path, PathBuf};
use tree::Tree;

#[derive(Clone)]
enum Row {
	OpenFiles,
	Spacer,
	Buffer {
		id: SessionId,
		label: String,
		active: bool,
		dirty: bool,
	},
	Root,
	Entry(tree::Entry),
}

impl Row {
	fn same_target(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::OpenFiles, Self::OpenFiles) | (Self::Root, Self::Root) => true,
			(Self::Buffer { id: left, .. }, Self::Buffer { id: right, .. }) => left == right,
			(Self::Entry(left), Self::Entry(right)) => left.path == right.path,
			_ => false,
		}
	}
}

pub(super) enum Request {
	None,
	Select(SessionId),
	Close(SessionId),
	Open(PathBuf),
	Notice(String),
}

pub(super) struct Sidebar {
	pub visible: bool,
	tree: Option<Tree>,
	open_files: bool,
	root_open: bool,
	selected: usize,
	top: usize,
	area: Rect,
	bars: Vec<Bar>,
	drag: Option<usize>,
	rows: Vec<Row>,
}

impl Default for Sidebar {
	fn default() -> Self {
		Self {
			visible: false,
			tree: None,
			open_files: true,
			root_open: true,
			selected: 0,
			top: 0,
			area: Rect::default(),
			bars: Vec::new(),
			drag: None,
			rows: Vec::new(),
		}
	}
}

impl Sidebar {
	fn skip_spacer(&mut self, backwards: bool) {
		if matches!(self.rows.get(self.selected), Some(Row::Spacer)) {
			if backwards {
				self.selected -= 1;
			} else {
				self.selected += 1;
			}
		}
	}

	pub fn root(&self) -> Option<&Path> {
		self.tree.as_ref().map(|tree| tree.root.as_path())
	}

	pub fn open(&mut self, path: &Path) -> std::io::Result<()> {
		let tree = Tree::open(path)?;
		self.tree = Some(tree);
		self.visible = true;
		self.root_open = true;
		self.selected = 0;
		self.top = 0;
		Ok(())
	}

	pub fn hide_geometry(&mut self) {
		self.area = Rect::default();
		self.bars.clear();
		self.drag = None;
	}

	fn refresh_rows(&mut self, buffers: Vec<Row>) {
		let selected = self.rows.get(self.selected).cloned();
		self.rows.clear();
		self.rows.push(Row::OpenFiles);
		if self.open_files {
			self.rows.extend(buffers);
		}
		if let Some(tree) = &self.tree {
			self.rows.push(Row::Spacer);
			self.rows.push(Row::Root);
			if self.root_open {
				self.rows
					.extend(tree.entries.iter().cloned().map(Row::Entry));
			}
		}
		self.selected = selected
			.and_then(|selected| self.rows.iter().position(|row| row.same_target(&selected)))
			.unwrap_or(self.selected.min(self.rows.len().saturating_sub(1)));
		self.skip_spacer(false);
	}
}

impl super::Workspace {
	pub fn open_directory(&mut self, path: &Path) -> std::io::Result<()> {
		self.sidebar.open(path)?;
		self.cancel_pointer_interactions();
		self.active.session.editing.break_group();
		self.focus = super::Pane::Sidebar;
		Ok(())
	}

	pub(crate) fn directory(&self) -> Option<&Path> {
		self.sidebar.root()
	}

	fn refresh_sidebar_rows(&mut self) {
		let buffers = self
			.buffers()
			.map(|buffer| {
				let label = buffer
					.session
					.directory_source()
					.and_then(|path| {
						let path = super::buffers::path_identity(path);
						let relative = path.strip_prefix(self.sidebar.root()?).ok()?;
						Some(crate::libraries::text::display::sanitize(
							&relative.to_string_lossy(),
						))
					})
					.unwrap_or_else(|| buffer.name());
				Row::Buffer {
					id: buffer.session.id,
					label,
					dirty: buffer.session.dirty(),
					active: buffer.session.id == self.active.session.id,
				}
			})
			.collect();
		self.sidebar.refresh_rows(buffers);
	}

	fn sidebar_request(&mut self, request: Request) -> super::Effect {
		match request {
			Request::Select(id) => {
				self.select_session(id);
				self.focus = super::Pane::Editor;
				self.editor_open = true;
				if self.sidebar_overlay {
					self.sidebar.visible = false;
				}
				super::Effect::None
			}
			Request::Close(id) => super::Effect::CloseBuffer(id),
			Request::Open(path) => super::Effect::OpenFile(path),
			Request::Notice(message) => super::Effect::Notice(message),
			Request::None => super::Effect::None,
		}
	}

	pub(crate) fn sidebar_action(&mut self, action: crate::shortcuts::ActionId) -> super::Effect {
		self.refresh_sidebar_rows();
		let request = self.sidebar.action(action);
		self.sidebar_request(request)
	}
}

#[cfg(test)]
mod tests;
