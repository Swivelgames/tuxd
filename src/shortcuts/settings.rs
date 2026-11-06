//! Fixed settings controls never depend on the configurable action map.
use super::registry::{Subsection, category_actions};
use super::{
	ActionId, Config, InputScope, SettingsSection, ShortcutKey, default_slots, sequence_label,
};
use crate::scrollbar::Bar;
use crossterm::event::{
	Event, KeyCode, KeyEventKind, KeyModifiers as M, MouseButton, MouseEventKind,
};
use ratatui::layout::Rect;
use std::collections::{BTreeMap, BTreeSet};
#[path = "settings_view.rs"]
mod view;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Focus {
	Categories,
	List,
	Binding(usize),
	Reset,
	Unset,
	Timeout,
	Footer(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
	Section(Subsection),
	Action(ActionId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hit {
	Category(SettingsSection),
	Selector,
	Row(usize),
	Binding(usize),
	Reset,
	Unset,
	Timeout,
	Footer(usize),
	Discard,
	Keep,
}
struct Capture {
	action: ActionId,
	slot: usize,
	count: usize,
	keys: Vec<ShortcutKey>,
	received: Vec<String>,
}
pub struct Settings {
	pub draft: Config,
	original: Config,
	category: SettingsSection,
	collapsed: BTreeSet<(SettingsSection, Subsection)>,
	remembered: BTreeMap<SettingsSection, ActionId>,
	selected: Option<ActionId>,
	row: usize,
	slot: usize,
	focus: Focus,
	top: usize,
	nav_top: usize,
	viewport: usize,
	rows: Vec<Row>,
	positions: Vec<usize>,
	narrow: bool,
	selector_open: bool,
	capture: Option<Capture>,
	timeout: Option<String>,
	replace_timeout: bool,
	discard: Option<bool>,
	pending_g: bool,
	pub error: Option<String>,
	message: Option<String>,
	hits: Vec<(Rect, Hit)>,
	bar: Option<Bar>,
	nav_bar: Option<Bar>,
	drag: Option<(bool, usize)>,
	follow: bool,
	size: (u16, u16),
}
pub enum Outcome {
	Keep,
	Close,
	Save,
}
impl Settings {
	pub fn new(config: &Config, category: SettingsSection) -> Self {
		let mut state = Self {
			draft: config.clone(),
			original: config.clone(),
			category,
			collapsed: BTreeSet::new(),
			remembered: BTreeMap::new(),
			selected: None,
			row: 0,
			slot: 0,
			focus: Focus::List,
			top: 0,
			nav_top: 0,
			viewport: 1,
			rows: Vec::new(),
			positions: Vec::new(),
			narrow: false,
			selector_open: false,
			capture: None,
			timeout: None,
			replace_timeout: true,
			discard: None,
			pending_g: false,
			error: None,
			message: None,
			hits: Vec::new(),
			bar: None,
			nav_bar: None,
			drag: None,
			follow: true,
			size: (0, 0),
		};
		state.switch_category(category);
		state.focus = Focus::List;
		state
	}
	pub fn for_scope(config: &Config, scope: InputScope) -> Self {
		let mut state = Self::new(config, scope.section());
		let action = match scope {
			InputScope::Location | InputScope::Filename => Some(ActionId::SelectAll),
			InputScope::Radio | InputScope::ToolTabs | InputScope::Grid | InputScope::Format => {
				Some(ActionId::FieldLeft)
			}
			InputScope::Channel => Some(ActionId::RangeIncrease),
			InputScope::Tools => Some(ActionId::Activate),
			_ => None,
		};
		if let Some(action) = action {
			state.select_action(action);
		}
		state
	}
	fn rebuild_rows(&mut self) {
		self.rows.clear();
		let mut previous = None;
		for spec in category_actions(self.category) {
			if spec.subsection != previous {
				if let Some(sub) = spec.subsection {
					self.rows.push(Row::Section(sub));
				}
				previous = spec.subsection;
			}
			if spec
				.subsection
				.is_none_or(|sub| !self.collapsed.contains(&(self.category, sub)))
			{
				self.rows.push(Row::Action(spec.id));
			}
		}
		self.row = self.row.min(self.rows.len().saturating_sub(1));
	}
	fn switch_category(&mut self, category: SettingsSection) {
		self.category = category;
		self.top = 0;
		self.row = 0;
		self.selected = self
			.remembered
			.get(&category)
			.copied()
			.or_else(|| category_actions(category).first().map(|s| s.id));
		if let Some(action) = self.selected
			&& let Some(sub) = action.spec().subsection
		{
			self.collapsed.remove(&(category, sub));
		}
		self.rebuild_rows();
		if let Some(action) = self.selected {
			self.row = self
				.rows
				.iter()
				.position(|r| *r == Row::Action(action))
				.unwrap_or(0);
		}
		self.follow = true;
	}
	fn select_action(&mut self, action: ActionId) {
		self.remembered.insert(action.spec().section, action);
		self.switch_category(action.spec().section);
		self.focus = Focus::List;
	}
	fn select_row(&mut self, row: usize) {
		self.row = row.min(self.rows.len().saturating_sub(1));
		if let Some(Row::Action(action)) = self.rows.get(self.row).copied() {
			self.selected = Some(action);
			self.remembered.insert(self.category, action);
		}
		self.follow = true;
	}
	fn toggle_section(&mut self, sub: Subsection) {
		if !self.collapsed.remove(&(self.category, sub)) {
			self.collapsed.insert((self.category, sub));
		}
		self.rebuild_rows();
		self.row = self
			.rows
			.iter()
			.position(|r| *r == Row::Section(sub))
			.unwrap_or(0);
		self.follow = true;
	}
	fn close(&mut self) -> Outcome {
		if self.draft == self.original {
			Outcome::Close
		} else {
			self.discard = Some(false);
			Outcome::Keep
		}
	}
	fn validate(&mut self) {
		self.error = self.draft.build().err();
	}
	fn capture(&mut self, count: usize) {
		if let Some(action) = self.selected {
			self.capture = Some(Capture {
				action,
				slot: self.slot,
				count,
				keys: Vec::new(),
				received: Vec::new(),
			});
			self.focus = Focus::Binding(self.slot);
			self.error = None;
		}
	}
	fn reset(&mut self, unset: bool) {
		if let Some(action) = self.selected {
			self.draft.bindings.get_mut(&action).unwrap()[self.slot] = if unset {
				None
			} else {
				default_slots(action)[self.slot].clone()
			};
			self.validate();
		}
	}
	fn activate(&mut self) -> Outcome {
		match self.focus {
			Focus::Categories => {
				if self.narrow && !self.selector_open {
					self.selector_open = true;
				} else {
					self.selector_open = false;
					self.focus = if self.category == SettingsSection::Timeout {
						Focus::Timeout
					} else {
						Focus::List
					};
				}
			}
			Focus::List => match self.rows.get(self.row).copied() {
				Some(Row::Section(sub)) => self.toggle_section(sub),
				Some(Row::Action(_)) => self.focus = Focus::Binding(self.slot),
				_ => (),
			},
			Focus::Binding(slot) => {
				self.slot = slot;
				self.capture(1);
			}
			Focus::Reset => self.reset(false),
			Focus::Unset => self.reset(true),
			Focus::Timeout => {
				self.timeout = Some(self.draft.timeout_ms.to_string());
				self.replace_timeout = true;
			}
			Focus::Footer(0) => {
				self.draft = Config::default();
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
	fn cycle_focus(&mut self, back: bool) {
		let mut fields = vec![Focus::Categories];
		if self.category == SettingsSection::Timeout {
			fields.push(Focus::Timeout);
		} else {
			fields.push(Focus::List);
			if self.selected.is_some() {
				fields.extend([
					Focus::Binding(0),
					Focus::Binding(1),
					Focus::Binding(2),
					Focus::Reset,
					Focus::Unset,
				]);
			}
		}
		fields.extend([Focus::Footer(0), Focus::Footer(1), Focus::Footer(2)]);
		let at = fields.iter().position(|f| *f == self.focus).unwrap_or(0);
		self.focus = fields[(at + if back { fields.len() - 1 } else { 1 }) % fields.len()];
		if let Focus::Binding(slot) = self.focus {
			self.slot = slot;
		}
		self.selector_open = false;
	}
	fn navigate(&mut self, code: KeyCode) {
		let up = matches!(code, KeyCode::Up | KeyCode::Char('k') | KeyCode::PageUp);
		let down = matches!(code, KeyCode::Down | KeyCode::Char('j') | KeyCode::PageDown);
		let left = matches!(code, KeyCode::Left | KeyCode::Char('h'));
		let right = matches!(code, KeyCode::Right | KeyCode::Char('l'));
		match self.focus {
			Focus::Categories => {
				let at = SettingsSection::ALL
					.iter()
					.position(|s| *s == self.category)
					.unwrap();
				let next = match code {
					KeyCode::Home => 0,
					KeyCode::End | KeyCode::Char('G') => 6,
					KeyCode::Char('M') => 3,
					_ if up => at.saturating_sub(1),
					_ if down => (at + 1).min(6),
					_ => at,
				};
				self.switch_category(SettingsSection::ALL[next]);
				if right {
					self.selector_open = false;
					self.focus = if self.category == SettingsSection::Timeout {
						Focus::Timeout
					} else {
						Focus::List
					};
				}
			}
			Focus::List => {
				let last = self.rows.len().saturating_sub(1);
				let current = self.positions.get(self.row).copied().unwrap_or(0);
				let target = match code {
					KeyCode::Home => Some(0),
					KeyCode::End | KeyCode::Char('G') => Some(last),
					KeyCode::PageUp | KeyCode::PageDown | KeyCode::Char('M') => {
						let y = if code == KeyCode::Char('M') {
							self.top + self.viewport / 2
						} else if up {
							current.saturating_sub(self.viewport)
						} else {
							current + self.viewport
						};
						self.positions
							.iter()
							.enumerate()
							.min_by_key(|(_, p)| p.abs_diff(y))
							.map(|(i, _)| i)
					}
					_ if up => Some(self.row.saturating_sub(1)),
					_ if down => Some((self.row + 1).min(last)),
					_ => None,
				};
				if let Some(row) = target {
					self.select_row(row);
				} else if left {
					self.focus = Focus::Categories;
				} else if right {
					if let Some(Row::Section(sub)) = self.rows.get(self.row).copied() {
						self.collapsed.remove(&(self.category, sub));
						self.rebuild_rows();
					} else {
						self.focus = Focus::Binding(self.slot);
					}
				}
			}
			Focus::Binding(slot) => {
				if left {
					self.slot = slot.saturating_sub(1);
					self.focus = Focus::Binding(self.slot);
				}
				if right {
					self.slot = (slot + 1).min(2);
					self.focus = Focus::Binding(self.slot);
				}
				if up {
					self.focus = if self.narrow && slot > 0 {
						self.slot -= 1;
						Focus::Binding(self.slot)
					} else {
						Focus::List
					};
				}
				if down {
					self.focus = if self.narrow && slot < 2 {
						self.slot += 1;
						Focus::Binding(self.slot)
					} else {
						Focus::Reset
					};
				}
			}
			Focus::Reset | Focus::Unset => {
				if left {
					self.focus = Focus::Reset;
				}
				if right {
					self.focus = Focus::Unset;
				}
				if up {
					self.focus = Focus::Binding(self.slot);
				}
				if down {
					self.focus = Focus::Footer(1);
				}
			}
			Focus::Timeout => {
				if left || up {
					self.focus = Focus::Categories;
				}
				if down {
					self.focus = Focus::Footer(1);
				}
			}
			Focus::Footer(i) => {
				if left {
					self.focus = Focus::Footer(i.saturating_sub(1));
				}
				if right {
					self.focus = Focus::Footer((i + 1).min(2));
				}
				if up {
					self.focus = if self.category == SettingsSection::Timeout {
						Focus::Timeout
					} else {
						Focus::List
					};
				}
			}
		}
	}
	pub fn event(&mut self, event: Event) -> Outcome {
		if let Event::Key(key) = event {
			if key.kind == KeyEventKind::Release {
				return Outcome::Keep;
			}
			let decoded = ShortcutKey::from(key);
			if let Some(capture) = &mut self.capture {
				if key.kind != KeyEventKind::Press {
					return Outcome::Keep;
				}
				if decoded.code == KeyCode::Esc && decoded.modifiers.is_empty() {
					self.capture = None;
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
				capture.keys.push(decoded);
				if capture.keys.len() == capture.count {
					let capture = self.capture.take().unwrap();
					let label = sequence_label(&capture.keys);
					let received = capture.received.join(" ");
					self.message = Some(if label == received {
						format!("Recorded: {label}")
					} else {
						format!("Recorded: {label} (terminal reported {received})")
					});
					self.draft.bindings.get_mut(&capture.action).unwrap()[capture.slot] =
						Some(capture.keys);
					self.validate();
				}
				return Outcome::Keep;
			}
			if let Some(discard) = &mut self.discard {
				match decoded.code {
					KeyCode::Esc => self.discard = None,
					KeyCode::Enter if *discard => return Outcome::Close,
					KeyCode::Enter => self.discard = None,
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
			if let Some(text) = &mut self.timeout {
				match decoded.code {
					KeyCode::Esc => {
						self.timeout = None;
						self.validate();
					}
					KeyCode::Enter => match text.parse::<u64>() {
						Ok(ms) if (1..=60_000).contains(&ms) => {
							self.draft.timeout_ms = ms;
							self.timeout = None;
							self.validate();
						}
						_ => self.error = Some("Timeout must be 1–60000 milliseconds".into()),
					},
					KeyCode::Backspace => {
						if self.replace_timeout {
							text.clear();
						} else {
							text.pop();
						}
						self.replace_timeout = false;
					}
					KeyCode::Char('a') if decoded.modifiers == M::CONTROL => {
						self.replace_timeout = true
					}
					KeyCode::Char(ch) if ch.is_ascii_digit() && decoded.modifiers.is_empty() => {
						if self.replace_timeout {
							text.clear();
							self.replace_timeout = false;
						}
						if text.len() < 5 {
							text.push(ch);
						}
					}
					_ => (),
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
						self.switch_category(SettingsSection::Timeout);
						self.focus = Focus::Timeout;
						return self.activate();
					}
					KeyCode::Char('l') => {
						self.focus = Focus::Categories;
						self.selector_open = self.narrow;
					}
					_ => (),
				}
				return Outcome::Keep;
			}
			match decoded.code {
				KeyCode::Esc if self.selector_open => {
					self.selector_open = false;
					self.focus = Focus::Categories;
				}
				KeyCode::Esc => return self.close(),
				KeyCode::Enter => return self.activate(),
				KeyCode::Tab => self.cycle_focus(decoded.modifiers.contains(M::SHIFT)),
				KeyCode::Char(ch @ '1'..='9')
					if matches!(self.focus, Focus::Binding(_))
						|| (self.focus == Focus::List
							&& matches!(self.rows.get(self.row), Some(Row::Action(_)))) =>
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
			self.pending_g = false;
			if self.capture.is_some() || self.timeout.is_some() {
				return Outcome::Keep;
			}
			let point = (mouse.column, mouse.row).into();
			match mouse.kind {
				MouseEventKind::Down(MouseButton::Left) => {
					if let Some((_, hit)) = self
						.hits
						.iter()
						.rev()
						.find(|(r, _)| r.contains(point))
						.copied()
					{
						if self.discard.is_some() {
							match hit {
								Hit::Discard => return Outcome::Close,
								Hit::Keep => self.discard = None,
								_ => (),
							}
							return Outcome::Keep;
						}
						match hit {
							Hit::Category(cat) => {
								self.switch_category(cat);
								self.selector_open = false;
								self.focus = if cat == SettingsSection::Timeout {
									Focus::Timeout
								} else {
									Focus::List
								};
							}
							Hit::Selector => {
								self.focus = Focus::Categories;
								self.selector_open = !self.selector_open;
							}
							Hit::Row(i) => {
								self.select_row(i);
								self.focus = Focus::List;
								if let Some(Row::Section(sub)) = self.rows.get(i).copied() {
									self.toggle_section(sub);
								}
							}
							Hit::Binding(slot) => {
								self.slot = slot;
								self.focus = Focus::Binding(slot);
								self.capture(1);
							}
							Hit::Reset => {
								self.focus = Focus::Reset;
								self.reset(false);
							}
							Hit::Unset => {
								self.focus = Focus::Unset;
								self.reset(true);
							}
							Hit::Timeout => {
								self.focus = Focus::Timeout;
								return self.activate();
							}
							Hit::Footer(i) => {
								self.focus = Focus::Footer(i);
								return self.activate();
							}
							_ => (),
						}
						return Outcome::Keep;
					}
					if self.discard.is_none() {
						for (nav, bar) in [(false, self.bar), (true, self.nav_bar)] {
							if let Some(bar) = bar.filter(|b| b.area.contains(point)) {
								let at = bar.coordinate(mouse.column, mouse.row);
								let (start, size) = bar.thumb();
								let grab = if (start..start + size).contains(&at) {
									at - start
								} else {
									size / 2
								};
								if nav {
									self.nav_top = bar.offset_at(at, grab);
								} else {
									self.top = bar.offset_at(at, grab);
								}
								self.drag = Some((nav, grab));
								self.follow = false;
							}
						}
					}
				}
				MouseEventKind::Drag(MouseButton::Left) => {
					if let Some((nav, grab)) = self.drag
						&& let Some(bar) = if nav { self.nav_bar } else { self.bar }
					{
						if nav {
							self.nav_top =
								bar.offset_at(bar.coordinate(mouse.column, mouse.row), grab);
						} else {
							self.top = bar.offset_at(bar.coordinate(mouse.column, mouse.row), grab);
						}
						self.follow = false;
					}
				}
				MouseEventKind::Up(MouseButton::Left) => self.drag = None,
				MouseEventKind::ScrollUp | MouseEventKind::ScrollDown if self.discard.is_none() => {
					let nav = self.hits.iter().any(|(r, h)| {
						r.contains(point) && matches!(h, Hit::Category(_) | Hit::Selector)
					});
					let offset = if nav {
						&mut self.nav_top
					} else {
						&mut self.top
					};
					*offset = if mouse.kind == MouseEventKind::ScrollUp {
						offset.saturating_sub(3)
					} else {
						offset.saturating_add(3)
					};
					self.follow = false;
				}
				_ => (),
			}
		}
		Outcome::Keep
	}
}
impl crate::app::App {
	pub fn settings_event(&mut self, event: Event) {
		let Some(settings) = &mut self.settings else {
			return;
		};
		match settings.event(event) {
			Outcome::Close => {
				self.settings = None;
				self.shortcuts.cancel();
			}
			Outcome::Save => match self.shortcuts.save(settings.draft.clone()) {
				Ok(()) => {
					self.settings = None;
					self.notice = Some("Shortcuts saved".into());
				}
				Err(error) => settings.error = Some(format!("Could not save shortcuts: {error}")),
			},
			Outcome::Keep => (),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::super::{ActionId, Dispatch, InputScope};
	use super::*;
	use crossterm::event::KeyEvent;
	use ratatui::{Terminal, backend::TestBackend};
	use std::time::Instant;
	fn event(text: &str) -> Event {
		let key = super::super::key::defaults(text).remove(0);
		Event::Key(KeyEvent::new(key.code, key.modifiers))
	}
	fn select(settings: &mut Settings, action: ActionId, slot: usize) {
		settings.select_action(action);
		settings.slot = slot;
		settings.focus = Focus::Binding(slot);
	}
	#[test]
	fn capture_is_counted_ignores_repeat_release_and_can_cancel() {
		let mut settings = Settings::new(&Config::default(), SettingsSection::Editor);
		select(&mut settings, ActionId::FocusTools, 2);
		settings.event(event("2"));
		let Event::Key(mut repeat) = event("Ctrl+w") else {
			unreachable!()
		};
		repeat.kind = KeyEventKind::Repeat;
		settings.event(Event::Key(repeat));
		repeat.kind = KeyEventKind::Release;
		settings.event(Event::Key(repeat));
		assert!(settings.capture.as_ref().unwrap().keys.is_empty());
		settings.event(event("Ctrl+w"));
		assert_eq!(settings.capture.as_ref().unwrap().keys.len(), 1);
		settings.event(event("Right"));
		assert!(settings.capture.is_none());
		assert_eq!(
			settings.draft.bindings[&ActionId::FocusTools][2],
			Some(super::super::key::defaults("Ctrl+w Right"))
		);
		assert!(settings.error.is_none());
		settings.event(event("Enter"));
		settings.event(event("Esc"));
		assert!(settings.capture.is_none());
		assert!(settings.draft.bindings[&ActionId::FocusTools][2].is_some());
	}
	#[test]
	fn conflicts_keep_the_draft_editable_and_reset_unset_and_discard_are_fixed() {
		let mut settings = Settings::new(&Config::default(), SettingsSection::Editor);
		select(&mut settings, ActionId::FocusTools, 0);
		settings.event(event("Enter"));
		settings.event(event("F1"));
		assert!(settings.error.as_ref().unwrap().contains("reserved"));
		assert!(matches!(settings.event(event("Ctrl+s")), Outcome::Keep));
		settings.focus = Focus::Binding(0);
		settings.event(event("Delete"));
		assert!(settings.error.is_none());
		select(&mut settings, ActionId::Bold, 0);
		settings.event(event("Delete"));
		assert!(settings.draft.bindings[&ActionId::Bold][0].is_none());
		settings.focus = Focus::Reset;
		settings.event(event("Enter"));
		assert_eq!(
			settings.draft.bindings[&ActionId::Bold],
			default_slots(ActionId::Bold)
		);
		settings.focus = Focus::Binding(0);
		settings.event(event("Delete"));
		settings.event(event("Esc"));
		assert_eq!(settings.discard, Some(false));
		settings.event(event("Enter"));
		assert!(settings.discard.is_none());
		settings.event(event("Esc"));
		settings.event(event("Left"));
		assert!(matches!(settings.event(event("Enter")), Outcome::Close));
	}
	#[test]
	fn save_activates_and_persists_only_after_explicit_save_and_restart() {
		let folder = tempfile::tempdir().unwrap();
		let path = folder.path().join("shortcuts.json");
		let mut app = crate::app::App::default();
		app.shortcuts.path = Ok(path.clone());
		app.handle(event("F1"));
		let settings = app.settings.as_mut().unwrap();
		select(settings, ActionId::ToggleCode, 0);
		app.handle(event("Enter"));
		app.handle(event("F8"));
		assert_eq!(app.shortcuts.config.hint(ActionId::ToggleCode), "F2");
		assert!(!path.exists());
		app.handle(event("Ctrl+s"));
		assert!(app.settings.is_none());
		assert_eq!(app.shortcuts.config.hint(ActionId::ToggleCode), "F8");
		app.handle(event("F2"));
		assert!(app.code_open);
		app.handle(event("F8"));
		assert!(!app.code_open);
		let mut restarted = crate::app::App::default();
		restarted.shortcuts.path = Ok(path);
		restarted.shortcuts.load().unwrap();
		restarted.handle(event("F8"));
		assert!(!restarted.code_open);
	}
	#[test]
	fn save_failure_retains_draft_and_capture_reports_actual_terminal_key() {
		let mut app = crate::app::App::default();
		app.shortcuts.path = Err("test unavailable config directory".into());
		app.handle(event("F1"));
		let settings = app.settings.as_mut().unwrap();
		select(settings, ActionId::FocusTools, 0);
		app.handle(event("Enter"));
		app.handle(event("Ctrl+7"));
		assert!(app.settings.as_ref().unwrap().error.is_some());
		app.handle(event("Enter"));
		app.handle(event("F8"));
		app.handle(event("Ctrl+s"));
		let settings = app.settings.as_ref().unwrap();
		assert!(
			settings
				.error
				.as_ref()
				.unwrap()
				.contains("test unavailable")
		);
		assert_eq!(settings.draft.hint(ActionId::FocusTools), "F8");
		assert!(app.shortcuts.config.hint(ActionId::FocusTools).is_empty());
	}
	#[test]
	fn timeout_edit_reset_all_and_fixed_navigation_are_independent_of_bindings() {
		let mut config = Config::default();
		for slots in config.bindings.values_mut() {
			*slots = [None, None, None];
		}
		let mut settings = Settings::new(&config, SettingsSection::Editor);
		settings.event(event("Ctrl+t"));
		for ch in ["2", "5", "0"] {
			settings.event(event(ch));
		}
		settings.event(event("Enter"));
		assert_eq!(settings.draft.timeout_ms, 250);
		settings.focus = Focus::Categories;
		settings.event(event("g"));
		settings.event(event("g"));
		assert_eq!(settings.category, SettingsSection::Timeout);
		settings.event(event("j"));
		assert_eq!(settings.category, SettingsSection::Workspace);
		settings.event(event("G"));
		assert_eq!(settings.category, SettingsSection::Explorer);
		settings.event(event("Ctrl+r"));
		assert_eq!(settings.draft, Config::default());
		assert!(matches!(settings.event(event("Ctrl+s")), Outcome::Save));
	}
	#[test]
	fn resize_scroll_mouse_and_context_jump_keep_controls_reachable() {
		let mut settings = Settings::new(&Config::default(), SettingsSection::Explorer);
		for (width, height) in [
			(140, 40),
			(80, 24),
			(64, 20),
			(40, 12),
			(20, 5),
			(1, 1),
			(140, 40),
		] {
			let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
			terminal.draw(|frame| settings.draw(frame)).unwrap();
			if width >= 40 && height >= 20 {
				assert!(settings.hits.iter().any(|(_, hit)| *hit == Hit::Footer(2)));
				assert!(!settings.hits.is_empty());
				let rect = settings
					.hits
					.iter()
					.find(|(_, c)| matches!(c, Hit::Binding(_)))
					.unwrap()
					.0;
				settings.event(Event::Mouse(crossterm::event::MouseEvent {
					kind: MouseEventKind::Down(MouseButton::Left),
					column: rect.x,
					row: rect.y,
					modifiers: M::NONE,
				}));
				assert!(settings.capture.is_some());
				settings.event(event("Esc"));
			}
		}
		let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
		settings.event(event("G"));
		terminal.draw(|frame| settings.draw(frame)).unwrap();
		assert!(settings.hits.iter().any(|(_, c)| *c == Hit::Footer(2)));
		settings.focus = Focus::Footer(2);
		assert!(matches!(settings.event(event("Enter")), Outcome::Close));
	}
	#[test]
	fn draft_sequence_uses_library_after_save_and_does_not_change_dialog_scope() {
		let folder = tempfile::tempdir().unwrap();
		let mut app = crate::app::App::default();
		app.shortcuts.path = Ok(folder.path().join("shortcuts.json"));
		app.handle(event("F1"));
		select(app.settings.as_mut().unwrap(), ActionId::FocusCode, 0);
		app.handle(event("2"));
		app.handle(event("Ctrl+w"));
		app.handle(event("Right"));
		app.handle(event("Ctrl+s"));
		app.handle(event("Ctrl+w"));
		assert!(app.shortcuts.pending_label().is_some());
		app.handle(event("Right"));
		assert_eq!(app.focus, 2);
		assert!(matches!(
			app.shortcuts.dispatch(
				super::super::key::defaults("Ctrl+w").remove(0),
				InputScope::Prompt,
				Instant::now()
			),
			Dispatch::Unbound
		));
	}
	#[test]
	fn categories_subsections_and_context_jump_keep_movement_and_selection_separate() {
		assert_eq!(
			SettingsSection::ALL.map(SettingsSection::label),
			[
				"Sequence Timeout",
				"Workspace",
				"Editor",
				"Code",
				"Tools",
				"Dialogs",
				"File Explorer"
			]
		);
		for category in SettingsSection::ALL {
			let settings = Settings::new(&Config::default(), category);
			assert!(settings.collapsed.is_empty());
			for action in category_actions(category) {
				assert!(settings.rows.contains(&Row::Action(action.id)));
			}
		}
		let mut settings = Settings::for_scope(&Config::default(), InputScope::Grid);
		assert_eq!(settings.category, SettingsSection::Workspace);
		assert_eq!(settings.selected, Some(ActionId::FieldLeft));
		assert!(
			!settings
				.collapsed
				.contains(&(SettingsSection::Workspace, Subsection::FieldNavigation))
		);
		settings.select_action(ActionId::SelectLeft);
		assert_eq!(settings.category, SettingsSection::Editor);
		assert!(settings.rows.contains(&Row::Action(ActionId::SelectLeft)));
		assert!(settings.rows.contains(&Row::Action(ActionId::MoveLeft)));
		settings.toggle_section(Subsection::Selection);
		settings.event(event("2"));
		assert!(
			settings.capture.is_none(),
			"A collapsed heading must not edit a hidden action"
		);
		settings.select_action(ActionId::ToggleCode);
		assert_eq!(settings.category, SettingsSection::Code);
	}
	#[test]
	fn category_selector_and_footer_remain_usable_at_small_sizes() {
		let mut settings = Settings::new(&Config::default(), SettingsSection::Editor);
		for (width, height) in [
			(120, 32),
			(80, 24),
			(64, 24),
			(40, 16),
			(40, 12),
			(24, 8),
			(20, 5),
			(1, 1),
		] {
			let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
			terminal.draw(|frame| settings.draw(frame)).unwrap();
			for (rect, _) in &settings.hits {
				assert!(
					rect.right() <= width && rect.bottom() <= height,
					"{rect:?} outside {width}x{height}"
				);
			}
			if width >= 24 && height >= 8 {
				assert!(settings.hits.iter().any(|(_, h)| *h == Hit::Footer(1)));
				assert!(settings.hits.iter().any(|(_, h)| *h == Hit::Footer(2)));
				settings.event(event("Ctrl+l"));
				settings.event(event("g"));
				settings.event(event("g"));
				terminal.draw(|frame| settings.draw(frame)).unwrap();
				assert_eq!(settings.category, SettingsSection::Timeout);
				settings.event(event("Enter"));
				assert_eq!(settings.focus, Focus::Timeout);
				settings.event(event("Enter"));
				settings.event(event("Esc"));
				assert!(settings.timeout.is_none());
				settings.switch_category(SettingsSection::Editor);
				settings.focus = Focus::List;
			}
		}
	}
}
