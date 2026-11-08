use crate::libraries::controls::FieldAction;
use crate::settings::interaction::Mode;
use crate::settings::{Hit, Outcome, Settings, ThemeRow};
use crate::shortcuts::ShortcutKey;
use crossterm::event::{
	Event, KeyCode, KeyEventKind, KeyModifiers as M, MouseButton, MouseEventKind,
};
use std::collections::BTreeSet;

pub(super) struct Appearance {
	pub(super) preset_cursor: usize,
	pub(super) preset_top: usize,
	pub(super) cursor: usize,
	pub(super) collapsed: BTreeSet<crate::theme::Group>,
	pub(super) draft: crate::theme::Config,
	pub(super) original: crate::theme::Config,
	pub(super) role: crate::theme::Role,
}

impl Settings {
	pub(super) fn cycle_preset(&mut self, back: bool) {
		let presets = crate::theme::PRESETS;
		let index = presets
			.iter()
			.position(|(id, _)| *id == self.appearance.draft.preset)
			.unwrap_or(0);
		self.appearance.draft.preset = presets
			[crate::libraries::controls::cycle(index, presets.len(), back)]
		.0
		.into();
		self.validate();
	}

	pub(super) fn select_theme_row(&mut self, index: usize) {
		self.appearance.cursor = index.min(self.theme_rows().len().saturating_sub(1));
		if let Some(ThemeRow::Color(role)) = self.theme_rows().get(self.appearance.cursor) {
			self.appearance.role = *role;
		}
		self.follow = true;
	}

	pub(super) fn reset_theme_role(&mut self) {
		self.appearance
			.draft
			.colors
			.remove(self.appearance.role.id());
		self.validate();
	}

	pub(super) fn color_key(&mut self, key: crossterm::event::KeyEvent) -> Outcome {
		let decoded = ShortcutKey::from(key);
		if let Mode::Color(input) = &mut self.mode {
			match decoded.code {
				KeyCode::Esc => {
					self.mode = Mode::Browse;
					self.validate();
				}
				KeyCode::Enter => {
					let role = self.appearance.role.id();
					let mut draft = self.appearance.draft.clone();
					if input.text().is_empty() {
						draft.colors.remove(role);
					} else {
						draft.colors.insert(role.into(), input.text().to_owned());
					}
					match draft.resolve() {
						Ok(_) => {
							self.appearance.draft = draft;
							self.mode = Mode::Browse;
							self.validate();
						}
						Err(error) => self.error = Some(error),
					}
				}
				KeyCode::Backspace => input.apply(FieldAction::Backspace),
				KeyCode::Char('a') if decoded.modifiers == M::CONTROL => input.select_all(),
				KeyCode::Char(ch) if decoded.modifiers.is_empty() && input.text().len() < 32 => {
					input.insert(&ch.to_string(), usize::MAX);
				}
				_ => (),
			}
			return Outcome::Keep;
		}
		Outcome::Keep
	}

	pub(super) fn preset_event(&mut self, event: Event) -> Outcome {
		if matches!(self.mode, Mode::Preset) {
			let last = crate::theme::PRESETS.len() - 1;
			match event {
				Event::Key(key) if key.kind != KeyEventKind::Release => match key.code {
					KeyCode::Esc => self.mode = Mode::Browse,
					KeyCode::Enter => {
						self.appearance.draft.preset = crate::theme::PRESETS
							[self.appearance.preset_cursor]
							.0
							.into();
						self.mode = Mode::Browse;
						self.validate();
					}
					KeyCode::Up | KeyCode::Char('k') => {
						self.appearance.preset_cursor =
							self.appearance.preset_cursor.saturating_sub(1)
					}
					KeyCode::Down | KeyCode::Char('j') => {
						self.appearance.preset_cursor =
							(self.appearance.preset_cursor + 1).min(last)
					}
					KeyCode::Home | KeyCode::Char('g') => self.appearance.preset_cursor = 0,
					KeyCode::End | KeyCode::Char('G') => self.appearance.preset_cursor = last,
					KeyCode::PageUp => {
						self.appearance.preset_cursor =
							self.appearance.preset_cursor.saturating_sub(5)
					}
					KeyCode::PageDown => {
						self.appearance.preset_cursor =
							(self.appearance.preset_cursor + 5).min(last)
					}
					_ => (),
				},
				Event::Mouse(mouse) => match mouse.kind {
					MouseEventKind::ScrollUp => {
						self.appearance.preset_cursor =
							self.appearance.preset_cursor.saturating_sub(1)
					}
					MouseEventKind::ScrollDown => {
						self.appearance.preset_cursor =
							(self.appearance.preset_cursor + 1).min(last)
					}
					MouseEventKind::Down(MouseButton::Left) => {
						if let Some((_, Hit::Preset(index))) = self
							.hits
							.iter()
							.find(|(rect, _)| rect.contains((mouse.column, mouse.row).into()))
						{
							self.appearance.draft.preset = crate::theme::PRESETS[*index].0.into();
							self.mode = Mode::Browse;
							self.validate();
						}
					}
					_ => (),
				},
				_ => (),
			}
			return Outcome::Keep;
		}
		Outcome::Keep
	}
}

impl Appearance {
	pub(super) fn new() -> Self {
		Self {
			preset_cursor: 0,
			preset_top: 0,
			cursor: 1,
			collapsed: BTreeSet::new(),
			draft: crate::theme::Config::default(),
			original: crate::theme::Config::default(),
			role: crate::theme::Role::Accent,
		}
	}

	pub(super) fn rows(&self) -> Vec<ThemeRow> {
		let mut rows = Vec::new();
		for &group in crate::theme::Group::ALL {
			rows.push(ThemeRow::Section(group));
			if !self.collapsed.contains(&group) {
				rows.extend(
					crate::theme::Role::ALL
						.iter()
						.copied()
						.filter(|role| role.group() == group)
						.map(ThemeRow::Color),
				);
			}
		}
		rows
	}
	pub(super) fn preview(&self, mode: &Mode) -> crate::theme::Theme {
		let mut draft = self.draft.clone();
		if matches!(*mode, Mode::Preset) {
			draft.preset = crate::theme::PRESETS[self.preset_cursor].0.into();
		}
		if let Mode::Color(input) = mode {
			let role = self.role.id();
			if input.text().is_empty() {
				draft.colors.remove(role);
			} else {
				draft.colors.insert(role.into(), input.text().to_owned());
			}
		}
		draft
			.resolve()
			.unwrap_or_else(|_| self.draft.resolve().unwrap_or_default())
	}
	pub(super) fn set_theme(&mut self, config: &crate::theme::Config) {
		self.draft = config.clone();
		self.original = config.clone();
	}
}

impl Settings {
	pub(super) fn theme_rows(&self) -> Vec<ThemeRow> {
		self.appearance.rows()
	}
	pub fn preview(&self) -> crate::theme::Theme {
		self.appearance.preview(&self.mode)
	}
	pub fn set_theme(&mut self, config: &crate::theme::Config) {
		self.appearance.set_theme(config);
	}
}
