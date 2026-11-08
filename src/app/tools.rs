use super::App;
use crate::workspace::Effect;
impl App {
	pub(super) fn tool_effect(&mut self, effect: Effect) {
		match effect {
			Effect::Copy(text) => self.copy_field(text),
			Effect::Paste(field) => self.paste_rgb(field),
			Effect::Notice(message) => self.notice = Some(message),
			Effect::None => (),
		}
	}
}
