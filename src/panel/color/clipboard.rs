//! A pending paste belongs to the original RGB field even when focus moves.
use super::Color;
use crate::{features::clipboard_notice, libraries::clipboard::Service};

impl Color {
	pub fn paste(&mut self, service: &Service, field: usize) {
		if self.paste.is_none() {
			self.paste = Some((field, service.read()));
		}
	}
	/// None means pending/idle; an empty notice means the draft was updated.
	pub fn poll_paste(&mut self) -> Option<Option<String>> {
		let (field, request) = self.paste.as_mut()?;
		let completion = request.poll()?;
		let field = *field;
		self.paste = None;
		Some(match completion.result {
			Ok(text) => {
				self.rgb.edit(field, text.trim(), false);
				None
			}
			Err(error) => Some(clipboard_notice::paste_error(error)),
		})
	}
}
