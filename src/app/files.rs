use super::App;
use crate::{
	files::{Action, Dialog, MENU},
	libraries::ansi::{Document, Mode},
	persistence,
};
use std::path::PathBuf;

pub(super) enum Pending {
	Replace {
		document: Document,
		path: Option<PathBuf>,
		imported: bool,
	},
	Quit,
}

impl App {
	pub(crate) fn file_command(&mut self, action: crate::shortcuts::ActionId) -> bool {
		use crate::shortcuts::ActionId as A;
		let command = match action {
			A::New => Action::New,
			A::Open => Action::Open,
			A::Save => Action::Save,
			A::SaveAs => Action::SaveAs,
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
		self.workspace.session.dirty()
	}

	pub fn load_document(&mut self, document: Document, path: Option<PathBuf>, imported: bool) {
		self.editor_clipboard.cancel_edits();
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
		self.workspace.cancel_pointer_interactions();
		self.workspace.session.editing.break_group();
	}

	fn cancel_file(&mut self) {
		self.import_review = None;
		self.shortcuts.cancel();
		self.files.cancel();
		self.pending_file = None;
	}

	fn finish_pending(&mut self) {
		self.files.dialog = None;
		if let Some(pending) = self.pending_file.take() {
			match pending {
				Pending::Replace {
					document,
					path,
					imported,
				} => self.load_document(document, path, imported),
				Pending::Quit => self.exiting = true,
			}
		}
	}

	pub(crate) fn replace_import(&mut self, document: Document, path: Option<PathBuf>) {
		self.replace_or_prompt(Pending::Replace {
			document,
			path,
			imported: true,
		});
	}

	fn replace_or_prompt(&mut self, pending: Pending) {
		self.pending_file = Some(pending);
		if self.dirty() {
			self.file_dialog(Dialog::Dirty);
		} else {
			self.finish_pending();
		}
	}

	fn path_dialog(&mut self, action: Action) {
		let text = match action {
			Action::Save | Action::SaveAs => self
				.workspace
				.session
				.path
				.as_ref()
				.map(|p| p.to_string_lossy().into_owned())
				.unwrap_or_default(),
			_ => String::new(),
		};
		let picker = crate::files::picker::Picker::new(
			action,
			&text,
			self.workspace.session.directory_source(),
		);
		self.file_dialog(Dialog::Path {
			picker: Box::new(picker),
		});
	}

	pub fn file_action(&mut self, action: Action) {
		self.files.error = None;
		match action {
			Action::New => self.replace_or_prompt(Pending::Replace {
				document: Document::default(),
				path: None,
				imported: false,
			}),
			Action::Quit => self.replace_or_prompt(Pending::Quit),
			Action::Save => {
				if let Some(path) = self.workspace.session.path.clone() {
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
			self.workspace.session.directory_source(),
		);
		self.file_dialog(Dialog::Path {
			picker: Box::new(picker),
		});
		self.files.error = Some(format!("{error}"));
		self.files.reveal_error();
	}

	pub(crate) fn write_file(&mut self, action: Action, path: PathBuf, replace: bool) {
		use crate::workflows::publication::{self, Format};
		let format = match action {
			Action::ExportRaw => Format::Raw,
			Action::ExportEscaped => Format::Escaped,
			_ => Format::Native,
		};
		let result =
			publication::publish(&mut self.workspace.session, path.clone(), format, replace);
		match result {
			Ok(()) => {
				self.files.error = None;
				self.notice = Some(format.notice().into());
				self.finish_pending();
			}
			Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && !replace => {
				self.file_dialog(Dialog::Overwrite { action, path })
			}
			Err(error) => self.file_failure(action, path, error),
		}
	}

	pub(crate) fn submit_path(&mut self, action: Action, mut path: PathBuf) {
		if path.as_os_str().is_empty() {
			self.files.error = Some("Enter a file path".into());
			return;
		}
		if matches!(action, Action::Save | Action::SaveAs) && path.extension().is_none() {
			path.set_extension("tuxd.json");
		}
		if action == Action::Open
			&& !path
				.file_name()
				.is_some_and(|name| name.to_string_lossy().ends_with(".tuxd.json"))
		{
			let mut picker = crate::files::picker::Picker::new(
				Action::ImportEscaped,
				&path.to_string_lossy(),
				self.workspace.session.path.as_deref(),
			);
			picker.offer_import();
			self.file_dialog(Dialog::Path {
				picker: Box::new(picker),
			});
			return;
		}
		match action {
			Action::Open => match persistence::load(&path) {
				Ok(document) => self.replace_or_prompt(Pending::Replace {
					document,
					path: Some(path),
					imported: false,
				}),
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
