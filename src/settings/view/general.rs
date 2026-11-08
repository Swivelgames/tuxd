use crate::settings::interaction::Mode;
use crate::settings::view::{block, line, text, wrap};
use crate::settings::{Focus, Hit, Settings};
use crate::shortcuts::{ShortcutKey, sequence_label};
use crate::theme::{accent, muted};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Clear, Paragraph};

impl Settings {
	pub(super) fn draw_general(&mut self, frame: &mut Frame, body: Rect) {
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
			.shortcuts
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
	}
	pub(super) fn draw_leader_prompt(&mut self, frame: &mut Frame, inner: Rect) {
		if let Mode::LeaderChange(change) = &self.mode {
			let mut lines = vec![if let Some(key) = change {
				format!("Set Leader to {}?", key.label())
			} else {
				"Unset Leader?".into()
			}];
			for (action, slots) in &self.shortcuts.draft.bindings {
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
			self.general.leader_scroll = self
				.general
				.leader_scroll
				.min(wrapped.len().saturating_sub(text_area.height as usize) as u16);
			frame.render_widget(
				Paragraph::new(wrapped.join("\n")).scroll((self.general.leader_scroll, 0)),
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
					self.general.leader_choice == index,
				);
				x += width;
			}
		}
	}

	pub(super) fn draw_timeout(&mut self, frame: &mut Frame, area: Rect) {
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
			.mode
			.timeout_input()
			.map(str::to_owned)
			.unwrap_or_else(|| self.shortcuts.draft.timeout_ms.to_string());
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
}
