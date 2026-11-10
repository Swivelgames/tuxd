//! Poll feature-owned clipboard completions and publish notices.
use super::App;
#[cfg(test)]
use crate::workspace::Pane;
use std::io::Write;

impl App {
	pub fn poll_clipboard(&mut self) -> bool {
		self.poll_clipboard_with(&mut std::io::stdout().lock())
	}

	fn poll_clipboard_with(&mut self, output: &mut impl Write) -> bool {
		let path_updated = self.files.poll();
		let mut code_copied = false;
		for buffer in self.workspace.buffers_mut() {
			if let Some(notice) = buffer.code.poll_copy(output) {
				self.notice = Some(notice);
				code_copied = true;
			}
		}
		let editor_copied = self.poll_editor_copy(output);
		let field_copied = self.poll_field_copy(output);
		let editor_pasted = self.poll_editor_paste();
		let rgb_pasted = self.poll_rgb_paste();
		let characters_pasted = match self.workspace.panel.characters.poll_paste() {
			Some(notice) => {
				if notice.is_some() {
					self.notice = notice;
				}
				true
			}
			None => false,
		};
		path_updated
			|| code_copied
			|| editor_copied
			|| field_copied
			|| editor_pasted
			|| rgb_pasted
			|| characters_pasted
	}

	pub(crate) fn copy_field(&mut self, text: String) {
		if let Some(notice) = self.field_copy.begin(&self.clipboard, text) {
			self.notice = Some(notice);
		}
	}

	fn poll_field_copy(&mut self, output: &mut impl Write) -> bool {
		if let Some(notice) = self.field_copy.poll(output) {
			self.notice = Some(notice);
			true
		} else {
			false
		}
	}

	pub(super) fn paste_rgb(&mut self, field: usize) {
		self.workspace.panel.color.paste(&self.clipboard, field);
	}

	fn poll_rgb_paste(&mut self) -> bool {
		let Some(notice) = self.workspace.panel.color.poll_paste() else {
			return false;
		};
		if let Some(notice) = notice {
			self.notice = Some(notice);
		}
		true
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		libraries::clipboard::{Receipt, WriteOutput},
		libraries::editing::Movement,
		test_clipboard::{controlled, until},
	};

	#[test]
	fn code_completion_keeps_its_owner_after_focus_changes_and_blocks_overlapping_cut() {
		let (service, control) = controlled();
		let mut app = App {
			clipboard: service,
			..Default::default()
		};
		app.paste_editor_text(
			"keep",
			crate::features::editor::clipboard::PasteMode::Styled,
		);
		app.workspace.active.editor.navigate(
			&mut app.workspace.active.session.editing,
			Movement::Start,
			true,
		);
		let before = app.workspace.active.session.editing.document.clone();
		app.workspace.focus = Pane::Code;
		app.execute_action(crate::shortcuts::ActionId::Copy);
		app.workspace.focus = Pane::Editor;
		app.copy_selection(true);
		assert!(!app.workspace.active.clipboard.copying());
		control
			.writes
			.send(Ok(WriteOutput::Delivered(Receipt::Tmux)))
			.unwrap();
		until(|| app.poll_clipboard_with(&mut Vec::new()));
		assert_eq!(app.workspace.active.session.editing.document, before);
		assert_eq!(
			app.notice.as_deref(),
			Some("Sent escaped ANSI via tmux; clipboard delivery unconfirmed")
		);
	}

	#[test]
	fn replacement_session_invalidates_cut_and_pending_paste_even_with_identical_content() {
		let (service, control) = controlled();
		let mut app = App {
			clipboard: service,
			..Default::default()
		};
		app.paste_editor_text(
			"keep",
			crate::features::editor::clipboard::PasteMode::Styled,
		);
		app.workspace.active.editor.navigate(
			&mut app.workspace.active.session.editing,
			Movement::Start,
			true,
		);
		app.copy_selection(true);
		app.request_editor_paste(crate::features::editor::clipboard::PasteMode::Styled);
		let document = app.workspace.active.session.editing.document.clone();
		app.load_document(document.clone(), None, false);
		app.workspace.active.editor.navigate(
			&mut app.workspace.active.session.editing,
			Movement::Finish,
			true,
		);
		control
			.writes
			.send(Ok(WriteOutput::Delivered(Receipt::Desktop)))
			.unwrap();
		control.reads.send(Ok("stale".into())).unwrap();
		until(|| app.poll_clipboard_with(&mut Vec::new()));
		assert_eq!(app.workspace.active.session.editing.document, document);
		assert!(!app.workspace.active.clipboard.pasting());
	}

	#[test]
	fn character_search_paste_stays_in_its_field_after_focus_moves() {
		let (service, control) = controlled();
		let mut app = App {
			clipboard: service,
			..Default::default()
		};
		app.workspace.panel.characters.paste(&app.clipboard);
		app.workspace.focus = Pane::Editor;
		control.reads.send(Ok("U+2500".into())).unwrap();
		until(|| app.poll_clipboard_with(&mut Vec::new()));
		assert_eq!(app.workspace.panel.characters.results, vec!['─']);
		assert!(app.workspace.active.session.editing.document.is_empty());
	}

	#[test]
	fn field_copy_and_rgb_read_retain_payload_and_original_field() {
		let (service, control) = controlled();
		let mut app = App {
			clipboard: service,
			..Default::default()
		};
		app.copy_field("#123456".into());
		assert_eq!(
			control
				.payloads
				.recv_timeout(std::time::Duration::from_secs(1))
				.unwrap(),
			"#123456"
		);
		app.paste_rgb(1);
		app.workspace.panel.selected = 0;
		app.workspace.focus = Pane::Code;
		control.reads.send(Ok(" 128 ".into())).unwrap();
		until(|| app.poll_rgb_paste());
		assert_eq!(app.workspace.panel.color.rgb.fields[1].text(), "128");
		control
			.writes
			.send(Ok(WriteOutput::Delivered(Receipt::Desktop)))
			.unwrap();
		until(|| app.poll_clipboard_with(&mut Vec::new()));
		assert_eq!(app.notice.as_deref(), Some("Copied selection to clipboard"));
	}
}
