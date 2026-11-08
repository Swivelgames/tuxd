//! Coordinates editing commands with the editor's own text viewport.
mod actions;
pub mod clipboard;
pub mod shortcuts;
use crate::components::text_surface::Surface;
use crate::libraries::ansi::Document;
use crate::libraries::editing::{EditKind, Editing, Movement, Navigation};
use crate::libraries::text::Layout;
use std::time::Instant;

#[derive(Default)]
pub struct Editor {
	pub surface: Surface,
}
impl Editor {
	pub fn draw(
		&mut self,
		frame: &mut ratatui::Frame,
		border: ratatui::layout::Rect,
		inner: ratatui::layout::Rect,
		editing: &Editing,
		focused: bool,
	) {
		use crate::components::text_surface::{Content, gutter_style, view::text_area};
		use ratatui::{
			style::{Color, Style},
			widgets::Block,
		};
		let text = text_area(inner, editing.document.lines().len());
		self.resize(editing, text.width as usize, text.height as usize);
		frame.render_widget(
			Block::default().style(Style::default().fg(Color::Reset).bg(Color::Reset)),
			text,
		);
		let layout = self.layout(editing);
		self.surface.draw(
			frame,
			border,
			inner,
			text,
			Content {
				lines: editing.document.lines(),
				layout: &layout,
				active_line: Some(editing.cursor.row),
				cursor_space: true,
				gutter_style: gutter_style(),
			},
			|position| {
				let style =
					crate::adapters::ratatui::text_style(editing.document.style_at(position));
				if editing.selected(position) {
					style.bg(crate::theme::selection_overlay_color())
				} else {
					style
				}
			},
		);
		if focused {
			self.surface.cursor(frame, &layout, editing.cursor);
		}
	}

	pub fn layout(&self, editing: &Editing) -> Layout {
		self.surface.viewport.layout(editing.document.lines(), true)
	}
	pub fn ensure_visible(&mut self, editing: &Editing) {
		let layout = self.layout(editing);
		self.surface.viewport.reveal(&layout, editing.cursor);
	}
	pub fn resize(&mut self, editing: &Editing, width: usize, height: usize) {
		let view = &mut self.surface.viewport;
		if (view.width, view.height) != (width, height) {
			view.width = width;
			view.height = height;
			self.ensure_visible(editing);
		}
	}
	pub fn toggle_wrap(&mut self, editing: &mut Editing) {
		self.surface.viewport.toggle_wrap();
		editing.reset_movement();
		self.ensure_visible(editing);
	}
	pub fn navigate(&mut self, editing: &mut Editing, movement: Movement, selecting: bool) {
		let layout = self.layout(editing);
		let view = &self.surface.viewport;
		editing.navigate(
			movement,
			selecting,
			Navigation {
				layout: &layout,
				wrap: view.wrap,
				height: view.height,
			},
		);
		self.ensure_visible(editing);
	}
	#[cfg(test)]
	pub fn move_cursor(&mut self, editing: &mut Editing, movement: Movement) {
		let layout = self.layout(editing);
		let view = &self.surface.viewport;
		editing.move_cursor(
			movement,
			Navigation {
				layout: &layout,
				wrap: view.wrap,
				height: view.height,
			},
		);
		self.ensure_visible(editing);
	}
	pub fn click(&mut self, editing: &mut Editing, column: usize, row: usize) {
		let position = self
			.surface
			.viewport
			.hit(&self.layout(editing), column, row);
		editing.place_cursor(position);
		self.ensure_visible(editing);
	}
	pub fn scroll(&mut self, editing: &mut Editing, down: bool) {
		editing.break_group();
		let layout = self.layout(editing);
		self.surface.viewport.scroll(down, 3, &layout);
	}
	pub fn insert(&mut self, editing: &mut Editing, text: &str, kind: EditKind, now: Instant) {
		editing.insert(text, kind, now);
		if !text.is_empty() {
			self.ensure_visible(editing);
		}
	}
	pub fn paste_document(&mut self, editing: &mut Editing, document: &Document, formatted: bool) {
		editing.paste_document(document, formatted);
		if !document.is_empty() {
			self.ensure_visible(editing);
		}
	}
	pub fn delete(&mut self, editing: &mut Editing, backwards: bool, now: Instant) {
		let revision = editing.revision;
		editing.delete(backwards, now);
		if revision != editing.revision {
			self.ensure_visible(editing);
		}
	}
	pub fn delete_word(&mut self, editing: &mut Editing, now: Instant) {
		let revision = editing.revision;
		editing.delete_word(now);
		if revision != editing.revision {
			self.ensure_visible(editing);
		}
	}
	pub fn undo(&mut self, editing: &mut Editing, redo: bool) {
		let revision = editing.revision;
		editing.undo(redo);
		if revision != editing.revision {
			self.ensure_visible(editing);
		}
	}
}

#[cfg(test)]
mod tests;
