//! Read-only text mechanics. Editing policy is supplied by the owning feature.
pub mod view;
use crate::{
	components::scrollbar::{self, Axis, Bar},
	libraries::{
		ansi::Position,
		text::{Layout, Viewport},
	},
	theme,
};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{Frame, layout::Rect, style::Style};

pub struct Content<'a> {
	pub lines: &'a [String],
	pub layout: &'a Layout,
	pub active_line: Option<usize>,
	pub cursor_space: bool,
	pub gutter_style: Style,
}
#[derive(Default)]
pub struct Surface {
	pub viewport: Viewport,
	pub text: Rect,
	pub inner: Rect,
	pub bars: Vec<Bar<Axis>>,
	drag: Option<(Axis, usize)>,
}
impl Surface {
	pub fn hide(&mut self) {
		self.text = Rect::default();
		self.inner = Rect::default();
		self.bars.clear();
	}
	pub fn cancel_drag(&mut self) {
		self.drag = None;
	}
	pub fn layout_content(
		&self,
		mut inner: Rect,
		lines: &[String],
		cursor_space: bool,
	) -> (Rect, Rect, Layout) {
		let mut text = view::text_area(inner, lines.len());
		let width = text.width as usize;
		let layout = Layout::new(lines, width, self.viewport.wrap, cursor_space);
		if !self.viewport.wrap && width > 0 && layout.columns(cursor_space) > width {
			inner.height = inner.height.saturating_sub(1);
			text.height = inner.height;
		}
		(inner, text, layout)
	}

	pub fn draw(
		&mut self,
		frame: &mut Frame,
		border: Rect,
		inner: Rect,
		text: Rect,
		content: Content<'_>,
		style_at: impl Fn(Position) -> Style,
	) {
		self.inner = inner;
		self.text = text;
		self.bars.clear();
		view::draw_lines(
			frame,
			view::View {
				inner,
				text,
				lines: content.lines,
				layout: content.layout,
				top: self.viewport.top,
				left: self.viewport.left,
				active_line: content.active_line,
				gutter_style: content.gutter_style,
			},
			style_at,
		);
		for (axis, area, count, viewport, offset) in [
			(
				Axis::Vertical,
				Rect::new(border.right().saturating_sub(1), inner.y, 1, inner.height),
				content.layout.rows.len(),
				text.height as usize,
				self.viewport.top,
			),
			(
				Axis::Horizontal,
				Rect::new(text.x, border.bottom().saturating_sub(1), text.width, 1),
				content.layout.columns(content.cursor_space),
				text.width as usize,
				self.viewport.left,
			),
		] {
			if axis == Axis::Horizontal && self.viewport.wrap {
				continue;
			}
			scrollbar::draw(
				frame,
				&mut self.bars,
				Bar {
					target: axis,
					area,
					content: count,
					viewport,
					offset,
				},
			);
		}
	}
	pub fn cursor(&self, frame: &mut Frame, layout: &Layout, position: Position) {
		let (row, x) = layout.location(position);
		let view = &self.viewport;
		if row >= view.top
			&& row < view.top + self.text.height as usize
			&& x >= view.left
			&& x < view.left + self.text.width as usize
		{
			frame.set_cursor_position((
				self.text.x + (x - view.left) as u16,
				self.text.y + (row - view.top) as u16,
			));
		}
	}
	fn offset(&mut self, axis: Axis, offset: usize) {
		match axis {
			Axis::Vertical => self.viewport.top = offset,
			Axis::Horizontal => self.viewport.left = offset,
		}
	}
	/// Only scrollbar events are consumed. Caret placement belongs to Editor.
	pub fn scrollbar_event(&mut self, mouse: MouseEvent) -> bool {
		if mouse.kind == MouseEventKind::Up(MouseButton::Left) {
			return self.drag.take().is_some();
		}
		if mouse.kind == MouseEventKind::Drag(MouseButton::Left)
			&& let Some((axis, grab)) = self.drag
		{
			if let Some(bar) = self.bars.iter().find(|bar| bar.target == axis).copied() {
				self.offset(
					axis,
					bar.offset_at(bar.coordinate(mouse.column, mouse.row), grab),
				);
			}
			return true;
		}
		let Some(bar) = self
			.bars
			.iter()
			.find(|bar| bar.area.contains((mouse.column, mouse.row).into()))
			.copied()
		else {
			return false;
		};
		match mouse.kind {
			MouseEventKind::Down(MouseButton::Left) => {
				let at = bar.coordinate(mouse.column, mouse.row);
				let (start, size) = bar.thumb();
				let on_thumb = (start..start + size).contains(&at);
				let grab = bar.grab_at(at);
				self.drag = Some((bar.target, grab));
				if !on_thumb {
					self.offset(bar.target, bar.offset_at(at, grab));
				}
			}
			MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
				let offset = if mouse.kind == MouseEventKind::ScrollDown {
					bar.offset
						.saturating_add(3)
						.min(bar.content.saturating_sub(bar.viewport))
				} else {
					bar.offset.saturating_sub(3)
				};
				self.offset(bar.target, offset);
			}
			_ => return false,
		}
		true
	}
}
pub fn gutter_style() -> Style {
	Style::default()
		.fg(theme::muted_color())
		.bg(theme::field_color())
}
