mod actions;
#[cfg(test)]
mod buffer_tests;
mod clipboard;
mod editor_clipboard;
#[cfg(test)]
mod editor_clipboard_tests;
mod files;
mod focus;
mod review;
mod settings;
mod startup;
mod view;
#[cfg(test)]
mod view_tests;
mod workspace;

#[cfg(test)]
use crate::panel::Control;
use crate::{libraries::editing::EditKind, workspace::Pane};
use crossterm::event::{Event, KeyEventKind};
use std::time::Instant;

pub struct App {
	pending_file: Option<files::Pending>,
	exiting: bool,
	pub workspace: crate::workspace::Workspace,
	pub theme: crate::theme::Config,
	pub theme_path: Result<std::path::PathBuf, String>,
	pub clipboard: crate::libraries::clipboard::Service,
	pub import_review: Option<crate::workflows::PendingReview>,
	pub files: crate::files::Files,
	pub notice: Option<String>,
	field_copy: crate::components::field_clipboard::Copy,

	pub shortcuts: crate::shortcuts::Runtime,
	pub settings: Option<crate::settings::Settings>,
}

impl Default for App {
	fn default() -> Self {
		Self {
			pending_file: None,
			exiting: false,
			workspace: Default::default(),
			theme: crate::theme::Config::default(),
			theme_path: crate::configuration::directory().map(|dir| dir.join("theme.json")),
			clipboard: Default::default(),
			import_review: None,
			files: Default::default(),
			notice: Default::default(),
			field_copy: Default::default(),
			shortcuts: Default::default(),
			settings: None,
		}
	}
}

impl App {
	pub(crate) fn workspace_available(&self) -> bool {
		self.files.dialog.is_none() && self.import_review.is_none() && self.settings.is_none()
	}

	/// Returns true only when the user has chosen to exit.
	pub fn handle(&mut self, event: Event) -> bool {
		use crate::shortcuts::{Dispatch, ShortcutKey};
		if self.settings.is_some() {
			self.settings_event(event);
			return false;
		}
		let scope = self.input_scope();
		if let Event::Key(key) = event {
			if key.kind == KeyEventKind::Release {
				return false;
			}
			let decoded = ShortcutKey::from(key);
			if decoded.opener() {
				self.shortcuts.cancel();
				self.workspace.active.session.editing.break_group();
				self.settings = Some(crate::settings::Settings::for_scope(
					&self.shortcuts.config,
					scope,
				));
				self.settings.as_mut().unwrap().set_theme(&self.theme);
				return false;
			}
			self.notice = None;
			match self
				.shortcuts
				.dispatch(decoded.clone(), scope, Instant::now())
			{
				Dispatch::Action(action) => self.execute_action(action),
				Dispatch::Unbound => {
					if let Some(ch) = decoded.printable() {
						if self.import_review.is_some() {
							// Review consumes text without editing the underlying session.
						} else if self.files.dialog.is_some() {
							self.file_event(&Event::Paste(ch.to_string()));
						} else if self.workspace.focus == Pane::Editor {
							self.workspace.active.editor.insert(
								&mut self.workspace.active.session.editing,
								&ch.to_string(),
								EditKind::Typing,
								Instant::now(),
							);
						} else if self.workspace.focus == Pane::Tools {
							self.workspace.panel.insert_text(&ch.to_string());
						}
					} else {
						self.workspace.active.session.editing.break_group();
					}
				}
				Dispatch::Pending | Dispatch::Consumed => (),
			}
		} else {
			// A mouse/paste/resize/focus event ends any pending keyboard sequence.
			self.shortcuts.cancel();
			if let Some(review) = &mut self.import_review {
				if let Some(decision) = review.view.event(&event) {
					self.decide_review(decision);
				}
			} else if self.file_event(&event).is_none() {
				match event {
					Event::Paste(text) if self.workspace.focus == Pane::Tools => {
						self.workspace.panel.insert_text(text.trim())
					}
					Event::Paste(text) if self.workspace.focus == Pane::Editor => self
						.paste_editor_text(
							&text,
							crate::features::editor::clipboard::PasteMode::Styled,
						),
					Event::Mouse(mouse) => {
						let effect = self.workspace.mouse(mouse);
						self.workspace_effect(effect);
					}
					Event::Resize(_, _) | Event::FocusLost => {
						self.workspace.active.session.editing.break_group()
					}
					_ => (),
				}
			}
		}
		if self.input_scope() != scope {
			self.shortcuts.cancel();
		}
		self.exiting
	}
}

#[cfg(test)]
mod tests;
