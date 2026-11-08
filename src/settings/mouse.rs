use crate::settings::interaction::Mode;
use crate::settings::sections::Section;
use crate::settings::{Focus, Hit, Outcome, Row, Settings, ThemeRow};
use crossterm::event::{MouseButton, MouseEventKind};

impl Settings {
	pub(super) fn mouse_event(&mut self, mouse: crossterm::event::MouseEvent) -> Outcome {
		self.pending_g = false;
		if matches!(self.mode, Mode::Capture(_)) || matches!(self.mode, Mode::Timeout(_)) {
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
					if matches!(self.mode, Mode::Discard(_)) {
						match hit {
							Hit::Discard => return Outcome::Close,
							Hit::Keep => self.mode = Mode::Browse,
							_ => (),
						}
						return Outcome::Keep;
					}
					match hit {
						Hit::Category(cat) => {
							self.switch_category(cat);
							if matches!(cat, Section::ThemeGroup | Section::ShortcutsGroup) {
								self.toggle_group();
								self.focus = Focus::Categories;
								return Outcome::Keep;
							}
							self.mode = Mode::Browse;
							self.focus = if cat == Section::Appearance {
								Focus::Preset
							} else if cat == Section::Timeout {
								Focus::Leader
							} else {
								Focus::List
							};
						}
						Hit::Selector => {
							self.focus = Focus::Categories;
							self.mode = if matches!(self.mode, Mode::Selector) {
								Mode::Browse
							} else {
								Mode::Selector
							};
						}
						Hit::PresetOpen => {
							self.focus = Focus::Preset;
							return self.activate();
						}
						Hit::Preset(index) => {
							self.appearance.draft.preset = crate::theme::PRESETS[index].0.into();
							self.focus = Focus::Preset;
							self.validate();
						}
						Hit::ResetTheme => {
							self.focus = Focus::ResetTheme;
							return self.activate();
						}
						Hit::ThemeReset(role) => {
							if let Some(index) = self
								.theme_rows()
								.iter()
								.position(|row| *row == ThemeRow::Color(role))
							{
								self.select_theme_row(index);
							}
							self.focus = Focus::Reset;
							self.reset_theme_role();
						}
						Hit::ThemeRow(i) => {
							self.select_theme_row(i);
							self.focus = Focus::List;
							return self.activate();
						}
						Hit::Row(i) => {
							self.select_row(i);
							self.focus = Focus::List;
							if let Some(Row::Section(sub)) = self.shortcuts.rows.get(i).copied() {
								self.toggle_section(sub);
							}
						}
						Hit::Binding(slot) => {
							self.shortcuts.slot = slot;
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
						Hit::Leader => {
							self.focus = Focus::Leader;
							return self.activate();
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
				if !matches!(self.mode, Mode::Discard(_)) {
					for (nav, bar) in [(false, self.bar), (true, self.nav_bar)] {
						if let Some(bar) = bar.filter(|b| b.area.contains(point)) {
							let at = bar.coordinate(mouse.column, mouse.row);
							let grab = bar.grab_at(at);
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
						self.nav_top = bar.offset_at(bar.coordinate(mouse.column, mouse.row), grab);
					} else {
						self.top = bar.offset_at(bar.coordinate(mouse.column, mouse.row), grab);
					}
					self.follow = false;
				}
			}
			MouseEventKind::Up(MouseButton::Left) => self.drag = None,
			MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
				if !matches!(self.mode, Mode::Discard(_)) =>
			{
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
		Outcome::Keep
	}
}
