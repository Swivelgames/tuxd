//! Key adapter and painting for semantic fields. Input filtering stays with the caller.
pub use crate::libraries::controls::{Field, FieldAction};
use crate::{shortcuts::ActionId as A, theme};
use ratatui::{
	Frame,
	layout::Rect,
	style::{Modifier, Style},
	widgets::{Paragraph, Widget},
};
use unicode_width::UnicodeWidthStr;

pub fn action(action: A) -> Option<FieldAction> {
	Some(match action {
		A::SelectAll => FieldAction::SelectAll,
		A::FieldHome => FieldAction::Beginning,
		A::FieldEnd => FieldAction::End,
		A::FieldLeft => FieldAction::Left,
		A::FieldRight => FieldAction::Right,
		A::FieldDeleteBefore => FieldAction::Backspace,
		A::FieldDeleteAfter => FieldAction::Delete,
		_ => return None,
	})
}
pub fn draw(field: &Field, frame: &mut Frame, area: Rect, focused: bool) {
	draw_buffer(field, frame.buffer_mut(), area, focused);
}

pub fn draw_buffer(field: &Field, buffer: &mut ratatui::buffer::Buffer, area: Rect, focused: bool) {
	let before = display_text(&field.text()[..field.cursor()]);
	let text = format!(
		"{}{}{}",
		before,
		if focused { "▏" } else { "" },
		display_text(&field.text()[field.cursor()..])
	);
	let skip = if focused {
		before
			.width()
			.saturating_sub(area.width.saturating_sub(1) as usize)
	} else {
		0
	};
	Paragraph::new(text)
		.scroll((0, skip.min(u16::MAX as usize) as u16))
		.style(
			Style::default()
				.bg(if focused {
					theme::selected_color()
				} else {
					theme::field_color()
				})
				.add_modifier(if focused && field.selected() {
					Modifier::REVERSED
				} else {
					Modifier::empty()
				}),
		)
		.render(area, buffer);
}

/// Fields accept literal search/path text; display fallback must not change it.
fn display_text(text: &str) -> String {
	use crate::libraries::text::display::{display_grapheme, glyphs, sanitize};
	let sanitized = sanitize(text);
	glyphs(&sanitized)
		.map(|glyph| display_grapheme(glyph.text))
		.collect()
}

#[cfg(test)]
mod tests {
	use super::display_text;
	#[test]
	fn fields_use_the_same_safe_grapheme_projection_as_document_views() {
		assert_eq!(display_text("café 界 ─"), "café 界 ─");
		assert_eq!(display_text("flag 🇺🇳"), "flag \\u{1f1fa}\\u{1f1f3}");
	}
}
