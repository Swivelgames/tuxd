//! Inert source preview and independently scrollable import diagnostics.
mod projection;

use crate::theme;
use crate::{
	components::scrollbar::{self, Axis, Bar},
	libraries::ansi::Import,
	libraries::text::Layout,
	libraries::text::display::sanitize,
};
use crossterm::event::{Event, MouseButton, MouseEventKind};
use ratatui::{
	Frame,
	layout::{Constraint, Rect},
	style::{Modifier, Style},
	widgets::{BorderType, Clear, Paragraph, Row, Table},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
	Strip,
	Abort,
}

impl Decision {
	fn from_button(index: usize) -> Self {
		if index == 0 { Self::Strip } else { Self::Abort }
	}
}

pub struct Presentation<'a> {
	pub title: &'a str,
	pub explanation: &'a str,
	pub accept: &'a str,
}

pub struct Review {
	preview: crate::components::text_surface::Surface,
	errors_top: usize,
	areas: [Rect; 2],
	error_bar: Option<Bar>,
	buttons: Vec<(Rect, usize)>,
	focus: usize,
	drag: Option<(usize, usize)>,
}

impl Default for Review {
	fn default() -> Self {
		Self {
			preview: Default::default(),
			errors_top: 0,
			areas: [Rect::default(); 2],
			error_bar: None,
			buttons: Vec::new(),
			focus: 3,
			drag: None,
		}
	}
}

impl Review {
	fn bar(&self, index: usize) -> Option<Bar> {
		if index == 0 {
			self.preview.bars.first().copied()
		} else {
			self.error_bar
		}
	}

	fn offset_mut(&mut self, index: usize) -> &mut usize {
		if index == 0 {
			&mut self.preview.viewport.top
		} else {
			&mut self.errors_top
		}
	}

	#[cfg(test)]
	pub fn has_scrollbar(&self) -> bool {
		self.error_bar.is_some() || !self.preview.bars.is_empty()
	}

	#[cfg(test)]
	pub fn scrolled(&self) -> bool {
		self.preview.viewport.top > 0 || self.errors_top > 0
	}

	pub fn input_scope(&self) -> crate::shortcuts::InputScope {
		if self.focus >= 2 {
			crate::shortcuts::InputScope::ReviewButtons
		} else {
			crate::shortcuts::InputScope::ReviewContent
		}
	}

	pub fn shortcut(&mut self, action: crate::shortcuts::ActionId) -> Option<Decision> {
		use crate::shortcuts::ActionId as A;
		let at = self.focus.min(1);
		match action {
			A::Close | A::Abort | A::Negative => return Some(Decision::Abort),
			A::Positive => return Some(Decision::Strip),
			A::NextField => self.focus = (self.focus + 1) % 4,
			A::PreviousField => self.focus = (self.focus + 3) % 4,
			A::FieldLeft | A::FieldRight | A::FieldUp | A::FieldDown if self.focus >= 2 => {
				self.focus = 5 - self.focus
			}
			A::Activate if self.focus >= 2 => return Some(Decision::from_button(self.focus - 2)),
			A::FieldUp => *self.offset_mut(at) = (*self.offset_mut(at)).saturating_sub(1),
			A::FieldDown => *self.offset_mut(at) = (*self.offset_mut(at)).saturating_add(1),
			A::FieldPageUp => *self.offset_mut(at) = (*self.offset_mut(at)).saturating_sub(5),
			A::FieldPageDown => *self.offset_mut(at) = (*self.offset_mut(at)).saturating_add(5),
			A::FieldHome => *self.offset_mut(at) = 0,
			A::FieldEnd => *self.offset_mut(at) = usize::MAX,
			_ => (),
		}
		None
	}

	pub fn event(&mut self, event: &Event) -> Option<Decision> {
		if let Event::Mouse(mouse) = event {
			let point = (mouse.column, mouse.row).into();
			if self.preview.scrollbar_event(*mouse) {
				self.focus = 0;
				return None;
			}
			match mouse.kind {
				MouseEventKind::Up(MouseButton::Left) => self.drag = None,
				MouseEventKind::Drag(MouseButton::Left) => {
					if let Some((i, grab)) = self.drag
						&& let Some(bar) = self.bar(i)
					{
						*self.offset_mut(i) =
							bar.offset_at(bar.coordinate(mouse.column, mouse.row), grab);
					}
				}
				MouseEventKind::Down(MouseButton::Left) => {
					for (rect, choice) in &self.buttons {
						if rect.contains(point) {
							return Some(Decision::from_button(*choice));
						}
					}
					for i in 0..2 {
						if self.areas[i].contains(point) {
							self.focus = i;
						}
						if let Some(bar) = self.bar(i).filter(|bar| bar.area.contains(point)) {
							let at = bar.coordinate(mouse.column, mouse.row);
							let grab = bar.grab_at(at);
							*self.offset_mut(i) = bar.offset_at(at, grab);
							self.drag = Some((i, grab));
						}
					}
				}
				MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
					for i in 0..2 {
						if self.areas[i].contains(point) {
							self.focus = i;
							*self.offset_mut(i) = if mouse.kind == MouseEventKind::ScrollUp {
								(*self.offset_mut(i)).saturating_sub(3)
							} else {
								(*self.offset_mut(i)).saturating_add(3)
							};
						}
					}
				}
				_ => (),
			}
		}
		None
	}

	pub fn draw(&mut self, frame: &mut Frame, import: &Import, presentation: Presentation<'_>) {
		let screen = frame.area();
		let width = screen
			.width
			.saturating_sub(if screen.width > 40 { 4 } else { 0 })
			.min(100);
		let content_width = width.saturating_sub(12) as usize;
		let projection = projection::Projection::new(import);
		let lines = &projection.lines;
		let marks = &projection.marks;
		let layout = Layout::new(lines, content_width, true, false);
		let preview_height = layout.rows.len().clamp(1, 5) as u16;
		let error_height = import.issues.len().clamp(1, 5) as u16;
		let height = (14 + preview_height + error_height).min(screen.height);
		let popup = Rect::new(
			(screen.width - width) / 2,
			(screen.height - height) / 2,
			width,
			height,
		);
		frame.render_widget(Clear, popup);
		let block = theme::block()
			.border_type(BorderType::Rounded)
			.title(" Unsupported ANSI Sequences ");
		let inner = block.inner(popup);
		frame.render_widget(block, popup);
		self.error_bar = None;
		self.preview.hide();
		self.buttons.clear();
		self.areas = [Rect::default(); 2];
		if inner.width < 2 || inner.height < 3 {
			return;
		}
		let row = |y: u16| {
			Rect::new(
				inner.x + 1,
				inner.y + y.min(inner.height),
				inner.width.saturating_sub(2),
				u16::from(y < inner.height),
			)
		};
		frame.render_widget(
			Paragraph::new("Any unsupported ANSI sequences will be stripped."),
			row(if screen.height < 18 { 0 } else { 1 }),
		);
		frame.render_widget(Paragraph::new(presentation.explanation), row(3));
		let preview = Rect::new(
			inner.x + 1,
			inner.y + if screen.height < 18 { 1 } else { 5 },
			inner.width.saturating_sub(2),
			(preview_height + 2).min(inner.height.saturating_sub(if screen.height < 18 {
				6
			} else {
				10
			})),
		);
		let block = theme::block()
			.border_type(BorderType::Rounded)
			.title(format!(" {} ", sanitize(presentation.title)));
		let inside = block.inner(preview);
		frame.render_widget(block, preview);
		self.areas[0] = inside;
		self.preview.viewport.top = (self.preview.viewport.top)
			.min(layout.rows.len().saturating_sub(inside.height as usize));
		let red = Style::default()
			.fg(theme::error_color())
			.add_modifier(Modifier::BOLD);
		let gutter = inside.width.min(6);
		let text = Rect::new(
			inside.x + gutter,
			inside.y,
			inside.width.saturating_sub(gutter),
			inside.height,
		);
		self.preview.viewport.wrap = true;
		self.preview.viewport.width = text.width as usize;
		self.preview.viewport.height = text.height as usize;
		self.preview.draw(
			frame,
			preview,
			inside,
			text,
			crate::components::text_surface::Content {
				lines,
				layout: &layout,
				active_line: None,
				cursor_space: false,
				gutter_style: Style::default().bg(theme::field_color()),
			},
			|position| {
				if marks[position.row]
					.get(position.byte)
					.copied()
					.unwrap_or(false)
				{
					red
				} else {
					Style::default()
				}
			},
		);
		let table_y = preview
			.bottom()
			.saturating_add(u16::from(screen.height >= 18));
		let table_height = (error_height + 2).min(inner.bottom().saturating_sub(table_y + 2));
		let table_area = Rect::new(
			inner.x + 2,
			table_y,
			inner.width.saturating_sub(4),
			table_height,
		);
		self.areas[1] = table_area;
		let visible = table_height.saturating_sub(2) as usize;
		self.errors_top = (self.errors_top).min(import.issues.len().saturating_sub(visible));
		let rows = import
			.issues
			.iter()
			.skip(self.errors_top)
			.take(visible)
			.map(|issue| {
				Row::new(vec![
					ratatui::widgets::Cell::from(
						import.source[issue.start..issue.end]
							.chars()
							.map(projection::display_character)
							.collect::<String>(),
					)
					.style(red),
					ratatui::widgets::Cell::from(issue.reason.clone()),
				])
			});
		let seq_width = import
			.issues
			.iter()
			.map(|i| i.notation.len())
			.max()
			.unwrap_or(4)
			.clamp(6, 24) as u16;
		frame.render_widget(
			Table::new(rows, [Constraint::Length(seq_width), Constraint::Min(1)])
				.column_spacing(3)
				.header(
					Row::new(["Seq.", "Error"])
						.style(Style::default().add_modifier(Modifier::BOLD))
						.bottom_margin(1),
				)
				.style(Style::default().bg(theme::field_color())),
			table_area,
		);
		let mut bars = Vec::new();
		scrollbar::draw(
			frame,
			&mut bars,
			Bar {
				target: Axis::Vertical,
				area: Rect::new(
					table_area.right(),
					table_area.y + 2,
					1,
					table_height.saturating_sub(2),
				),
				content: import.issues.len(),
				viewport: visible,
				offset: self.errors_top,
			},
		);
		self.error_bar = bars.first().copied();
		let button_y = inner.bottom() - 1;
		let mut x = inner.right().saturating_sub(26).max(inner.x);
		for (i, label) in [presentation.accept, "Abort"].iter().enumerate() {
			let rect = Rect::new(
				x,
				button_y,
				(label.len() as u16 + 2).min(inner.right().saturating_sub(x)),
				1,
			);
			frame.render_widget(
				Paragraph::new(format!(" {label} ")).style(
					Style::default()
						.fg(theme::accent_color())
						.bg(if self.focus == i + 2 {
							theme::selected_color()
						} else {
							theme::surface_color()
						}),
				),
				rect,
			);
			self.buttons.push((rect, i));
			x += label.len() as u16 + 4;
		}
	}
}

#[cfg(test)]
mod tests;
