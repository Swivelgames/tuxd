//! Geometry for the shared workspace frame and its pane dividers.
use ratatui::layout::Rect;

pub(super) struct Layout {
	pub editor: Rect,
	pub code: Rect,
	pub tools: Option<Rect>,
	pub status: Rect,
}

impl Layout {
	pub fn new(area: Rect, editor_open: bool, code_open: bool, slim_tool: bool) -> Self {
		let tools_width = if area.width > 64 { 35 } else { 0 };
		let left_width = area.width - tools_width;
		let code_height = if !code_open {
			2
		} else if !editor_open {
			area.height.saturating_sub(2)
		} else if slim_tool {
			(area.height / 2).max(8).min(area.height.saturating_sub(3))
		} else {
			(area.height / 3).max(4).min(area.height.saturating_sub(3))
		};
		let editor = Rect::new(area.x, area.y, left_width, area.height - code_height);
		let code = Rect::new(area.x, editor.bottom(), left_width, code_height);
		let status = Rect::new(area.x + 1, editor.bottom() - 1, left_width - 2, 1);
		let tools = (tools_width > 0).then(|| {
			Rect::new(
				editor.right() - 1,
				area.y + 1,
				tools_width + 1,
				area.height - 2,
			)
		});
		Self {
			editor,
			code,
			tools,
			status,
		}
	}
}

/// Leave room inside the frame for the vertical scrollbar track.
pub(super) fn surface_rects(bounds: Rect) -> (Rect, Rect) {
	let border = Rect::new(
		bounds.x,
		bounds.y,
		bounds.width.saturating_sub(1),
		bounds.height.saturating_sub(1),
	);
	let inner = Rect::new(
		bounds.x + 1,
		bounds.y + 1,
		bounds.width.saturating_sub(3),
		bounds.height.saturating_sub(2),
	);
	(border, inner)
}
