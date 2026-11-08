use super::App;
use crate::settings::Outcome;
use crossterm::event::Event;

impl App {
	pub fn settings_event(&mut self, event: Event) {
		let Some(settings) = &mut self.settings else {
			return;
		};
		match settings.event(event) {
			Outcome::Close => {
				self.settings = None;
				self.shortcuts.cancel();
			}
			Outcome::Save => match crate::configuration::commit::save(
				settings.drafts(),
				|config| self.shortcuts.save(config.clone()),
				&self.theme_path,
			) {
				Ok(()) => {
					self.theme = settings.drafts().theme.clone();
					self.settings = None;
					self.notice = Some("Settings saved".into());
				}
				Err(error) => settings.error = Some(format!("Could not save settings: {error}")),
			},
			Outcome::Keep => (),
		}
	}
}
