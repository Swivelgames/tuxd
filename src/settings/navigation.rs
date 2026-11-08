use crate::settings::interaction::Mode;
use crate::settings::sections::Section;
use crate::settings::{Focus, Row, Settings, ThemeRow};
use crate::shortcuts::ActionId;
use crate::shortcuts::registry::{Subsection, category_actions};
use crossterm::event::KeyCode;

impl Settings {
	pub(super) fn categories(&self) -> Vec<Section> {
		Section::ALL
			.into_iter()
			.filter(|category| match category {
				Section::ThemeGroup | Section::ShortcutsGroup => true,
				Section::Appearance => !self.closed_groups.contains(&Section::ThemeGroup),
				_ => !self.closed_groups.contains(&Section::ShortcutsGroup),
			})
			.collect()
	}

	pub(super) fn toggle_group(&mut self) {
		if !self.closed_groups.remove(&self.category) {
			self.closed_groups.insert(self.category);
		}
	}

	pub(super) fn rebuild_rows(&mut self) {
		self.shortcuts.rows.clear();
		let mut previous = None;
		for spec in self
			.category
			.shortcut_category()
			.map(category_actions)
			.unwrap_or_default()
		{
			if spec.subsection != previous {
				if let Some(sub) = spec.subsection {
					self.shortcuts.rows.push(Row::Section(sub));
				}
				previous = spec.subsection;
			}
			if spec
				.subsection
				.is_none_or(|sub| !self.shortcuts.collapsed.contains(&(self.category, sub)))
			{
				self.shortcuts.rows.push(Row::Action(spec.id));
			}
		}
		self.shortcuts.row = self
			.shortcuts
			.row
			.min(self.shortcuts.rows.len().saturating_sub(1));
	}

	pub(super) fn switch_category(&mut self, category: Section) {
		if category == Section::Appearance {
			self.closed_groups.remove(&Section::ThemeGroup);
		} else if !matches!(category, Section::ThemeGroup | Section::ShortcutsGroup) {
			self.closed_groups.remove(&Section::ShortcutsGroup);
		}
		self.category = category;
		self.top = 0;
		self.shortcuts.row = 0;
		self.shortcuts.selected = self
			.shortcuts
			.remembered
			.get(&category)
			.copied()
			.or_else(|| {
				category
					.shortcut_category()
					.map(category_actions)
					.unwrap_or_default()
					.first()
					.map(|s| s.id)
			});
		if let Some(action) = self.shortcuts.selected
			&& let Some(sub) = action.spec().subsection
		{
			self.shortcuts.collapsed.remove(&(category, sub));
		}
		self.rebuild_rows();
		if let Some(action) = self.shortcuts.selected {
			self.shortcuts.row = self
				.shortcuts
				.rows
				.iter()
				.position(|r| *r == Row::Action(action))
				.unwrap_or(0);
		}
		self.follow = true;
	}

	pub(super) fn select_action(&mut self, action: ActionId) {
		self.shortcuts
			.remembered
			.insert(action.spec().section.into(), action);
		self.switch_category(action.spec().section.into());
		self.focus = Focus::List;
	}

	pub(super) fn select_row(&mut self, row: usize) {
		self.shortcuts.row = row.min(self.shortcuts.rows.len().saturating_sub(1));
		if let Some(Row::Action(action)) = self.shortcuts.rows.get(self.shortcuts.row).copied() {
			self.shortcuts.selected = Some(action);
			self.shortcuts.remembered.insert(self.category, action);
		}
		self.follow = true;
	}

	pub(super) fn toggle_section(&mut self, sub: Subsection) {
		if !self.shortcuts.collapsed.remove(&(self.category, sub)) {
			self.shortcuts.collapsed.insert((self.category, sub));
		}
		self.rebuild_rows();
		self.shortcuts.row = self
			.shortcuts
			.rows
			.iter()
			.position(|r| *r == Row::Section(sub))
			.unwrap_or(0);
		self.follow = true;
	}

	pub(super) fn cycle_focus(&mut self, back: bool) {
		let mut fields = vec![Focus::Categories];
		if self.category == Section::Appearance {
			fields.push(Focus::Preset);
			fields.push(Focus::ResetTheme);
			fields.push(Focus::List);
			if matches!(
				self.theme_rows().get(self.appearance.cursor),
				Some(ThemeRow::Color(_))
			) {
				fields.push(Focus::Reset);
			}
		} else if self.category == Section::Timeout {
			fields.push(Focus::Leader);
			fields.push(Focus::Timeout);
		} else {
			fields.push(Focus::List);
			if self.shortcuts.selected.is_some() {
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
		self.focus = fields[crate::libraries::controls::cycle(at, fields.len(), back)];
		if let Focus::Binding(slot) = self.focus {
			self.shortcuts.slot = slot;
		}
		self.mode = Mode::Browse;
	}

	pub(super) fn navigate(&mut self, code: KeyCode) {
		let up = matches!(code, KeyCode::Up | KeyCode::Char('k') | KeyCode::PageUp);
		let down = matches!(code, KeyCode::Down | KeyCode::Char('j') | KeyCode::PageDown);
		let left = matches!(code, KeyCode::Left | KeyCode::Char('h'));
		let right = matches!(code, KeyCode::Right | KeyCode::Char('l'));
		match self.focus {
			Focus::Preset => {
				if left || right {
					self.cycle_preset(left);
				}
				if up {
					self.focus = Focus::Categories;
				}
				if down {
					self.focus = Focus::List;
				}
			}
			Focus::ResetTheme => {
				if up || left {
					self.focus = Focus::Preset;
				}
				if down {
					self.focus = Focus::List;
				}
			}
			Focus::Categories => {
				let categories = self.categories();
				let at = categories.iter().position(|s| *s == self.category).unwrap();
				let next = match code {
					KeyCode::Home => 0,
					KeyCode::End | KeyCode::Char('G') => categories.len() - 1,
					KeyCode::Char('M') => categories.len() / 2,
					_ if up => at.saturating_sub(1),
					_ if down => (at + 1).min(categories.len() - 1),
					_ => at,
				};
				self.switch_category(categories[next]);
				if matches!(self.category, Section::ThemeGroup | Section::ShortcutsGroup) {
					if left {
						self.closed_groups.insert(self.category);
					}
					if right {
						self.closed_groups.remove(&self.category);
					}
					return;
				}
				if right {
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
			Focus::List if self.category == Section::Appearance => {
				if up && self.appearance.cursor == 0 {
					self.focus = Focus::Preset;
					return;
				}
				let last = self.theme_rows().len().saturating_sub(1);
				let next = match code {
					KeyCode::Home => 0,
					KeyCode::End | KeyCode::Char('G') => last,
					KeyCode::PageUp => self
						.appearance
						.cursor
						.saturating_sub(self.viewport.max(1) / 2),
					KeyCode::PageDown => {
						(self.appearance.cursor + self.viewport.max(1) / 2).min(last)
					}
					KeyCode::Char('M') => last / 2,
					_ if up => self.appearance.cursor.saturating_sub(1),
					_ if down => (self.appearance.cursor + 1).min(last),
					_ => self.appearance.cursor,
				};
				self.select_theme_row(next);
				if left || right {
					if let Some(ThemeRow::Section(section)) =
						self.theme_rows().get(self.appearance.cursor)
					{
						if right {
							self.appearance.collapsed.remove(section);
						} else {
							self.appearance.collapsed.insert(*section);
						}
					} else if right {
						self.focus = Focus::Reset;
					} else {
						self.focus = Focus::Categories;
					}
				}
			}
			Focus::Reset if self.category == Section::Appearance => {
				if left || up {
					self.focus = Focus::List;
				}
				if down {
					self.focus = Focus::List;
					self.select_theme_row(self.appearance.cursor + 1);
				}
			}
			Focus::List => {
				let last = self.shortcuts.rows.len().saturating_sub(1);
				let current = self
					.shortcuts
					.positions
					.get(self.shortcuts.row)
					.copied()
					.unwrap_or(0);
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
						self.shortcuts
							.positions
							.iter()
							.enumerate()
							.min_by_key(|(_, p)| p.abs_diff(y))
							.map(|(i, _)| i)
					}
					_ if up => Some(self.shortcuts.row.saturating_sub(1)),
					_ if down => Some((self.shortcuts.row + 1).min(last)),
					_ => None,
				};
				if let Some(row) = target {
					self.select_row(row);
				} else if left {
					self.focus = Focus::Categories;
				} else if right {
					if let Some(Row::Section(sub)) =
						self.shortcuts.rows.get(self.shortcuts.row).copied()
					{
						self.shortcuts.collapsed.remove(&(self.category, sub));
						self.rebuild_rows();
					} else {
						self.focus = Focus::Binding(self.shortcuts.slot);
					}
				}
			}
			Focus::Binding(slot) => {
				if left {
					self.shortcuts.slot = slot.saturating_sub(1);
					self.focus = Focus::Binding(self.shortcuts.slot);
				}
				if right {
					self.shortcuts.slot = (slot + 1).min(2);
					self.focus = Focus::Binding(self.shortcuts.slot);
				}
				if up {
					self.focus = if self.narrow && slot > 0 {
						self.shortcuts.slot -= 1;
						Focus::Binding(self.shortcuts.slot)
					} else {
						Focus::List
					};
				}
				if down {
					self.focus = if self.narrow && slot < 2 {
						self.shortcuts.slot += 1;
						Focus::Binding(self.shortcuts.slot)
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
					self.focus = Focus::Binding(self.shortcuts.slot);
				}
				if down {
					self.focus = Focus::Footer(1);
				}
			}
			Focus::Leader => {
				if left || up {
					self.focus = Focus::Categories;
				}
				if down {
					self.focus = Focus::Timeout;
				}
			}
			Focus::Timeout => {
				if left {
					self.focus = Focus::Categories;
				}
				if up {
					self.focus = Focus::Leader;
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
					self.focus = if self.category == Section::Appearance {
						Focus::Preset
					} else if self.category == Section::Timeout {
						Focus::Leader
					} else {
						Focus::List
					};
				}
			}
		}
	}
}
