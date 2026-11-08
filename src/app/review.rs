use super::App;
use crate::{
	components::import_review::Decision,
	libraries::ansi::Import,
	workflows::{Destination, PendingReview, Source},
};
impl App {
	pub fn import_document(&mut self, import: Import, source: Source) {
		if import.issues.is_empty() {
			self.replace_import(import.document, source.path());
		} else {
			self.begin_review(import, source, Destination::Document);
		}
	}
	pub fn begin_review(&mut self, import: Import, source: Source, destination: Destination) {
		self.shortcuts.cancel();
		self.files.dialog = None;
		self.workspace.cancel_pointer_interactions();
		self.workspace.session.editing.break_group();
		self.import_review = Some(PendingReview::new(import, source, destination));
	}
	pub fn decide_review(&mut self, decision: Decision) {
		let Some(review) = self.import_review.take() else {
			return;
		};
		self.shortcuts.cancel();
		match review.decide(decision) {
			Some(crate::workflows::Commit::Document { document, path }) => {
				self.replace_import(document, path)
			}
			Some(crate::workflows::Commit::Paste { document, target }) => {
				self.finish_paste_review(target, &document)
			}
			None => (),
		}
	}
}
