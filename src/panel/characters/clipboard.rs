use super::Characters;

impl Characters {
	pub fn paste(&mut self, service: &crate::libraries::clipboard::Service) {
		if self.paste.is_none() {
			self.paste = Some(service.read());
		}
	}

	pub fn poll_paste(&mut self) -> Option<Option<String>> {
		let completion = self.paste.as_mut()?.poll()?;
		self.paste = None;
		Some(match completion.result {
			Ok(text) => {
				self.insert(&text);
				None
			}
			Err(error) => Some(crate::features::clipboard_notice::paste_error(error)),
		})
	}
}
