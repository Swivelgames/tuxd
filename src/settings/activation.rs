use crate::settings::interaction::Mode;
use crate::settings::sections::Section;
use crate::settings::{Focus, Outcome, Row, Settings, ThemeRow};
use crate::shortcuts::Config;

impl Settings {
	pub(super) fn activate(&mut self) -> Outcome {
		if matches!(self.category, Section::ThemeGroup | Section::ShortcutsGroup)
			&& matches!(self.focus, Focus::Categories | Focus::List)
		{
			self.toggle_group();
			return Outcome::Keep;
		}
		if self.category == Section::Appearance && self.focus == Focus::Reset {
			self.reset_theme_role();
			return Outcome::Keep;
		}
		if self.category == Section::Appearance && self.focus == Focus::List {
			if let Some(ThemeRow::Section(section)) = self.theme_rows().get(self.appearance.cursor)
			{
				let section = *section;
				if !self.appearance.collapsed.remove(&section) {
					self.appearance.collapsed.insert(section);
				}
				return Outcome::Keep;
			}
			let role = self.appearance.role.id();
			self.mode = Mode::Color(crate::libraries::controls::Field::selecting(
				self.appearance
					.draft
					.colors
					.get(role)
					.cloned()
					.unwrap_or_default(),
			));
			return Outcome::Keep;
		}
		match self.focus {
			Focus::Preset => {
				self.appearance.preset_cursor = crate::theme::PRESETS
					.iter()
					.position(|(id, _)| *id == self.appearance.draft.preset)
					.unwrap_or(0);
				self.appearance.preset_top = 0;
				self.mode = Mode::Preset;
			}
			Focus::ResetTheme => {
				self.appearance.draft.colors.clear();
				self.validate();
				self.message = Some("Theme customizations reset in draft".into());
			}
			Focus::Categories => {
				if self.narrow && !matches!(self.mode, Mode::Selector) {
					self.mode = Mode::Selector;
				} else {
					self.mode = Mode::Browse;
					self.focus = if self.category == Section::Appearance {
						Focus::Preset
					} else if self.category == Section::Timeout {
						Focus::Leader
					} else {
						Focus::List
					};
				}
			}
			Focus::List => match self.shortcuts.rows.get(self.shortcuts.row).copied() {
				Some(Row::Section(sub)) => self.toggle_section(sub),
				Some(Row::Action(_)) => self.focus = Focus::Binding(self.shortcuts.slot),
				_ => (),
			},
			Focus::Binding(slot) => {
				self.shortcuts.slot = slot;
				self.capture(1);
			}
			Focus::Reset => self.reset(false),
			Focus::Unset => self.reset(true),
			Focus::Leader => {
				self.mode = Mode::LeaderCapture;
			}
			Focus::Timeout => {
				self.mode = Mode::Timeout(crate::libraries::controls::Field::selecting(
					self.shortcuts.draft.timeout_ms.to_string(),
				));
			}
			Focus::Footer(0) => {
				self.shortcuts.draft = Config::default();
				self.appearance.draft = crate::theme::Config::default();
				self.validate();
				self.message = Some("Defaults restored in draft".into());
			}
			Focus::Footer(1) => {
				self.validate();
				if self.error.is_none() {
					return Outcome::Save;
				}
			}
			Focus::Footer(_) => return self.close(),
		}
		Outcome::Keep
	}
}
