use super::App;
use crate::theme;

impl App {
	pub fn load_configuration(&mut self) {
		if let Err(error) = self.shortcuts.load() {
			self.notice = Some(format!(
				"Shortcuts: {}; using defaults",
				crate::libraries::text::display::sanitize(&error)
			));
		}
		self.theme = match self
			.theme_path
			.as_ref()
			.map_err(Clone::clone)
			.and_then(|path| theme::Config::read(path))
		{
			Ok(theme) => theme,
			Err(error) => {
				let notice = format!(
					"Theme: {}; using Midnight",
					crate::libraries::text::display::sanitize(&error)
				);
				self.notice = Some(
					self.notice
						.take()
						.map_or(notice.clone(), |old| format!("{old}; {notice}")),
				);
				theme::Config::default()
			}
		};
	}
}
