use super::App;
impl App {
	pub fn draw(&mut self, frame: &mut ratatui::Frame) {
		let active = self
			.settings
			.as_ref()
			.map(|s| s.preview())
			.unwrap_or_else(|| self.theme.resolve().unwrap_or_default());
		crate::theme::with(&active, || {
			let status = self
				.shortcuts
				.pending_label()
				.or_else(|| self.notice.clone());
			let focused = self.workspace_available();
			self.workspace.draw(
				frame,
				crate::workspace::Chrome {
					config: &self.shortcuts.config,
					status: status.as_deref(),
					focused,
				},
			);
			crate::files::draw(frame, &mut self.files, &self.shortcuts.config);
			if let Some(review) = &mut self.import_review {
				review.draw(frame);
			}
			if let Some(settings) = &mut self.settings {
				settings.draw(frame);
			}
		});
	}
}
