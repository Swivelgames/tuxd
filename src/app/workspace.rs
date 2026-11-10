use super::App;
use crate::workspace::Effect;
impl App {
	pub(super) fn workspace_effect(&mut self, effect: Effect) {
		match effect {
			Effect::OpenFile(path) => {
				self.submit_path(crate::files::Action::Open, path);
			}
			Effect::Copy(text) => self.copy_field(text),
			Effect::Paste(field) => self.paste_rgb(field),
			Effect::PasteCharacters => self.workspace.panel.characters.paste(&self.clipboard),
			Effect::Notice(message) => self.notice = Some(message),
			Effect::CloseBuffer(id) => {
				if self.workspace.select_session(id) {
					self.file_action(crate::files::Action::CloseBuffer);
				}
			}
			Effect::None => (),
		}
	}
}
