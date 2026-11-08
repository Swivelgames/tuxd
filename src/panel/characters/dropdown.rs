//! Transient choices: navigation previews a choice, activation commits it.
use super::Characters;
use crate::{
	libraries::{
		characters::CATALOG,
		controls::{self, Direction},
	},
	panel::{Control, Panel},
	shortcuts::ActionId as A,
	theme,
};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
	Frame,
	layout::Rect,
	widgets::{Clear, Paragraph},
};
use std::time::{Duration, Instant};

pub struct Dropdown {
	choices: Vec<&'static str>,
	cursor: usize,
	top: usize,
	area: Rect,
	prefix: String,
	typed_at: Option<Instant>,
	bar: Option<crate::components::scrollbar::Bar>,
	grab: Option<usize>,
}

impl Dropdown {
	fn page(&self) -> usize {
		self.area.height.saturating_sub(2).max(1) as usize
	}

	fn reveal(&mut self) {
		self.top = self
			.top
			.min(self.cursor)
			.max(self.cursor.saturating_sub(self.page() - 1));
		self.top = self.top.min(self.choices.len().saturating_sub(self.page()));
	}

	fn navigate(&mut self, direction: Direction) {
		self.cursor =
			controls::navigate(self.cursor, self.choices.len(), 1, self.page(), direction);
		self.prefix.clear();
		self.reveal();
	}

	fn type_prefix(&mut self, text: &str) {
		let now = Instant::now();
		if self
			.typed_at
			.is_none_or(|at| now.duration_since(at) > Duration::from_secs(1))
		{
			self.prefix.clear();
		}
		self.typed_at = Some(now);
		self.prefix.extend(
			text.chars()
				.filter(|c| !c.is_control())
				.flat_map(char::to_lowercase),
		);
		if let Some(index) = self
			.choices
			.iter()
			.position(|choice| choice.to_lowercase().starts_with(&self.prefix))
		{
			self.cursor = index;
			self.reveal();
		}
	}
}

impl Characters {
	pub fn open_dropdown(&mut self) {
		let choices = CATALOG.groups.iter().map(|group| group.name).collect();
		self.dropdown = Some(Dropdown {
			choices,
			cursor: self.group,
			top: 0,
			area: Rect::default(),
			prefix: String::new(),
			typed_at: None,
			bar: None,
			grab: None,
		});
	}

	fn commit_dropdown(&mut self) {
		let Some(dropdown) = self.dropdown.take() else {
			return;
		};
		self.group = dropdown.cursor;
		self.refresh();
	}

	pub fn dropdown_text(&mut self, text: &str) -> bool {
		let Some(dropdown) = &mut self.dropdown else {
			return false;
		};
		dropdown.type_prefix(text);
		true
	}
}

impl Panel {
	pub fn overlay_action(&mut self, action: A) -> bool {
		let Some(dropdown) = &mut self.characters.dropdown else {
			return false;
		};
		match action {
			A::Close | A::Abort => self.characters.dropdown = None,
			A::Activate | A::Submit => self.characters.commit_dropdown(),
			A::FieldUp | A::FieldLeft | A::PreviousField => dropdown.navigate(Direction::Previous),
			A::FieldDown | A::FieldRight | A::NextField => dropdown.navigate(Direction::Next),
			A::FieldHome => dropdown.navigate(Direction::Beginning),
			A::FieldEnd => dropdown.navigate(Direction::End),
			A::FieldPageUp => dropdown.navigate(Direction::PageUp),
			A::FieldPageDown => dropdown.navigate(Direction::PageDown),
			_ => (),
		}
		true
	}

	pub fn overlay_mouse(&mut self, mouse: MouseEvent) -> bool {
		let Some(dropdown) = &mut self.characters.dropdown else {
			return false;
		};
		let point = (mouse.column, mouse.row).into();
		match mouse.kind {
			MouseEventKind::Up(MouseButton::Left) => dropdown.grab = None,
			MouseEventKind::ScrollUp => dropdown.navigate(Direction::Up),
			MouseEventKind::ScrollDown => dropdown.navigate(Direction::Down),
			MouseEventKind::Down(MouseButton::Left) => {
				if let Some(bar) = dropdown.bar.filter(|bar| bar.area.contains(point)) {
					let at = bar.coordinate(mouse.column, mouse.row);
					let grab = bar.grab_at(at);
					dropdown.grab = Some(grab);
					dropdown.top = bar.offset_at(at, grab);
				} else if dropdown.area.contains(point) {
					let row = mouse.row.saturating_sub(dropdown.area.y + 1) as usize;
					if mouse.row > dropdown.area.y
						&& row < dropdown.page()
						&& dropdown.top + row < dropdown.choices.len()
					{
						dropdown.cursor = dropdown.top + row;
						self.characters.commit_dropdown();
					}
				} else {
					self.characters.dropdown = None;
				}
			}
			MouseEventKind::Drag(MouseButton::Left) => {
				if let (Some(bar), Some(grab)) = (dropdown.bar, dropdown.grab) {
					dropdown.top = bar.offset_at(bar.coordinate(mouse.column, mouse.row), grab);
				}
			}
			_ => (),
		}
		true
	}

	pub(in crate::panel) fn draw_character_dropdown(&mut self, frame: &mut Frame, area: Rect) {
		if self.characters.dropdown.is_none() {
			return;
		}
		let control = Control::CharacterGroup;
		let items = self.items();
		let anchor = self
			.tool_areas
			.iter()
			.find(|(_, index)| {
				items
					.get(*index)
					.is_some_and(|item| item.control == control)
			})
			.map(|(rect, _)| *rect)
			.unwrap_or(area);
		let dropdown = self.characters.dropdown.as_mut().unwrap();
		let height = (dropdown.choices.len() + 2)
			.min(14)
			.min(area.height as usize) as u16;
		let y = if area.bottom().saturating_sub(anchor.bottom()) >= height {
			anchor.bottom()
		} else {
			anchor
				.y
				.saturating_sub(height)
				.max(area.y)
				.min(area.bottom().saturating_sub(height))
		};
		let screen = frame.area();
		let longest = dropdown
			.choices
			.iter()
			.map(|label| label.len())
			.max()
			.unwrap_or(0);
		let width = (longest.saturating_add(3).min(u16::MAX as usize) as u16)
			.max(area.width.saturating_sub(2))
			.min(screen.width.saturating_sub(2));
		let x = anchor
			.x
			.min(screen.right().saturating_sub(width + 1))
			.max(screen.x + 1);
		let bounds = Rect::new(x, y, width, height);
		if dropdown.area != bounds {
			dropdown.area = bounds;
			dropdown.reveal();
		}
		frame.render_widget(Clear, bounds);
		let block = theme::block().title(" Group ");
		let inner = block.inner(bounds);
		frame.render_widget(block, bounds);
		for (row, label) in dropdown
			.choices
			.iter()
			.enumerate()
			.skip(dropdown.top)
			.take(inner.height as usize)
		{
			let style = if row == dropdown.cursor {
				theme::muted()
					.bg(theme::selected_color())
					.fg(theme::selected_text_color())
			} else {
				theme::muted().bg(theme::surface_color())
			};
			frame.render_widget(
				Paragraph::new(*label).style(style),
				Rect::new(
					inner.x,
					inner.y + (row - dropdown.top) as u16,
					inner.width.saturating_sub(1),
					1,
				),
			);
		}
		let mut bars = Vec::new();
		crate::components::scrollbar::draw(
			frame,
			&mut bars,
			crate::components::scrollbar::Bar {
				target: crate::components::scrollbar::Axis::Vertical,
				area: Rect::new(inner.right().saturating_sub(1), inner.y, 1, inner.height),
				content: dropdown.choices.len(),
				viewport: inner.height as usize,
				offset: dropdown.top,
			},
		);
		dropdown.bar = bars.pop();
	}
}
