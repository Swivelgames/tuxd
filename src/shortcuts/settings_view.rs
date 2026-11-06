use super::*;
use crate::scrollbar::{self, Target};
use ratatui::{
	Frame,
	style::{Color, Modifier, Style},
	text::{Line, Span},
	widgets::{Block, BorderType, Clear, Paragraph},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const SELECTED: Color = Color::Rgb(41, 54, 83);
const FIELD: Color = Color::Rgb(23, 28, 44);
const BORDER: Color = Color::Rgb(62, 78, 106);
fn accent() -> Style {
	Style::default()
		.fg(Color::Cyan)
		.add_modifier(Modifier::BOLD)
}
fn muted() -> Style {
	Style::default().fg(Color::Rgb(137, 151, 183))
}
fn block(title: &str) -> Block<'_> {
	Block::bordered()
		.border_type(BorderType::Rounded)
		.border_style(Style::default().fg(BORDER))
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
			accent().bg(SELECTED)
		} else {
			Style::default().bg(FIELD)
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
		let outer = block(if self.draft == self.original {
			" Shortcut Settings "
		} else {
			" Shortcut Settings · unsaved "
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
		let status = if let Some(capture) = &self.capture {
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
				Style::default().fg(Color::Red)
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
					Style::default().fg(BORDER),
				);
			}
			body.x += 24;
			body.width = body.width.saturating_sub(24);
		}
		if self.category == SettingsSection::Timeout {
			self.draw_timeout(frame, body);
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
				" Discard unsaved shortcut changes?",
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
		self.follow = false;
	}
	fn draw_categories(&mut self, frame: &mut Frame, area: Rect) {
		let height = area.height as usize;
		if height == 0 {
			return;
		}
		let at = SettingsSection::ALL
			.iter()
			.position(|cat| *cat == self.category)
			.unwrap();
		self.nav_top = self.nav_top.min(7usize.saturating_sub(height));
		if self.follow {
			if at < self.nav_top {
				self.nav_top = at;
			} else if at >= self.nav_top + height {
				self.nav_top = at + 1 - height;
			}
		}
		for (i, cat) in SettingsSection::ALL
			.into_iter()
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
				format!("{}{}", if selected { "› " } else { "  " }, cat.label()),
				if selected {
					accent().bg(SELECTED)
				} else {
					Style::default()
				},
			);
			self.hits.push((row, Hit::Category(cat)));
		}
		if height < 7 {
			let bar = Bar {
				target: Target::Tools,
				area: Rect::new(area.right() - 1, area.y, 1, area.height),
				content: 7,
				viewport: height,
				offset: self.nav_top,
			};
			scrollbar::draw(frame, &mut Vec::new(), bar);
			self.nav_bar = Some(bar);
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
				Style::default()
					.bg(SELECTED)
					.add_modifier(if self.focus == Focus::List {
						Modifier::BOLD
					} else {
						Modifier::empty()
					})
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
