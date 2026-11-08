use super::*;
use crate::libraries::ansi::Mode;
use ratatui::{Terminal, backend::TestBackend};
#[test]
fn independent_five_row_views_and_error_highlighting() {
	let import =
		crate::libraries::ansi::parse(&"before\\x1b[2J after\n".repeat(12), Mode::Escaped).unwrap();
	let mut review = Review::default();
	let mut terminal = Terminal::new(TestBackend::new(100, 32)).unwrap();
	terminal
		.draw(|frame| {
			review.draw(
				frame,
				&import,
				Presentation {
					title: "sample.txt",
					explanation: "The original file will be unaffected.",
					accept: "Import anyway",
				},
			)
		})
		.unwrap();
	assert_eq!(review.areas[0].height, 5);
	assert_eq!(review.areas[1].height, 7);
	assert!(review.error_bar.is_some() && !review.preview.bars.is_empty());
	assert!(
		terminal
			.backend()
			.buffer()
			.content
			.iter()
			.any(|cell| cell.fg == theme::error_color() && cell.modifier.contains(Modifier::BOLD))
	);
	review.focus = 0;
	review.shortcut(crate::shortcuts::ActionId::FieldEnd);
	terminal
		.draw(|frame| {
			review.draw(
				frame,
				&import,
				Presentation {
					title: "sample.txt",
					explanation: "The original file will be unaffected.",
					accept: "Import anyway",
				},
			)
		})
		.unwrap();
	assert!(review.preview.viewport.top > 0);
	assert_eq!(review.errors_top, 0);
}
