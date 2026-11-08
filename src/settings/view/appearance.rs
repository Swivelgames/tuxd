use crate::components::scrollbar::{self, Axis, Bar};
use crate::settings::interaction::Mode;
use crate::settings::view::{block, line, text};
use crate::settings::{Focus, Hit, Settings, ThemeRow};
use crate::theme::{self, accent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Clear;

impl Settings {
	pub(super) fn draw_theme(&mut self, frame: &mut Frame, area: Rect) {
		if area.is_empty() {
			return;
		}
		text(frame, line(area, 0), "Appearance", accent());
		text(frame, line(area, 2), "Preset", Style::default());
		let label = crate::theme::PRESETS
			.iter()
			.find(|(id, _)| *id == self.appearance.draft.preset)
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
			&& let Some(position) = positions.get(self.appearance.cursor)
		{
			let needed = if stacked && matches!(rows[self.appearance.cursor], ThemeRow::Color(_)) {
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
			let focused = self.focus == Focus::List && self.appearance.cursor == index;
			if position >= self.top && position < self.top + self.viewport {
				let rect = Rect::new(list.x, list.y + (position - self.top) as u16, width, 1);
				match row {
					ThemeRow::Section(section) => {
						text(
							frame,
							rect,
							format!(
								"{} {}",
								if self.appearance.collapsed.contains(section) {
									"▸"
								} else {
									"▾"
								},
								section.label()
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
							format!("  {}", role.label()),
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
			let ThemeRow::Color(selected_role) = row else {
				continue;
			};
			let field_position = position + usize::from(stacked);
			if field_position < self.top || field_position >= self.top + self.viewport {
				continue;
			}
			let role = selected_role.id();
			let color = crate::theme::color(resolved.color(*selected_role));
			let value = if *selected_role == self.appearance.role {
				self.mode.color_input().map(str::to_owned)
			} else {
				None
			}
			.unwrap_or_else(|| {
				self.appearance
					.draft
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
					if matches!(self.mode, Mode::Color(_)) && *selected_role == self.appearance.role
					{
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
					Hit::ThemeReset(*selected_role),
					self.focus == Focus::Reset && self.appearance.cursor == index,
				);
			}
		}
		if total > self.viewport {
			let bar = Bar {
				target: Axis::Vertical,
				area: Rect::new(list.right() - 1, list.y, 1, list.height),
				content: total,
				viewport: self.viewport,
				offset: self.top,
			};
			scrollbar::draw(frame, &mut Vec::new(), bar);
			self.bar = Some(bar);
		}
	}
}

impl Settings {
	pub(super) fn draw_preset_popup(&mut self, frame: &mut Frame, inner: Rect) {
		if matches!(self.mode, Mode::Preset) {
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
				if self.appearance.preset_cursor < self.appearance.preset_top {
					self.appearance.preset_top = self.appearance.preset_cursor;
				}
				if self.appearance.preset_cursor >= self.appearance.preset_top + height {
					self.appearance.preset_top = self.appearance.preset_cursor + 1 - height;
				}
				for (index, (_, label)) in crate::theme::PRESETS
					.iter()
					.enumerate()
					.skip(self.appearance.preset_top)
					.take(height)
				{
					self.button(
						frame,
						Rect::new(
							content.x,
							content.y + (index - self.appearance.preset_top) as u16,
							content.width.saturating_sub(1),
							1,
						),
						&format!(" {label}"),
						Hit::Preset(index),
						index == self.appearance.preset_cursor,
					);
				}
				if crate::theme::PRESETS.len() > height && content.width > 0 {
					let bar = Bar {
						target: Axis::Vertical,
						area: Rect::new(content.right() - 1, content.y, 1, content.height),
						content: crate::theme::PRESETS.len(),
						viewport: height,
						offset: self.appearance.preset_top,
					};
					scrollbar::draw(frame, &mut Vec::new(), bar);
				}
			}
		}
	}
}
