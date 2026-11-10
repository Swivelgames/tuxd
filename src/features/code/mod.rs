//! Read-only Code projection, viewport, and copy coordination.
mod highlight;
pub mod projection;
pub mod shortcuts;

use super::clipboard_notice;
use crate::libraries::{
	ansi::{Document, Position},
	clipboard::{Service, WriteRequest},
	text::Layout,
};
use std::io::Write;

/// Changes that should resume following after the user manually scrolls Code.
#[derive(PartialEq, Eq)]
struct FollowTarget {
	line: usize,
	visual_row: usize,
	rows: usize,
	width: usize,
	height: usize,
	wrap: bool,
}

pub struct Code {
	pub surface: crate::components::text_surface::Surface,
	copy: Option<WriteRequest>,
	follow_target: Option<FollowTarget>,
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
		let page = self.surface.text.height as usize;
		let viewport = &mut self.surface.viewport;
		match action {
			A::CodeUp => viewport.top = viewport.top.saturating_sub(1),
			A::CodeDown => viewport.top = viewport.top.saturating_add(1),
			A::CodeLeft => viewport.left = viewport.left.saturating_sub(4),
			A::CodeRight => viewport.left = viewport.left.saturating_add(4),
			A::CodePageUp => viewport.top = viewport.top.saturating_sub(page),
			A::CodePageDown => viewport.top = viewport.top.saturating_add(page),
			A::CodeHome => viewport.left = 0,
			A::CodeEnd => viewport.left = usize::MAX,
			A::CodeStart => {
				viewport.top = 0;
				viewport.left = 0;
			}
			A::CodeFinish => {
				viewport.top = usize::MAX;
				viewport.left = usize::MAX;
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
		let active_line = projection.active_line(active_source);
		self.follow_active_line(&layout, active_line);
		self.surface.viewport.clamp(&layout, false);

		self.surface.draw(
			frame,
			border,
			inner,
			text,
			Content {
				lines: &projection.lines,
				layout: &layout,
				active_line,
				cursor_space: false,
				gutter_style: gutter_style(),
			},
			|position| projection.style_at(position),
		);
	}

	fn follow_active_line(&mut self, layout: &Layout, active_line: Option<usize>) {
		let view = &mut self.surface.viewport;
		let Some(line) = active_line.filter(|_| view.width > 0 && view.height > 0) else {
			return;
		};

		let visual_row = layout.location(Position { row: line, byte: 0 }).0;
		let target = FollowTarget {
			line,
			visual_row,
			rows: layout.rows.len(),
			width: view.width,
			height: view.height,
			wrap: view.wrap,
		};
		if self.follow_target.as_ref() == Some(&target) {
			return;
		}

		view.top = visual_row.saturating_sub(view.height / 2);
		self.follow_target = Some(target);
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

impl Default for Code {
	fn default() -> Self {
		let mut surface = crate::components::text_surface::Surface::default();
		surface.viewport.wrap = true;
		Self {
			surface,
			copy: None,
			follow_target: None,
		}
	}
}

#[cfg(test)]
mod tests;
