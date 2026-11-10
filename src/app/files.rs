use super::App;
use crate::session::SessionId;
use crate::{
	files::{Action, Dialog, MENU},
	libraries::ansi::{Document, Mode},
	persistence,
};
use std::{collections::VecDeque, path::PathBuf};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum BatchMode {
	SaveAll,
	Quit,
}

pub(super) enum Pending {
	Close(SessionId),
	Batch {
		remaining: VecDeque<SessionId>,
		mode: BatchMode,
		original: SessionId,
	},
}

impl App {
	pub(crate) fn file_command(&mut self, action: crate::shortcuts::ActionId) -> bool {
		use crate::shortcuts::ActionId as A;
		let command = match action {
			A::New => Action::New,
			A::Open => Action::Open,
			A::OpenDirectory => Action::OpenDirectory,
			A::Save => Action::Save,
			A::SaveAs => Action::SaveAs,
			A::SaveAll => Action::SaveAll,
			A::CloseBuffer => Action::CloseBuffer,
			A::NextBuffer | A::PreviousBuffer => {
				self.shortcuts.cancel();
				self.workspace.cycle_buffer(action == A::NextBuffer);
				return true;
			}
			A::Import => Action::ImportEscaped,
			A::Export => Action::ExportEscaped,
			A::Quit => Action::Quit,
			A::FileMenu => {
				self.open_file_menu();
				return true;
			}
			_ => return false,
		};
		self.file_action(command);
		true
	}

	pub fn dirty(&self) -> bool {
		self.workspace.active.session.dirty()
	}

	pub fn load_document(&mut self, document: Document, path: Option<PathBuf>, imported: bool) {
		self.workspace.active.clipboard.cancel_edits();
		self.import_review = None;
		self.workspace.replace(document, path, imported);
		self.notice = Some(
			if imported {
				"ANSI imported · unsaved document"
			} else {
				"Document opened"
			}
			.into(),
		);
	}

	pub fn open_file_menu(&mut self) {
		self.file_dialog(Dialog::Menu { index: 0 });
	}

	pub(crate) fn file_dialog(&mut self, dialog: Dialog) {
		self.shortcuts.cancel();
		self.files.open(dialog);
		self.files.document_name = self.workspace.active.name();
		self.workspace.cancel_pointer_interactions();
		self.workspace.active.session.editing.break_group();
	}

	fn cancel_file(&mut self) {
		self.import_review = None;
		self.shortcuts.cancel();
		self.files.cancel();
		self.pending_file = None;
	}

	fn finish_pending(&mut self) {
		self.files.dialog = None;
		match self.pending_file.take() {
			Some(Pending::Close(id)) if self.workspace.select_session(id) => {
				self.workspace.close_buffer();
			}
			Some(Pending::Batch {
				remaining,
				mode,
				original,
			}) => {
				self.continue_batch(remaining, mode, original);
			}
			Some(Pending::Close(_)) | None => (),
		}
	}

	fn continue_batch(
		&mut self,
		mut remaining: VecDeque<SessionId>,
		mode: BatchMode,
		original: SessionId,
	) {
		while let Some(id) = remaining.pop_front() {
			if !self.workspace.select_session(id) || !self.dirty() {
				continue;
			}

			// Associated files can save immediately; failures already open a retry dialog.
			let saved = if mode == BatchMode::SaveAll
				&& let Some(path) = self.workspace.active.session.path.clone()
			{
				self.publish_file(Action::Save, path, true)
			} else {
				if mode == BatchMode::Quit {
					self.file_dialog(Dialog::Dirty);
				} else {
					self.path_dialog(Action::SaveAs);
				}
				false
			};
			if saved {
				continue;
			}

			// The current dialog owns this buffer; resume with the rest only after it succeeds.
			self.pending_file = Some(Pending::Batch {
				remaining,
				mode,
				original,
			});
			return;
		}

		self.workspace.select_session(original);
		self.exiting = mode == BatchMode::Quit;
	}

	fn begin_batch(&mut self, mode: BatchMode) {
		// Cancel delayed edits before deciding which documents need saving.
		for buffer in self.workspace.buffers_mut() {
			buffer.clipboard.cancel_edits();
		}
		let remaining = self
			.workspace
			.buffers()
			.filter(|buffer| buffer.session.dirty())
			.map(|buffer| buffer.session.id)
			.collect();
		self.pending_file = Some(Pending::Batch {
			remaining,
			mode,
			original: self.workspace.active.session.id,
		});
		self.finish_pending();
	}

	fn open_buffer(&mut self, document: Document, path: Option<PathBuf>, imported: bool) {
		self.shortcuts.cancel();
		self.files.cancel();
		self.workspace.open_document(document, path, imported);
		self.notice = Some(
			if imported {
				"ANSI imported · unsaved document"
			} else {
				"Document opened"
			}
			.into(),
		);
	}

	pub(crate) fn add_import(&mut self, document: Document, path: Option<PathBuf>) {
		self.open_buffer(document, path, true);
	}

	fn path_dialog(&mut self, action: Action) {
		let text = match action {
			Action::Save | Action::SaveAs => self
				.workspace
				.active
				.session
				.path
				.as_ref()
				.map(|p| p.to_string_lossy().into_owned())
				.unwrap_or_default(),
			_ => String::new(),
		};
		let source = self.workspace.active.session.directory_source();
		let mut picker = crate::files::picker::Picker::new(action, &text, source);
		if source.is_none()
			&& let Some(root) = self.workspace.directory()
		{
			picker.navigate(root.to_owned());
		}
		self.file_dialog(Dialog::Path {
			picker: Box::new(picker),
		});
	}

	pub fn file_action(&mut self, action: Action) {
		self.files.error = None;
		match action {
			Action::New => {
				self.shortcuts.cancel();
				self.files.cancel();
				self.workspace.add_buffer(Document::default(), None, false);
				self.notice = Some("New document".into());
			}
			Action::Quit => self.begin_batch(BatchMode::Quit),
			Action::SaveAll => self.begin_batch(BatchMode::SaveAll),
			Action::CloseBuffer => {
				self.workspace.active.clipboard.cancel_edits();
				self.pending_file = Some(Pending::Close(self.workspace.active.session.id));
				if self.dirty() {
					self.file_dialog(Dialog::Dirty);
				} else {
					self.finish_pending();
				}
			}
			Action::Save => {
				if let Some(path) = self.workspace.active.session.path.clone() {
					self.write_file(Action::Save, path, true);
				} else {
					self.path_dialog(Action::SaveAs);
				}
			}
			_ => self.path_dialog(action),
		}
	}

	fn file_failure(&mut self, action: Action, path: PathBuf, error: impl std::fmt::Display) {
		let text = path.to_string_lossy().into_owned();
		let picker = crate::files::picker::Picker::new(
			action,
			&text,
			self.workspace.active.session.directory_source(),
		);
		self.file_dialog(Dialog::Path {
			picker: Box::new(picker),
		});
		self.files.error = Some(format!("{error}"));
		self.files.reveal_error();
	}

	pub(crate) fn write_file(&mut self, action: Action, path: PathBuf, replace: bool) {
		if self.publish_file(action, path, replace) {
			self.finish_pending();
		}
	}

	fn publish_file(&mut self, action: Action, path: PathBuf, replace: bool) -> bool {
		use crate::workflows::publication::{self, Format};
		let format = match action {
			Action::ExportRaw => Format::Raw,
			Action::ExportEscaped => Format::Escaped,
			_ => Format::Native,
		};
		if self
			.workspace
			.native_buffer(&path)
			.is_some_and(|id| id != self.workspace.active.session.id)
		{
			self.file_failure(action, path, "This file is already open in another buffer");
			return false;
		}
		let result = publication::publish(
			&mut self.workspace.active.session,
			path.clone(),
			format,
			replace,
		);
		match result {
			Ok(()) => {
				self.files.error = None;
				self.notice = Some(format.notice().into());
				return true;
			}
			Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && !replace => {
				self.file_dialog(Dialog::Overwrite { action, path })
			}
			Err(error) => self.file_failure(action, path, error),
		}
		false
	}

	pub(crate) fn submit_path(&mut self, action: Action, mut path: PathBuf) {
		if action == Action::OpenDirectory {
			match self.workspace.open_directory(&path) {
				Ok(()) => {
					self.files.cancel();
					self.shortcuts.cancel();
				}
				Err(error) => self.file_failure(action, path, error),
			}
			return;
		}
		if path.as_os_str().is_empty() {
			self.files.error = Some("Enter a file path".into());
			return;
		}
		if matches!(action, Action::Save | Action::SaveAs) && path.extension().is_none() {
			path.set_extension("tuxd.json");
		}
		if action == Action::Open
			&& let Some(id) = self.workspace.native_buffer(&path)
		{
			self.files.cancel();
			self.workspace.select_session(id);
			self.workspace.focus = crate::workspace::Pane::Editor;
			self.workspace.editor_open = true;
			return;
		}
		if action == Action::Open
			&& !path
				.file_name()
				.is_some_and(|name| name.to_string_lossy().ends_with(".tuxd.json"))
		{
			let mut picker = crate::files::picker::Picker::new(
				Action::ImportEscaped,
				&path.to_string_lossy(),
				self.workspace.active.session.path.as_deref(),
			);
			picker.offer_import();
			self.file_dialog(Dialog::Path {
				picker: Box::new(picker),
			});
			return;
		}

		match action {
			Action::Open => match persistence::load(&path) {
				Ok(document) => self.open_buffer(document, Some(path), false),
				Err(error) => self.file_failure(action, path, error),
			},
			Action::ImportRaw | Action::ImportEscaped => {
				let result = persistence::read_file(&path)
					.map_err(|e| e.to_string())
					.and_then(|bytes| {
						crate::workflows::parse_bytes(
							bytes,
							if action == Action::ImportRaw {
								Mode::Raw
							} else {
								Mode::Escaped
							},
						)
					});
				match result {
					Ok(import) => {
						self.import_document(import, crate::workflows::Source::File(path.clone()))
					}
					Err(error) => self.file_failure(action, path, error),
				}
			}
			_ => self.write_file(action, path, false),
		}
	}

	fn activate_file(&mut self, choice: usize) {
		let Some(dialog) = self.files.dialog.take() else {
			return;
		};
		match dialog {
			Dialog::Menu { .. } => self.file_action(MENU[choice.min(MENU.len() - 1)].0),

			Dialog::Dirty => match choice {
				0 => self.file_action(Action::Save),
				1 => self.finish_pending(),
				_ => self.cancel_file(),
			},
			Dialog::Overwrite { action, path } => {
				if choice == 0 {
					self.write_file(action, path, true);
				} else {
					self.cancel_file();
				}
			}
			Dialog::Path { picker } => {
				let path = picker.destination();
				let action = picker.action();
				self.files.dialog = Some(Dialog::Path { picker });
				self.submit_path(action, path);
			}
		}
	}

	pub fn file_shortcut(&mut self, action: crate::shortcuts::ActionId) {
		let effect = self.files.shortcut(action);
		self.file_effect(effect);
	}

	pub fn file_event(&mut self, event: &crossterm::event::Event) -> Option<bool> {
		self.files.dialog.as_ref()?;
		let effect = self.files.event(event);
		self.file_effect(effect);
		Some(self.exiting)
	}

	fn file_effect(&mut self, effect: crate::files::Effect) {
		use crate::files::Effect;
		match effect {
			Effect::None => (),
			Effect::Cancel => self.cancel_file(),
			Effect::Activate(choice) => self.activate_file(choice),
			Effect::Submit(selection) => self.submit_path(selection.action, selection.path),
			Effect::Copy(text) => self.copy_field(text),
		}
	}
}
