//! Key adapter and painting for semantic fields. Input filtering stays with the caller.
pub use crate::libraries::controls::{Field, FieldAction};
use crate::{shortcuts::ActionId as A, theme};
use ratatui::{
	Frame,
	layout::Rect,
	style::{Modifier, Style},
	widgets::Paragraph,
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
	let before = crate::libraries::text::display::sanitize(&field.text()[..field.cursor()]);
	let text = format!(
		"{}{}{}",
		before,
		if focused { "▏" } else { "" },
		crate::libraries::text::display::sanitize(&field.text()[field.cursor()..])
	);
	let skip = if focused {
		before
			.width()
			.saturating_sub(area.width.saturating_sub(1) as usize)
	} else {
		0
	};
	frame.render_widget(
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
			),
		area,
	);
}
