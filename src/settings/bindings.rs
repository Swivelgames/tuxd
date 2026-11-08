use crate::settings::interaction::Mode;
use crate::settings::sections::Section;
use crate::settings::{Capture, Focus, Outcome, Row, Settings};
use crate::shortcuts::registry::Subsection;
use crate::shortcuts::{ActionId, Config, ShortcutKey, default_slots, sequence_label};
use crossterm::event::{KeyCode, KeyEventKind};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Shortcuts {
	pub(super) draft: Config,
	pub(super) original: Config,
	pub(super) collapsed: BTreeSet<(Section, Subsection)>,
	pub(super) remembered: BTreeMap<Section, ActionId>,
	pub(super) selected: Option<ActionId>,
	pub(super) row: usize,
	pub(super) slot: usize,
	pub(super) rows: Vec<Row>,
	pub(super) positions: Vec<usize>,
}

impl Settings {
	pub(super) fn capture(&mut self, count: usize) {
		if let Some(action) = self.shortcuts.selected {
			self.mode = Mode::Capture(Capture {
				action,
				slot: self.shortcuts.slot,
				count,
				keys: Vec::new(),
				received: Vec::new(),
			});
			self.focus = Focus::Binding(self.shortcuts.slot);
			self.error = None;
		}
	}

	pub(super) fn reset(&mut self, unset: bool) {
		if let Some(action) = self.shortcuts.selected {
			self.shortcuts.draft.bindings.get_mut(&action).unwrap()[self.shortcuts.slot] = if unset
			{
				None
			} else {
				default_slots(action)[self.shortcuts.slot].clone()
			};
			self.validate();
		}
	}

	pub(super) fn capture_key(&mut self, key: crossterm::event::KeyEvent) -> Outcome {
		let decoded = ShortcutKey::from(key);
		if let Mode::Capture(capture) = &mut self.mode {
			if decoded.is_leader() {
				return Outcome::Keep;
			}
			if key.kind != KeyEventKind::Press {
				return Outcome::Keep;
			}
			if decoded.code == KeyCode::Esc && decoded.modifiers.is_empty() {
				self.mode = Mode::Browse;
				self.validate();
				return Outcome::Keep;
			}
			capture.received.push(
				ShortcutKey {
					code: key.code,
					modifiers: key.modifiers,
				}
				.label(),
			);
			if capture.keys.is_empty() && self.shortcuts.draft.leader.as_ref() == Some(&decoded) {
				capture.keys.push(ShortcutKey::leader());
				capture.count += 1;
			} else {
				capture.keys.push(decoded);
			}
			if capture.keys.len() == capture.count {
				let Mode::Capture(capture) = std::mem::take(&mut self.mode) else {
					unreachable!()
				};
				let label = sequence_label(&capture.keys);
				let received = capture.received.join(" ");
				self.message = Some(if label == received {
					format!("Recorded: {label}")
				} else {
					format!("Recorded: {label} (terminal reported {received})")
				});
				self.shortcuts
					.draft
					.bindings
					.get_mut(&capture.action)
					.unwrap()[capture.slot] = Some(capture.keys);
				self.validate();
			}
			return Outcome::Keep;
		}
		Outcome::Keep
	}
}

impl Shortcuts {
	pub(super) fn new(config: &Config) -> Self {
		Self {
			draft: config.clone(),
			original: config.clone(),
			collapsed: BTreeSet::new(),
			remembered: BTreeMap::new(),
			selected: None,
			row: 0,
			slot: 0,
			rows: Vec::new(),
			positions: Vec::new(),
		}
	}
}
