use super::*;
use crate::scrollbar::{self, Target};
use crate::theme;
use ratatui::{
	Frame,
	style::{Color, Modifier, Style},
	text::{Line, Span},
	widgets::{Block, BorderType, Clear, Paragraph},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::theme::{accent, muted};
fn block(title: &str) -> Block<'_> {
	theme::block()
		.border_type(BorderType::Rounded)
		.border_style(Style::default().fg(theme::border_color()))
		.title(title)
}
fn wrap(text: &str, width: u16) -> Vec<String> {
	let width = width.max(1) as usize;
	let mut lines = Vec::new();
	for paragraph in text.split('\n') {
		let mut line = String::new();
		for word in paragraph.split_whitespace() {
			if !line.is_empty() && line.width() + 1 + word.width() > width {
				lines.push(std::mem::take(&mut line));
			}
			if !line.is_empty() {
				line.push(' ');
			}
			for ch in word.graphemes(true) {
				if !line.is_empty() && line.width() + ch.width() > width {
					lines.push(std::mem::take(&mut line));
				}
				line.push_str(ch);
			}
		}
		lines.push(line);
	}
	lines
}

fn text(frame: &mut Frame, area: Rect, value: impl Into<String>, style: Style) {
	frame.render_widget(Paragraph::new(value.into()).style(style), area);
}
fn line(area: Rect, offset: u16) -> Rect {
	Rect::new(
		area.x,
		area.y + offset.min(area.height),
		area.width,
		u16::from(offset < area.height),
	)
}
impl Settings {
	fn button(&mut self, frame: &mut Frame, area: Rect, label: &str, hit: Hit, active: bool) {
		if area.is_empty() {
			return;
		}
		let style = if active {
			accent().bg(theme::selected_color())
		} else {
			Style::default().bg(theme::field_color())
		};
		text(frame, area, label, style);
		self.hits.push((area, hit));
	}
	pub fn draw(&mut self, frame: &mut Frame) {
		let screen = frame.area();
		let width = screen.width.min(140);
		let area = Rect::new(
			screen.x + (screen.width - width) / 2,
			screen.y,
			width,
			screen.height,
		);
		if self.size != (width, area.height) {
			self.follow = true;
			self.size = (width, area.height);
		}
		self.hits.clear();
		self.bar = None;
		self.nav_bar = None;
		frame.render_widget(Clear, area);
		let outer = block(if !self.dirty() {
			" Settings "
		} else {
			" Settings · unsaved "
		});
		let inner = outer.inner(area);
		frame.render_widget(outer, area);
		if inner.is_empty() {
			return;
		}
		self.narrow = inner.width < 88;
		let footer = line(inner, inner.height.saturating_sub(1));
		let cancel_w = 8.min(footer.width);
		let save_w = 6.min(footer.width.saturating_sub(cancel_w));
		self.button(
			frame,
			Rect::new(
				footer.x,
				footer.y,
				11.min(footer.width.saturating_sub(save_w + cancel_w)),
				1,
			),
			" Reset all ",
			Hit::Footer(0),
			self.focus == Focus::Footer(0),
		);
		self.button(
			frame,
			Rect::new(footer.right() - cancel_w - save_w, footer.y, save_w, 1),
			" Save ",
			Hit::Footer(1),
			self.focus == Focus::Footer(1),
		);
		self.button(
			frame,
			Rect::new(footer.right() - cancel_w, footer.y, cancel_w, 1),
			" Cancel ",
			Hit::Footer(2),
			self.focus == Focus::Footer(2),
		);
		let status = if self.focus == Focus::Preset {
			"Enter chooses preset · ←/→ cycles · Overrides are preserved".into()
		} else if self.focus == Focus::ResetTheme {
			"Clear theme overrides; keep this preset and all shortcuts".into()
		} else if self.theme_input.is_some() && self.error.is_none() {
			"Color: #RRGGBB / ansi:0–255 / default · Enter accepts · Esc cancels".into()
		} else if self.leader_capture {
			"Recording Leader · Esc cancels".into()
		} else if let Some(capture) = &self.capture {
			format!(
				"Recording {}/{}: {} · Esc cancels",
				capture.keys.len(),
				capture.count,
				sequence_label(&capture.keys)
			)
		} else if let Some(error) = &self.error {
			error.clone()
		} else if let Some(message) = &self.message {
			message.clone()
		} else if self.timeout.is_some() {
			"Milliseconds · Enter confirms · Esc cancels".into()
		} else if let Focus::Binding(slot) = self.focus
			&& let Some(action) = self.selected
		{
			let value = self.draft.bindings[&action][slot]
				.as_ref()
				.map(|s| sequence_label(s))
				.unwrap_or_else(|| "Unset".into());
			format!(
				"{}: {value} · Enter/1–9 record · Del unset",
				["Primary", "Alternate 1", "Alternate 2"][slot]
			)
		} else {
			"Tab focus · Enter edit · 1–9 record · Ctrl+L categories · Esc close".into()
		};
		let status_lines = wrap(&status, inner.width);
		let status_h = (status_lines.len() as u16)
			.min(4)
			.min(inner.height.saturating_sub(2));
		let status_area = Rect::new(
			inner.x,
			footer.y.saturating_sub(status_h),
			inner.width,
			status_h,
		);
		text(
			frame,
			status_area,
			status_lines.join("\n"),
			if self.error.is_some() {
				Style::default().fg(theme::error_color())
			} else {
				muted()
			},
		);
		let mut body = Rect::new(
			inner.x,
			inner.y,
			inner.width,
			status_area.y.saturating_sub(inner.y),
		);
		if self.narrow {
			self.button(
				frame,
				line(body, 0),
				&format!(" {} ▾", self.category.label()),
				Hit::Selector,
				self.focus == Focus::Categories,
			);
			body.y += body.height.min(2);
			body.height = body.height.saturating_sub(2);
		} else {
			let nav = Rect::new(body.x, body.y, 21, body.height);
			self.draw_categories(frame, nav);
			for y in body.y..body.bottom() {
				text(
					frame,
					Rect::new(body.x + 22, y, 1, 1),
					"│",
					Style::default().fg(theme::border_color()),
				);
			}
			body.x += 24;
			body.width = body.width.saturating_sub(24);
		}
		if self.category == SettingsSection::Appearance {
			self.draw_theme(frame, body);
		} else if matches!(
			self.category,
			SettingsSection::ThemeGroup | SettingsSection::ShortcutsGroup
		) {
			text(frame, line(body, 0), self.category.label(), accent());
			text(
				frame,
				line(body, 2),
				"Enter to expand or collapse this group",
				muted(),
			);
		} else if self.category == SettingsSection::Timeout {
			text(frame, line(body, 0), "Leader", accent());
			let description = wrap("Optional prefix for multi-key shortcuts.", body.width);
			text(
				frame,
				Rect::new(
					body.x,
					body.y + 2.min(body.height),
					body.width,
					(description.len() as u16).min(body.height.saturating_sub(2)),
				),
				description.join("\n"),
				Style::default(),
			);
			let row = line(body, description.len() as u16 + 3);
			let value = self
				.draft
				.leader
				.as_ref()
				.map(ShortcutKey::label)
				.unwrap_or_else(|| "Unset".into());
			self.button(
				frame,
				row,
				&format!(" {value} "),
				Hit::Leader,
				self.focus == Focus::Leader,
			);
			text(
				frame,
				line(body, description.len() as u16 + 5),
				"Enter records · Delete unsets · Default: Unset",
				muted(),
			);
			let offset = body.height.min(description.len() as u16 + 8);
			self.draw_timeout(
				frame,
				Rect::new(body.x, body.y + offset, body.width, body.height - offset),
			);
		} else {
			self.draw_actions(frame, body);
		}
		if self.selector_open && self.narrow {
			let popup = Rect::new(
				inner.x,
				inner.y,
				inner.width,
				9.min(inner.height.saturating_sub(1)),
			);
			frame.render_widget(Clear, popup);
			let border = block(" Categories ");
			let content = border.inner(popup);
			frame.render_widget(border, popup);
			self.hits.clear();
			self.bar = None;
			self.draw_categories(frame, content);
		}
		if let Some(change) = &self.leader_change {
			let mut lines = vec![if let Some(key) = change {
				format!("Set Leader to {}?", key.label())
			} else {
				"Unset Leader?".into()
			}];
			for (action, slots) in &self.draft.bindings {
				for (index, keys) in slots.iter().enumerate() {
					if let Some(keys) = keys {
						let affected = match change {
							Some(key) => {
								keys.first() == Some(key)
									|| keys.first().is_some_and(ShortcutKey::is_leader)
							}
							None => keys.first().is_some_and(ShortcutKey::is_leader),
						};
						if affected {
							lines.push(format!(
								"{} slot {}: {} — {}",
								action.spec().label,
								index + 1,
								sequence_label(keys),
								if change.is_none() {
									"replace or remove"
								} else if keys.first().is_some_and(ShortcutKey::is_leader) {
									"follow new leader"
								} else if keys.len() == 1 {
									"unset"
								} else {
									"convert to <leader>"
								}
							));
						}
					}
				}
			}
			let width = inner.width.min(72);
			let wrapped = wrap(&lines.join("\n"), width.saturating_sub(2));
			let height = (wrapped.len() as u16 + 4).min(inner.height).min(14);
			let popup = Rect::new(
				inner.x + (inner.width - width) / 2,
				inner.y + (inner.height - height) / 2,
				width,
				height,
			);
			frame.render_widget(Clear, popup);
			let border = block("");
			let content = border.inner(popup);
			frame.render_widget(border, popup);
			let text_area = Rect::new(
				content.x,
				content.y,
				content.width,
				content.height.saturating_sub(2),
			);
			self.leader_scroll = self
				.leader_scroll
				.min(wrapped.len().saturating_sub(text_area.height as usize) as u16);
			frame.render_widget(
				Paragraph::new(wrapped.join("\n")).scroll((self.leader_scroll, 0)),
				text_area,
			);
			let labels: &[&str] = if change.is_some() {
				&["Apply", "Cancel"]
			} else {
				&["Preserve", "Remove", "Cancel"]
			};
			self.hits.clear();
			let row = line(content, content.height.saturating_sub(1));
			let total: u16 = labels.iter().map(|s| s.len() as u16 + 2).sum();
			let mut x = row.right().saturating_sub(total).max(row.x);
			for (index, label) in labels.iter().enumerate() {
				let width = (label.len() as u16 + 2).min(row.right().saturating_sub(x));
				self.button(
					frame,
					Rect::new(x, row.y, width, row.height),
					&format!(" {label} "),
					Hit::LeaderChoice(index),
					self.leader_choice == index,
				);
				x += width;
			}
			return;
		}
		if let Some(discard) = self.discard {
			let width = inner.width.min(48);
			let height = inner.height.min(5);
			let popup = Rect::new(
				inner.x + (inner.width - width) / 2,
				inner.y + (inner.height - height) / 2,
				width,
				height,
			);
			frame.render_widget(Clear, popup);
			let border = block("");
			let content = border.inner(popup);
			frame.render_widget(border, popup);
			text(
				frame,
				line(content, 0),
				" Discard unsaved settings changes?",
				Style::default(),
			);
			self.hits.clear();
			self.bar = None;
			self.nav_bar = None;
			let row = line(content, content.height.saturating_sub(1));
			let half = row.width / 2;
			self.button(
				frame,
				Rect::new(row.x, row.y, half, row.height),
				" Discard ",
				Hit::Discard,
				discard,
			);
			self.button(
				frame,
				Rect::new(row.x + half, row.y, row.width - half, row.height),
				" Keep editing ",
				Hit::Keep,
				!discard,
			);
		}
		if self.preset_open {
			let width = inner.width.min(38);
			let height = inner.height.min(12);
			let popup = Rect::new(
				inner.x + (inner.width - width) / 2,
				inner.y + (inner.height - height) / 2,
				width,
				height,
			);
			frame.render_widget(Clear, popup);
			let border = block(" Theme preset ");
			let content = border.inner(popup);
			frame.render_widget(border, popup);
			self.hits.clear();
			let height = content.height as usize;
			if height > 0 {
				if self.preset_cursor < self.preset_top {
					self.preset_top = self.preset_cursor;
				}
				if self.preset_cursor >= self.preset_top + height {
					self.preset_top = self.preset_cursor + 1 - height;
				}
				for (index, (_, label)) in crate::theme::PRESETS
					.iter()
					.enumerate()
					.skip(self.preset_top)
					.take(height)
				{
					self.button(
						frame,
						Rect::new(
							content.x,
							content.y + (index - self.preset_top) as u16,
							content.width.saturating_sub(1),
							1,
						),
						&format!(" {label}"),
						Hit::Preset(index),
						index == self.preset_cursor,
					);
				}
				if crate::theme::PRESETS.len() > height && content.width > 0 {
					let bar = Bar {
						target: Target::Tools,
						area: Rect::new(content.right() - 1, content.y, 1, content.height),
						content: crate::theme::PRESETS.len(),
						viewport: height,
						offset: self.preset_top,
					};
					scrollbar::draw(frame, &mut Vec::new(), bar);
				}
			}
		}

		self.follow = false;
	}
	fn draw_categories(&mut self, frame: &mut Frame, area: Rect) {
		let height = area.height as usize;
		if height == 0 {
			return;
		}
		let categories = self.categories();
		let at = categories
			.iter()
			.position(|cat| *cat == self.category)
			.unwrap();
		self.nav_top = self.nav_top.min(categories.len().saturating_sub(height));
		if self.follow {
			if at < self.nav_top {
				self.nav_top = at;
			} else if at >= self.nav_top + height {
				self.nav_top = at + 1 - height;
			}
		}
		for (i, cat) in categories
			.iter()
			.copied()
			.enumerate()
			.skip(self.nav_top)
			.take(height)
		{
			let row = Rect::new(
				area.x,
				area.y + (i - self.nav_top) as u16,
				area.width.saturating_sub(1),
				1,
			);
			let selected = cat == self.category;
			text(
				frame,
				row,
				format!(
					"{}{}",
					if matches!(
						cat,
						SettingsSection::ThemeGroup | SettingsSection::ShortcutsGroup
					) {
						if self.closed_groups.contains(&cat) {
							"▸ "
						} else {
							"▾ "
						}
					} else if selected {
						"  › "
					} else {
						"    "
					},
					cat.label()
				),
				if selected {
					accent().bg(theme::selected_color())
				} else {
					Style::default()
				},
			);
			self.hits.push((row, Hit::Category(cat)));
		}
		if height < categories.len() {
			let bar = Bar {
				target: Target::Tools,
				area: Rect::new(area.right() - 1, area.y, 1, area.height),
				content: categories.len(),
				viewport: height,
				offset: self.nav_top,
			};
			scrollbar::draw(frame, &mut Vec::new(), bar);
			self.nav_bar = Some(bar);
		}
	}
	fn draw_theme(&mut self, frame: &mut Frame, area: Rect) {
		if area.is_empty() {
			return;
		}
		text(frame, line(area, 0), "Appearance", accent());
		text(frame, line(area, 2), "Preset", Style::default());
		let label = crate::theme::PRESETS
			.iter()
			.find(|(id, _)| *id == self.theme_draft.preset)
			.map(|(_, name)| *name)
			.unwrap_or("Midnight");
		let row = line(area, 3);
		self.button(
			frame,
			Rect::new(row.x, row.y, row.width.min(30), row.height),
			&format!(" {label} ▾"),
			Hit::PresetOpen,
			self.focus == Focus::Preset,
		);
		let reset = line(area, 5);
		self.button(
			frame,
			Rect::new(reset.x, reset.y, reset.width.min(13), reset.height),
			" Reset theme ",
			Hit::ResetTheme,
			self.focus == Focus::ResetTheme,
		);
		let offset = area.height.min(7);
		let list = Rect::new(area.x, area.y + offset, area.width, area.height - offset);
		self.viewport = list.height as usize;
		if list.is_empty() {
			return;
		}
		let width = list.width.saturating_sub(1);
		let stacked = width < 48;
		let rows = self.theme_rows();
		let mut positions = Vec::new();
		let mut total = 0usize;
		for (index, row) in rows.iter().enumerate() {
			if matches!(row, ThemeRow::Section(_)) && index > 0 {
				total += 1;
			}
			positions.push(total);
			total += if matches!(row, ThemeRow::Section(_)) || !stacked {
				2
			} else {
				3
			};
		}
		if self.follow
			&& let Some(position) = positions.get(self.theme_cursor)
		{
			let needed = if stacked && matches!(rows[self.theme_cursor], ThemeRow::Color(_)) {
				2
			} else {
				1
			};
			if *position < self.top {
				self.top = *position;
			} else if position + needed > self.top + self.viewport {
				self.top = (position + needed).saturating_sub(self.viewport);
			}
		}
		self.top = self.top.min(total.saturating_sub(self.viewport));
		let resolved = self.preview();
		for (index, row) in rows.iter().enumerate() {
			let position = positions[index];
			let focused = self.focus == Focus::List && self.theme_cursor == index;
			if position >= self.top && position < self.top + self.viewport {
				let rect = Rect::new(list.x, list.y + (position - self.top) as u16, width, 1);
				match row {
					ThemeRow::Section(section) => {
						text(
							frame,
							rect,
							format!(
								"{} {}",
								if self.theme_collapsed.contains(section) {
									"▸"
								} else {
									"▾"
								},
								THEME_SECTIONS[*section].0
							),
							if focused {
								accent().bg(theme::selected_color())
							} else {
								accent()
							},
						);
						self.hits.push((rect, Hit::ThemeRow(index)));
					}
					ThemeRow::Color(role) => {
						let label_width = if stacked {
							width
						} else {
							width.saturating_sub(29)
						};
						let label = Rect::new(rect.x, rect.y, label_width, 1);
						text(
							frame,
							label,
							format!("  {}", crate::theme::ROLES[*role].1),
							if focused {
								Style::default().add_modifier(Modifier::BOLD)
							} else {
								Style::default()
							},
						);
						self.hits.push((label, Hit::ThemeRow(index)));
					}
				}
			}
			let ThemeRow::Color(role_index) = row else {
				continue;
			};
			let field_position = position + usize::from(stacked);
			if field_position < self.top || field_position >= self.top + self.viewport {
				continue;
			}
			let role = crate::theme::ROLES[*role_index].0;
			let color = resolved.color(role);
			let value = if *role_index == self.theme_row {
				self.theme_input.clone()
			} else {
				None
			}
			.unwrap_or_else(|| {
				self.theme_draft
					.colors
					.get(role)
					.cloned()
					.unwrap_or_else(|| match color {
						Color::Reset => "default".into(),
						Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
						Color::Indexed(n) => format!("ansi:{n}"),
						other => format!("{other:?}"),
					})
			});
			let x = if stacked {
				list.x + 2.min(width)
			} else {
				list.x + width.saturating_sub(29)
			};
			let y = list.y + (field_position - self.top) as u16;
			let remaining = list.x + width - x;
			let field_width = remaining.saturating_sub(12).clamp(1, 16).min(remaining);
			self.button(
				frame,
				Rect::new(x, y, field_width, 1),
				&format!(
					" {value}{}",
					if self.theme_input.is_some() && *role_index == self.theme_row {
						"▏"
					} else {
						""
					}
				),
				Hit::ThemeRow(index),
				focused,
			);
			let swatch_x = x + field_width + 1;
			if swatch_x + 3 <= list.x + width {
				text(
					frame,
					Rect::new(swatch_x, y, 3, 1),
					"   ",
					Style::default().bg(color),
				);
			}
			let reset_x = swatch_x + 4;
			if reset_x < list.x + width {
				self.button(
					frame,
					Rect::new(reset_x, y, (list.x + width - reset_x).min(7), 1),
					" Reset ",
					Hit::ThemeReset(*role_index),
					self.focus == Focus::Reset && self.theme_cursor == index,
				);
			}
		}
		if total > self.viewport {
			let bar = Bar {
				target: Target::Tools,
				area: Rect::new(list.right() - 1, list.y, 1, list.height),
				content: total,
				viewport: self.viewport,
				offset: self.top,
			};
			scrollbar::draw(frame, &mut Vec::new(), bar);
			self.bar = Some(bar);
		}
	}

	fn draw_timeout(&mut self, frame: &mut Frame, area: Rect) {
		if area.is_empty() {
			return;
		}
		text(frame, line(area, 0), "Sequence Timeout", accent());
		let lines = wrap(
			"How long to wait for the next key in a multi-key shortcut.",
			area.width,
		);
		let h = lines.len() as u16;
		text(
			frame,
			Rect::new(
				area.x,
				area.y + 2.min(area.height),
				area.width,
				h.min(area.height.saturating_sub(2)),
			),
			lines.join("\n"),
			Style::default(),
		);
		let row = line(area, (h + 3).min(area.height.saturating_sub(1)));
		let value = self
			.timeout
			.clone()
			.unwrap_or_else(|| self.draft.timeout_ms.to_string());
		self.button(
			frame,
			row,
			&format!(" {value} ms "),
			Hit::Timeout,
			self.focus == Focus::Timeout,
		);
		if row.y + 2 < area.bottom() {
			text(
				frame,
				line(area, h + 5),
				"1–60000 ms · Default: 1000",
				muted(),
			);
		}
	}
	fn draw_actions(&mut self, frame: &mut Frame, area: Rect) {
		if area.is_empty() {
			return;
		}
		let heading = u16::from(!self.narrow);
		if heading > 0 {
			text(frame, line(area, 0), self.category.label(), accent());
		}
		let detail_h = if self.narrow { 11 } else { 9 };
		let detail_h = detail_h.min(area.height.saturating_sub(5));
		let list = Rect::new(
			area.x,
			area.y + heading,
			area.width,
			area.height.saturating_sub(detail_h + heading),
		);
		self.draw_list(frame, list);
		if detail_h > 0 {
			self.draw_detail(
				frame,
				Rect::new(area.x, list.bottom(), area.width, detail_h),
			);
		}
	}
	fn draw_list(&mut self, frame: &mut Frame, area: Rect) {
		self.viewport = area.height as usize;
		if area.is_empty() {
			return;
		}
		let width = area.width.saturating_sub(2);
		let mut rendered: Vec<(Option<usize>, Line<'static>)> = Vec::new();
		self.positions.clear();
		for (index, row) in self.rows.iter().enumerate() {
			if matches!(row, Row::Section(_)) && index > 0 {
				rendered.push((None, Line::default()));
			}
			if let Row::Action(action) = row
				&& (matches!(
					index.checked_sub(1).and_then(|i| self.rows.get(i)),
					Some(Row::Section(_))
				) || action.starts_visual_group())
			{
				rendered.push((None, Line::default()));
			}
			self.positions.push(rendered.len());
			match *row {
				Row::Section(sub) => {
					let marker = if self.collapsed.contains(&(self.category, sub)) {
						"▸"
					} else {
						"▾"
					};
					rendered.push((
						Some(index),
						Line::styled(format!("{marker} {}", sub.label()), accent()),
					));
				}
				Row::Action(action) => {
					let keys: Vec<_> = self.draft.bindings[&action]
						.iter()
						.flatten()
						.map(|seq| sequence_label(seq))
						.collect();
					let summary = if keys.is_empty() {
						"Unset".into()
					} else {
						keys.join(" / ")
					};
					let label = format!("  {}", action.spec().label);
					if label.width() + summary.width() + 2 <= width as usize {
						let gap = width as usize - label.width() - summary.width();
						rendered.push((
							Some(index),
							Line::from(vec![
								Span::raw(label),
								Span::raw(" ".repeat(gap)),
								Span::styled(summary, muted()),
							]),
						));
					} else {
						for part in wrap(&label, width) {
							rendered.push((Some(index), Line::raw(part)));
						}
						for part in wrap(&format!("    {summary}"), width) {
							rendered.push((Some(index), Line::styled(part, muted())));
						}
					}
				}
			}
		}
		if self.follow
			&& let Some(pos) = self.positions.get(self.row)
		{
			if *pos < self.top {
				self.top = *pos;
			} else if *pos >= self.top + self.viewport {
				self.top = pos + 1 - self.viewport;
			}
		}
		self.top = self.top.min(rendered.len().saturating_sub(self.viewport));
		for (y, (index, value)) in rendered
			.iter()
			.enumerate()
			.skip(self.top)
			.take(self.viewport)
		{
			let row = Rect::new(area.x, area.y + (y - self.top) as u16, width, 1);
			let style = if *index == Some(self.row) {
				Style::default().bg(theme::selected_color()).add_modifier(
					if self.focus == Focus::List {
						Modifier::BOLD
					} else {
						Modifier::empty()
					},
				)
			} else {
				Style::default()
			};
			frame.render_widget(Paragraph::new(value.clone()).style(style), row);
			if let Some(index) = index {
				self.hits.push((row, Hit::Row(*index)));
			}
		}
		if rendered.len() > self.viewport {
			let bar = Bar {
				target: Target::Tools,
				area: Rect::new(area.right() - 1, area.y, 1, area.height),
				content: rendered.len(),
				viewport: self.viewport,
				offset: self.top,
			};
			scrollbar::draw(frame, &mut Vec::new(), bar);
			self.bar = Some(bar);
		}
	}
	fn draw_detail(&mut self, frame: &mut Frame, area: Rect) {
		let Some(action) = self.selected else {
			return;
		};
		let border = block(action.spec().label);
		let inner = border.inner(area);
		frame.render_widget(border, area);
		if inner.is_empty() {
			return;
		}
		let slots = &self.draft.bindings[&action];
		let values: Vec<_> = slots
			.iter()
			.map(|seq| {
				seq.as_ref()
					.map(|s| sequence_label(s))
					.unwrap_or_else(|| "Unset".into())
			})
			.collect();
		let fields_h = if self.narrow { 3 } else { 2 };
		let button_row = inner.height.saturating_sub(1);
		let field_y = button_row.saturating_sub(fields_h + 1);
		let description = format!(
			"{}\nApplies to: {}",
			action.description(),
			action.applicability()
		);
		text(
			frame,
			Rect::new(inner.x, inner.y, inner.width, field_y),
			wrap(&description, inner.width).join("\n"),
			muted(),
		);
		if inner.height < fields_h + 2 {
			let label = ["Primary", "Alternate 1", "Alternate 2"][self.slot];
			self.button(
				frame,
				line(inner, 0),
				&format!("{label}: {}", values[self.slot]),
				Hit::Binding(self.slot),
				true,
			);
			return;
		}
		for (slot, value) in values.iter().enumerate() {
			let label = ["Primary", "Alternate 1", "Alternate 2"][slot];
			if self.narrow {
				let row = line(inner, field_y + slot as u16);
				let label_w = 12.min(row.width);
				text(
					frame,
					Rect::new(row.x, row.y, label_w, row.height),
					label,
					if slot == self.slot { accent() } else { muted() },
				);
				self.button(
					frame,
					Rect::new(row.x + label_w, row.y, row.width - label_w, row.height),
					&format!(" {value}"),
					Hit::Binding(slot),
					self.focus == Focus::Binding(slot),
				);
			} else {
				let x = inner.x + inner.width * slot as u16 / 3;
				let end = inner.x + inner.width * (slot + 1) as u16 / 3;
				let width = (end - x).saturating_sub(1);
				text(
					frame,
					Rect::new(x, inner.y + field_y, width, 1),
					label,
					if slot == self.slot { accent() } else { muted() },
				);
				self.button(
					frame,
					Rect::new(x, inner.y + field_y + 1, width, 1),
					&format!(" {value}"),
					Hit::Binding(slot),
					self.focus == Focus::Binding(slot),
				);
			}
		}
		let row = line(inner, button_row);
		let half = 16.min(row.width / 2);
		self.button(
			frame,
			Rect::new(row.x, row.y, half, row.height),
			" Reset binding ",
			Hit::Reset,
			self.focus == Focus::Reset,
		);
		self.button(
			frame,
			Rect::new(
				row.x + half + u16::from(row.width > half * 2),
				row.y,
				half,
				row.height,
			),
			" Unset binding ",
			Hit::Unset,
			self.focus == Focus::Unset,
		);
	}
}
