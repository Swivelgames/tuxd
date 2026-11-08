//! One document's identity, editing state, and saved/file association.
use crate::libraries::{ansi::Document, editing::Editing};
use std::{
	path::{Path, PathBuf},
	sync::atomic::{AtomicU64, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionId(u64);

pub struct DocumentSession {
	pub id: SessionId,
	pub editing: Editing,
	pub path: Option<PathBuf>,
	pub origin: Option<PathBuf>,
	saved: Option<Document>,
}
impl Default for DocumentSession {
	fn default() -> Self {
		Self::new(Document::default(), None, false)
	}
}
impl DocumentSession {
	pub fn new(document: Document, path: Option<PathBuf>, imported: bool) -> Self {
		static NEXT: AtomicU64 = AtomicU64::new(1);
		Self {
			id: SessionId(NEXT.fetch_add(1, Ordering::Relaxed)),
			saved: (!imported).then(|| document.clone()),
			editing: Editing::new(document),
			origin: path.clone(),
			path: if imported { None } else { path },
		}
	}
	pub fn dirty(&self) -> bool {
		self.saved.as_ref() != Some(&self.editing.document)
	}
	pub fn directory_source(&self) -> Option<&Path> {
		self.path.as_deref().or(self.origin.as_deref())
	}
	pub fn saved(&mut self, path: PathBuf) {
		if self.path.is_none() && self.origin.is_none() {
			self.origin = Some(path.clone());
		}
		self.path = Some(path);
		self.saved = Some(self.editing.document.clone());
	}
	pub fn exported(&mut self, path: PathBuf) {
		if self.path.is_none() && self.origin.is_none() {
			self.origin = Some(path);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::libraries::editing::EditKind;
	use std::time::Instant;

	#[test]
	fn identity_is_distinct_from_revision_and_saved_content() {
		let mut session = DocumentSession::default();
		let id = session.id;
		assert!(!session.dirty());
		session
			.editing
			.insert("saved", EditKind::Atomic, Instant::now());
		session.saved(PathBuf::from("/tmp/document.tuxd.json"));
		let checkpoint = session.editing.document.clone();
		session
			.editing
			.insert(" change", EditKind::Atomic, Instant::now());
		assert!(session.dirty());
		session.editing.undo(false);
		assert!(!session.dirty());
		assert_eq!(session.id, id);
		session.editing.undo(true);
		assert!(session.dirty());
		let replacement = DocumentSession::new(checkpoint, session.path.clone(), false);
		assert_ne!(replacement.id, session.id);
		assert!(!replacement.dirty());
	}

	#[test]
	fn imported_and_exported_origins_preserve_file_dialog_policy() {
		let origin = PathBuf::from("/tmp/source.ansi");
		let mut imported = DocumentSession::new(Document::default(), Some(origin.clone()), true);
		assert!(imported.dirty());
		assert!(imported.path.is_none());
		assert_eq!(imported.directory_source(), Some(origin.as_path()));
		imported.exported(PathBuf::from("/other/export.ansi"));
		assert_eq!(imported.origin, Some(origin));
		let saved = PathBuf::from("/saved/document.tuxd.json");
		imported.saved(saved.clone());
		assert!(!imported.dirty());
		assert_eq!(imported.directory_source(), Some(saved.as_path()));
	}
}
