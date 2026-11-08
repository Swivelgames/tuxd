use crate::libraries::controls::FieldAction;
use crate::settings::interaction::Mode;
use crate::settings::{Outcome, Settings};
use crate::shortcuts::ShortcutKey;
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers as M};

pub(super) struct General {
	pub(super) leader_scroll: u16,
	pub(super) leader_choice: usize,
}

impl Settings {
	pub(super) fn request_leader(&mut self, leader: Option<ShortcutKey>) {
		if leader == self.shortcuts.draft.leader {
			return;
		}
		let affected = self
			.shortcuts
			.draft
			.bindings
			.values()
			.flatten()
			.flatten()
			.any(|keys| {
				keys.first().is_some_and(ShortcutKey::is_leader)
					|| (leader.is_some() && keys.first() == leader.as_ref())
			});
		if affected {
			self.general.leader_scroll = 0;
			self.general.leader_choice = if leader.is_some() { 1 } else { 2 };
			self.mode = Mode::LeaderChange(leader);
		} else {
			self.shortcuts.draft.change_leader(leader, false);
			self.validate();
		}
	}

	pub(super) fn choose_leader(&mut self, choice: usize) {
		if let Mode::LeaderChange(change) = std::mem::take(&mut self.mode) {
			let cancel = if change.is_some() { 1 } else { 2 };
			if choice < cancel {
				self.shortcuts.draft.change_leader(change, choice == 1);
				self.validate();
			}
		}
	}

	pub(super) fn leader_choice_key(&mut self, key: crossterm::event::KeyEvent) -> Outcome {
		let decoded = ShortcutKey::from(key);
		if let Mode::LeaderChange(change) = self.mode.clone() {
			let count = if change.is_some() { 2 } else { 3 };
			match decoded.code {
				KeyCode::Left | KeyCode::BackTab | KeyCode::Char('h') => {
					self.general.leader_choice =
						crate::libraries::controls::cycle(self.general.leader_choice, count, true)
				}
				KeyCode::Right | KeyCode::Tab | KeyCode::Char('l') => {
					self.general.leader_choice =
						crate::libraries::controls::cycle(self.general.leader_choice, count, false)
				}
				KeyCode::Enter => self.choose_leader(self.general.leader_choice),
				KeyCode::Down | KeyCode::Char('j') => {
					self.general.leader_scroll = self.general.leader_scroll.saturating_add(1)
				}
				KeyCode::Up | KeyCode::Char('k') => {
					self.general.leader_scroll = self.general.leader_scroll.saturating_sub(1)
				}
				KeyCode::Esc | KeyCode::Char('c') => self.mode = Mode::Browse,
				KeyCode::Char('y') if change.is_some() => {
					self.shortcuts.draft.change_leader(change, false);
					self.mode = Mode::Browse;
					self.validate();
				}
				KeyCode::Char('p' | 'r') if change.is_none() => {
					self.shortcuts
						.draft
						.change_leader(None, decoded.code == KeyCode::Char('r'));
					self.mode = Mode::Browse;
					self.validate();
				}
				_ => (),
			}
			return Outcome::Keep;
		}
		Outcome::Keep
	}

	pub(super) fn leader_key(&mut self, key: crossterm::event::KeyEvent) -> Outcome {
		let decoded = ShortcutKey::from(key);
		if matches!(self.mode, Mode::LeaderCapture) {
			if key.kind != KeyEventKind::Press {
				return Outcome::Keep;
			}
			self.mode = Mode::Browse;
			if decoded.code != KeyCode::Esc {
				if decoded.opener() || decoded.is_leader() {
					self.error = Some("This key is reserved".into());
				} else {
					self.request_leader(Some(decoded));
				}
			}
			return Outcome::Keep;
		}
		Outcome::Keep
	}

	pub(super) fn timeout_key(&mut self, key: crossterm::event::KeyEvent) -> Outcome {
		let decoded = ShortcutKey::from(key);
		if let Mode::Timeout(text) = &mut self.mode {
			match decoded.code {
				KeyCode::Esc => {
					self.mode = Mode::Browse;
					self.validate();
				}
				KeyCode::Enter => match text.text().parse::<u64>() {
					Ok(ms) if (1..=60_000).contains(&ms) => {
						self.shortcuts.draft.timeout_ms = ms;
						self.mode = Mode::Browse;
						self.validate();
					}
					_ => self.error = Some("Timeout must be 1–60000 milliseconds".into()),
				},
				KeyCode::Backspace => text.apply(FieldAction::Backspace),
				KeyCode::Char('a') if decoded.modifiers == M::CONTROL => text.select_all(),
				KeyCode::Char(ch) if ch.is_ascii_digit() && decoded.modifiers.is_empty() => {
					text.insert(&ch.to_string(), 5);
				}
				_ => (),
			}
			return Outcome::Keep;
		}
		Outcome::Keep
	}
}

impl General {
	pub(super) fn new() -> Self {
		Self {
			leader_scroll: 0,
			leader_choice: 0,
		}
	}
}
