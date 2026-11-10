use crate::{
	components::import_review::{Decision, Presentation, Review},
	libraries::ansi::{self, Import, Mode},
};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum Source {
	File(PathBuf),
	Stdin,
	Clipboard,
}
impl Source {
	pub fn path(&self) -> Option<PathBuf> {
		match self {
			Self::File(path) => Some(path.clone()),
			_ => None,
		}
	}
}
pub fn parse_bytes(bytes: Vec<u8>, mode: Mode) -> Result<Import, String> {
	let text = String::from_utf8(bytes).map_err(|_| "Import requires UTF-8 text")?;
	ansi::parse(&text, mode)
}

pub enum Destination {
	Document,
	Paste(crate::features::editor::clipboard::PasteReview),
}
pub struct PendingReview {
	pub import: Import,
	pub view: Review,
	source: Source,
	destination: Destination,
}
pub enum Commit {
	Document {
		document: crate::libraries::ansi::Document,
		path: Option<PathBuf>,
	},
	Paste {
		document: crate::libraries::ansi::Document,
		target: crate::features::editor::clipboard::PasteReview,
	},
}
impl PendingReview {
	pub fn new(import: Import, source: Source, destination: Destination) -> Self {
		Self {
			view: Review::default(),
			import,
			source,
			destination,
		}
	}
	pub fn decide(self, decision: Decision) -> Option<Commit> {
		if decision == Decision::Abort {
			return None;
		}
		Some(match self.destination {
			Destination::Document => Commit::Document {
				document: self.import.document,
				path: self.source.path(),
			},
			Destination::Paste(target) => Commit::Paste {
				document: self.import.document,
				target,
			},
		})
	}

	pub fn draw(&mut self, frame: &mut ratatui::Frame) {
		let cwd = std::env::current_dir().ok();
		let title = match &self.source {
			Source::File(path) => cwd
				.as_deref()
				.and_then(|cwd| path.strip_prefix(cwd).ok())
				.unwrap_or(path)
				.to_string_lossy()
				.into_owned(),
			Source::Stdin => "Standard input".into(),
			Source::Clipboard => "Clipboard".into(),
		};
		let explanation = match self.source {
			Source::File(_) => "The original file will be unaffected.",
			Source::Stdin => "Imported content will open as an unsaved document.",
			Source::Clipboard => "The clipboard will be unaffected.",
		};
		let accept = match self.destination {
			Destination::Paste(_) => "Strip and paste",
			Destination::Document => "Import anyway",
		};
		self.view.draw(
			frame,
			&self.import,
			Presentation {
				title: &title,
				explanation,
				accept,
			},
		);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{app::App, features::editor::clipboard::PasteMode, libraries::editing::EditKind};
	use std::time::Instant;

	#[test]
	fn reviewing_clipboard_is_independent_of_files_and_revalidates_before_commit() {
		let mut app = App::default();
		app.paste_editor_text("old", PasteMode::Styled);
		app.paste_editor_text(r"\x1b[31mnew\x1b[2J", PasteMode::Styled);
		assert!(app.files.dialog.is_none());
		assert!(matches!(
			app.import_review.as_ref().unwrap().source,
			Source::Clipboard
		));
		app.workspace
			.active
			.session
			.editing
			.insert(" changed", EditKind::Atomic, Instant::now());
		let before = app.workspace.active.session.editing.document.clone();
		app.decide_review(Decision::Strip);
		assert_eq!(app.workspace.active.session.editing.document, before);
		assert!(app.notice.as_ref().unwrap().contains("editor changed"));
		assert!(app.import_review.is_none());
	}

	#[test]
	fn imported_sources_share_review_but_keep_their_document_origin() {
		for source in [
			Source::Stdin,
			Source::File(PathBuf::from("/tmp/source.ansi")),
		] {
			let mut app = App::default();
			let import = parse_bytes(br"new\x1b[2J".to_vec(), Mode::Escaped).unwrap();
			app.import_document(import, source.clone());
			assert_eq!(app.workspace.active.session.editing.document.lines(), [""]);
			app.decide_review(Decision::Strip);
			assert_eq!(
				app.workspace.active.session.editing.document.lines(),
				["new"]
			);
			assert!(app.workspace.active.session.path.is_none());
			assert_eq!(app.workspace.active.session.origin, source.path());
			assert!(app.workspace.active.session.dirty());
		}
		assert!(
			parse_bytes(vec![0xff], Mode::Raw)
				.err()
				.unwrap()
				.contains("UTF-8")
		);
	}
}
