//! Read-only Code projection, viewport, and copy coordination.
mod highlight;
pub mod projection;
pub mod shortcuts;
use super::clipboard_notice;
use crate::libraries::{
	ansi::Document,
	clipboard::{Service, WriteRequest},
};
use std::io::Write;

pub struct Code {
	pub surface: crate::components::text_surface::Surface,
	copy: Option<WriteRequest>,
}
impl Code {
	pub fn handle_action(
		&mut self,
		action: crate::shortcuts::ActionId,
		document: &Document,
		clipboard: &Service,
	) -> Option<String> {
		use crate::shortcuts::ActionId as A;
		match action {
			A::Copy => return self.copy(document, clipboard),
			A::CodeWrap => self.surface.viewport.toggle_wrap(),
			_ => self.navigate(action),
		}
		None
	}
	pub fn navigate(&mut self, action: crate::shortcuts::ActionId) {
		use crate::shortcuts::ActionId as A;
		match action {
			A::CodeUp => self.surface.viewport.top = self.surface.viewport.top.saturating_sub(1),
			A::CodeDown => self.surface.viewport.top = self.surface.viewport.top.saturating_add(1),
			A::CodeLeft => {
				self.surface.viewport.left = self.surface.viewport.left.saturating_sub(4)
			}
			A::CodeRight => {
				self.surface.viewport.left = self.surface.viewport.left.saturating_add(4)
			}
			A::CodePageUp => {
				self.surface.viewport.top = self
					.surface
					.viewport
					.top
					.saturating_sub(self.surface.text.height as usize)
			}
			A::CodePageDown => {
				self.surface.viewport.top = self
					.surface
					.viewport
					.top
					.saturating_add(self.surface.text.height as usize)
			}
			A::CodeHome => self.surface.viewport.left = 0,
			A::CodeEnd => self.surface.viewport.left = usize::MAX,
			A::CodeStart => {
				self.surface.viewport.top = 0;
				self.surface.viewport.left = 0;
			}
			A::CodeFinish => {
				self.surface.viewport.top = usize::MAX;
				self.surface.viewport.left = usize::MAX;
			}
			_ => (),
		}
	}

	pub fn draw(
		&mut self,
		frame: &mut ratatui::Frame,
		border: ratatui::layout::Rect,
		inner: ratatui::layout::Rect,
		document: &Document,
		active_source: usize,
	) {
		use crate::components::text_surface::{Content, gutter_style, view::text_area};
		// Rebuilt from authoritative content: replacement, edits, and appearance cannot
		// leave stale derived rows. Layout is rebuilt for the current width/wrapping.
		let projection = projection::Projection::new(document);
		let text = text_area(inner, projection.lines.len());
		let view = &mut self.surface.viewport;
		view.width = text.width as usize;
		view.height = text.height as usize;
		let layout = view.layout(&projection.lines, false);
		view.clamp(&layout, false);
		self.surface.draw(
			frame,
			border,
			inner,
			text,
			Content {
				lines: &projection.lines,
				layout: &layout,
				active_line: projection.active_line(active_source),
				cursor_space: false,
				gutter_style: gutter_style(),
			},
			|position| projection.style_at(position),
		);
	}

	pub fn copy(&mut self, document: &Document, clipboard: &Service) -> Option<String> {
		if self.copy.is_some() {
			return None;
		}
		self.copy = Some(clipboard.write(document.to_escaped_ansi())?);
		Some("Copying escaped ANSI…".into())
	}
	pub fn poll_copy(&mut self, output: &mut impl Write) -> Option<String> {
		let completion = self.copy.as_mut()?.poll(output)?;
		self.copy = None;
		Some(clipboard_notice::copied(completion.result, "escaped ANSI"))
	}
}

#[cfg(test)]
mod tests;

impl Default for Code {
	fn default() -> Self {
		let mut surface = crate::components::text_surface::Surface::default();
		surface.viewport.wrap = true;
		Self {
			surface,
			copy: None,
		}
	}
}
