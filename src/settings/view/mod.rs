use crate::settings::interaction::Mode;
use crate::settings::sections::Section;
use crate::settings::{Focus, Hit, Settings};
use crate::shortcuts::sequence_label;
use ratatui::layout::Rect;
mod appearance;
mod general;
mod navigation;
mod shortcuts;
use crate::theme;
use ratatui::{
	Frame,
	style::Style,
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
		} else if matches!(self.mode, Mode::Color(_)) && self.error.is_none() {
			"Color: #RRGGBB / ansi:0–255 / default · Enter accepts · Esc cancels".into()
		} else if matches!(self.mode, Mode::LeaderCapture) {
			"Recording Leader · Esc cancels".into()
		} else if let Mode::Capture(capture) = &self.mode {
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
		} else if matches!(self.mode, Mode::Timeout(_)) {
			"Milliseconds · Enter confirms · Esc cancels".into()
		} else if let Focus::Binding(slot) = self.focus
			&& let Some(action) = self.shortcuts.selected
		{
			let value = self.shortcuts.draft.bindings[&action][slot]
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
		if self.category == Section::Appearance {
			self.draw_theme(frame, body);
		} else if matches!(self.category, Section::ThemeGroup | Section::ShortcutsGroup) {
			text(frame, line(body, 0), self.category.label(), accent());
			text(
				frame,
				line(body, 2),
				"Enter to expand or collapse this group",
				muted(),
			);
		} else if self.category == Section::Timeout {
			self.draw_general(frame, body);
		} else {
			self.draw_actions(frame, body);
		}
		if matches!(self.mode, Mode::Selector) && self.narrow {
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
		if matches!(self.mode, Mode::LeaderChange(_)) {
			self.draw_leader_prompt(frame, inner);
			return;
		}
		if let Mode::Discard(discard) = self.mode {
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
		self.draw_preset_popup(frame, inner);

		self.follow = false;
	}
}
