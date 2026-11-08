use crate::components::scrollbar::{self, Axis, Bar};
use crate::settings::view::{block, line, text, wrap};
use crate::settings::{Focus, Hit, Row, Settings};
use crate::shortcuts::sequence_label;
use crate::theme::{self, accent, muted};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

impl Settings {
	pub(super) fn draw_actions(&mut self, frame: &mut Frame, area: Rect) {
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

	pub(super) fn draw_list(&mut self, frame: &mut Frame, area: Rect) {
		self.viewport = area.height as usize;
		if area.is_empty() {
			return;
		}
		let width = area.width.saturating_sub(2);
		let mut rendered: Vec<(Option<usize>, Line<'static>)> = Vec::new();
		self.shortcuts.positions.clear();
		for (index, row) in self.shortcuts.rows.iter().enumerate() {
			if matches!(row, Row::Section(_)) && index > 0 {
				rendered.push((None, Line::default()));
			}
			if let Row::Action(action) = row
				&& (matches!(
					index
						.checked_sub(1)
						.and_then(|i| self.shortcuts.rows.get(i)),
					Some(Row::Section(_))
				) || action.starts_visual_group())
			{
				rendered.push((None, Line::default()));
			}
			self.shortcuts.positions.push(rendered.len());
			match *row {
				Row::Section(sub) => {
					let marker = if self.shortcuts.collapsed.contains(&(self.category, sub)) {
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
					let keys: Vec<_> = self.shortcuts.draft.bindings[&action]
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
			&& let Some(pos) = self.shortcuts.positions.get(self.shortcuts.row)
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
			let style = if *index == Some(self.shortcuts.row) {
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
				target: Axis::Vertical,
				area: Rect::new(area.right() - 1, area.y, 1, area.height),
				content: rendered.len(),
				viewport: self.viewport,
				offset: self.top,
			};
			scrollbar::draw(frame, &mut Vec::new(), bar);
			self.bar = Some(bar);
		}
	}

	pub(super) fn draw_detail(&mut self, frame: &mut Frame, area: Rect) {
		let Some(action) = self.shortcuts.selected else {
			return;
		};
		let border = block(action.spec().label);
		let inner = border.inner(area);
		frame.render_widget(border, area);
		if inner.is_empty() {
			return;
		}
		let slots = &self.shortcuts.draft.bindings[&action];
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
			let label = ["Primary", "Alternate 1", "Alternate 2"][self.shortcuts.slot];
			self.button(
				frame,
				line(inner, 0),
				&format!("{label}: {}", values[self.shortcuts.slot]),
				Hit::Binding(self.shortcuts.slot),
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
					if slot == self.shortcuts.slot {
						accent()
					} else {
						muted()
					},
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
					if slot == self.shortcuts.slot {
						accent()
					} else {
						muted()
					},
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
