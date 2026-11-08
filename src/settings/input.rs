use crate::settings::interaction::Mode;
use crate::settings::sections::Section;
use crate::settings::{Focus, Hit, Outcome, Row, Settings, ThemeRow};
use crate::shortcuts::ShortcutKey;
use crossterm::event::{
	Event, KeyCode, KeyEventKind, KeyModifiers as M, MouseButton, MouseEventKind,
};

impl Settings {
	pub fn event(&mut self, event: Event) -> Outcome {
		if matches!(self.mode, Mode::Preset) {
			return self.preset_event(event);
		}

		if let Event::Paste(text) = &event
			&& let Mode::Color(input) = &mut self.mode
		{
			if text.len() <= 32 && text.chars().all(|ch| !ch.is_control()) {
				input.set(text.clone());
			}
			return Outcome::Keep;
		}
		if !matches!(event, Event::Key(_)) && matches!(self.mode, Mode::Color(_)) {
			return Outcome::Keep;
		}

		if matches!(self.mode, Mode::LeaderChange(_))
			&& let Event::Mouse(mouse) = event
		{
			if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
				let hit = self
					.hits
					.iter()
					.find(|(area, _)| area.contains((mouse.column, mouse.row).into()))
					.map(|(_, hit)| *hit);
				if let Some(Hit::LeaderChoice(choice)) = hit {
					self.choose_leader(choice);
				}
			}
			return Outcome::Keep;
		}
		if !matches!(event, Event::Key(_))
			&& (matches!(self.mode, Mode::LeaderCapture)
				|| matches!(self.mode, Mode::LeaderChange(_)))
		{
			return Outcome::Keep;
		}
		if let Event::Key(key) = event {
			if key.kind == KeyEventKind::Release {
				return Outcome::Keep;
			}
			match self.mode {
				Mode::Color(_) => return self.color_key(key),
				Mode::LeaderChange(_) => return self.leader_choice_key(key),
				Mode::LeaderCapture => return self.leader_key(key),
				Mode::Capture(_) => return self.capture_key(key),
				Mode::Timeout(_) => return self.timeout_key(key),
				Mode::Discard(_) => return self.discard_key(key),
				Mode::Preset => unreachable!("preset events are handled above"),
				Mode::Browse | Mode::Selector => (),
			}
			let decoded = ShortcutKey::from(key);
			if self.category == Section::Appearance
				&& self.focus == Focus::List
				&& decoded.code == KeyCode::Delete
				&& matches!(
					self.theme_rows().get(self.appearance.cursor),
					Some(ThemeRow::Color(_))
				) {
				self.appearance
					.draft
					.colors
					.remove(self.appearance.role.id());
				self.validate();
				return Outcome::Keep;
			}

			if self.focus == Focus::Leader && decoded.code == KeyCode::Delete {
				if self.shortcuts.draft.leader.is_some() {
					self.request_leader(None);
				}
				return Outcome::Keep;
			}

			self.message = None;
			let second_g = self.pending_g && decoded.printable() == Some('g');
			self.pending_g = false;
			if decoded.modifiers == M::CONTROL {
				match decoded.code {
					KeyCode::Char('s') => {
						self.focus = Focus::Footer(1);
						return self.activate();
					}
					KeyCode::Char('r') => {
						self.focus = Focus::Footer(0);
						return self.activate();
					}
					KeyCode::Char('t') => {
						self.switch_category(Section::Timeout);
						self.focus = Focus::Timeout;
						return self.activate();
					}
					KeyCode::Char('l') => {
						self.focus = Focus::Categories;
						self.mode = if self.narrow {
							Mode::Selector
						} else {
							Mode::Browse
						};
					}
					_ => (),
				}
				return Outcome::Keep;
			}
			match decoded.code {
				KeyCode::Esc if matches!(self.mode, Mode::Selector) => {
					self.mode = Mode::Browse;
					self.focus = Focus::Categories;
				}
				KeyCode::Esc => return self.close(),
				KeyCode::Enter => return self.activate(),
				KeyCode::Tab => self.cycle_focus(decoded.modifiers.contains(M::SHIFT)),
				KeyCode::Char(ch @ '1'..='9')
					if matches!(self.focus, Focus::Binding(_))
						|| (self.focus == Focus::List
							&& matches!(
								self.shortcuts.rows.get(self.shortcuts.row),
								Some(Row::Action(_))
							)) =>
				{
					self.capture(ch.to_digit(10).unwrap() as usize)
				}
				KeyCode::Delete | KeyCode::Backspace
					if matches!(self.focus, Focus::Binding(_) | Focus::Unset) =>
				{
					self.reset(true)
				}
				KeyCode::Char('g') if second_g => self.navigate(KeyCode::Home),
				KeyCode::Char('g') => self.pending_g = true,
				code => self.navigate(code),
			}
		} else if let Event::Mouse(mouse) = event {
			return self.mouse_event(mouse);
		}
		Outcome::Keep
	}

	fn discard_key(&mut self, key: crossterm::event::KeyEvent) -> Outcome {
		let decoded = ShortcutKey::from(key);
		if let Mode::Discard(discard) = &mut self.mode {
			match decoded.code {
				KeyCode::Esc => self.mode = Mode::Browse,
				KeyCode::Enter if *discard => return Outcome::Close,
				KeyCode::Enter => self.mode = Mode::Browse,
				KeyCode::Left
				| KeyCode::Right
				| KeyCode::Up
				| KeyCode::Down
				| KeyCode::Tab
				| KeyCode::Char('h' | 'j' | 'k' | 'l') => *discard = !*discard,
				_ => (),
			}
			return Outcome::Keep;
		}
		Outcome::Keep
	}
}
