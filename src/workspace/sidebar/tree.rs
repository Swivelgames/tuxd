//! Lazy directory discovery, independent of buffers and presentation.
use std::{
	collections::BTreeSet,
	fs, io,
	path::{Path, PathBuf},
};

#[derive(Clone)]
pub(super) struct Entry {
	pub path: PathBuf,
	pub depth: usize,
	pub directory: bool,
	pub expanded: bool,
}

#[derive(Clone, Copy)]
pub(super) struct Filters {
	pub hidden: bool,
	pub native_only: bool,
}

pub(super) struct Tree {
	pub root: PathBuf,
	pub entries: Vec<Entry>,
	pub filters: Filters,
}

impl Tree {
	pub fn open(path: &Path) -> io::Result<Self> {
		let root = path.canonicalize()?;
		let mut tree = Self {
			root,
			entries: Vec::new(),
			filters: Filters {
				hidden: false,
				native_only: true,
			},
		};
		tree.refresh()?;
		Ok(tree)
	}

	fn children(&self, path: &Path, depth: usize) -> io::Result<Vec<Entry>> {
		let mut entries = Vec::new();
		for child in fs::read_dir(path)? {
			let child = child?;
			let kind = child.file_type()?;
			let name = child.file_name();
			let name = name.to_string_lossy();
			if !self.filters.hidden && name.starts_with('.') {
				continue;
			}
			// Do not descend into symlink directories or expose special files as documents.
			let directory = kind.is_dir();
			let file = kind.is_file() || (kind.is_symlink() && child.path().is_file());
			if !directory && (!file || (self.filters.native_only && !name.ends_with(".tuxd.json")))
			{
				continue;
			}
			entries.push(Entry {
				path: child.path(),
				depth,
				directory,
				expanded: false,
			});
		}
		entries.sort_by(|a, b| {
			b.directory
				.cmp(&a.directory)
				.then_with(|| a.path.file_name().cmp(&b.path.file_name()))
		});
		Ok(entries)
	}

	pub fn toggle(&mut self, path: &Path) -> io::Result<()> {
		let Some(index) = self.entries.iter().position(|entry| entry.path == path) else {
			return Ok(());
		};
		let entry = &self.entries[index];
		if !entry.directory {
			return Ok(());
		}
		if entry.expanded {
			let depth = entry.depth;
			let end = self.entries[index + 1..]
				.iter()
				.position(|child| child.depth <= depth)
				.map_or(self.entries.len(), |offset| index + 1 + offset);
			self.entries.drain(index + 1..end);
			self.entries[index].expanded = false;
		} else {
			let children = self.children(path, entry.depth + 1)?;
			self.entries[index].expanded = true;
			self.entries.splice(index + 1..index + 1, children);
		}
		Ok(())
	}

	pub fn refresh(&mut self) -> io::Result<()> {
		self.refresh_with(self.filters)
	}

	/// Publish filters and discovered entries together, only after discovery succeeds.
	pub fn refresh_with(&mut self, filters: Filters) -> io::Result<()> {
		let expanded: BTreeSet<_> = self
			.entries
			.iter()
			.filter(|entry| entry.expanded)
			.map(|entry| entry.path.clone())
			.collect();
		let mut next = Self {
			root: self.root.clone(),
			entries: Vec::new(),
			filters,
		};
		next.entries = next.children(&next.root, 0)?;
		let mut index = 0;
		while index < next.entries.len() {
			let path = next.entries[index].path.clone();
			if expanded.contains(&path) {
				next.toggle(&path)?;
			}
			index += 1;
		}
		*self = next;
		Ok(())
	}
}
