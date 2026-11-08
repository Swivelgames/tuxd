use super::*;
use crate::{
	libraries::{
		ansi::TextStyle,
		clipboard::{Error, Receipt, WriteOutput},
	},
	test_clipboard::{controlled, until},
};

#[test]
fn copy_owns_exact_payload_and_receipt_independently_of_document_changes() {
	for receipt in [Receipt::Desktop, Receipt::Tmux, Receipt::Osc52] {
		let (service, control) = controlled();
		let mut code = Code::default();
		let mut document = Document::from_text(
			"a\n界\\x1b[31m",
			TextStyle {
				bold: true,
				..Default::default()
			},
		)
		.unwrap();
		let expected = document.to_escaped_ansi();
		assert_eq!(
			code.copy(&document, &service).as_deref(),
			Some("Copying escaped ANSI…")
		);
		assert_eq!(
			control
				.payloads
				.recv_timeout(std::time::Duration::from_secs(1))
				.unwrap(),
			expected
		);
		assert!(code.copy(&document, &service).is_none());
		document = Document::default();
		assert!(document.is_empty());
		control
			.writes
			.send(Ok(WriteOutput::Delivered(receipt)))
			.unwrap();
		let mut notice = None;
		until(|| {
			notice = code.poll_copy(&mut Vec::new());
			notice.is_some()
		});
		assert_eq!(
			notice.unwrap(),
			clipboard_notice::copied(Ok(receipt), "escaped ANSI")
		);
		assert!(code.poll_copy(&mut Vec::new()).is_none());
	}
}

#[test]
fn failure_stays_local_and_destroyed_owner_cannot_deliver_to_a_new_code_instance() {
	let (service, control) = controlled();
	let mut code = Code::default();
	code.copy(&Document::default(), &service);
	control.writes.send(Err(Error::TmuxWrite)).unwrap();
	let mut notice = None;
	until(|| {
		notice = code.poll_copy(&mut Vec::new());
		notice.is_some()
	});
	assert!(notice.unwrap().contains("Copy failed: tmux"));

	code.copy(&Document::default(), &service);
	drop(code);
	let mut replacement = Code::default();
	control
		.writes
		.send(Ok(WriteOutput::Terminal(b"discarded".to_vec())))
		.unwrap();
	let mut output = Vec::new();
	assert!(replacement.poll_copy(&mut output).is_none());
	assert!(output.is_empty());
}

#[test]
fn readonly_navigation_and_replacement_rendering_do_not_retain_old_content() {
	use crate::shortcuts::ActionId as A;
	use ratatui::{Terminal, backend::TestBackend, layout::Rect};
	let first = Document::from_text(&"old content\n".repeat(30), Default::default()).unwrap();
	let replacement = Document::from_text("new", Default::default()).unwrap();
	let original = first.clone();
	let mut code = Code::default();
	let mut terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
	let draw = |code: &mut Code, terminal: &mut Terminal<TestBackend>, document: &Document| {
		terminal
			.draw(|frame| {
				code.draw(
					frame,
					Rect::new(0, 0, 40, 12),
					Rect::new(1, 1, 38, 10),
					document,
					0,
				)
			})
			.unwrap();
	};
	draw(&mut code, &mut terminal, &first);
	code.navigate(A::CodeFinish);
	draw(&mut code, &mut terminal, &first);
	assert!(code.surface.viewport.top > 0);
	// Directly invoking an editing action on Code has no mutation path.
	code.navigate(A::DeleteAfter);
	code.navigate(A::InsertNewline);
	assert_eq!(first, original);
	draw(&mut code, &mut terminal, &replacement);
	assert_eq!(code.surface.viewport.top, 0);
	let visible: String = terminal
		.backend()
		.buffer()
		.content
		.iter()
		.map(|cell| cell.symbol())
		.collect();
	assert!(visible.contains("new"));
	assert!(!visible.contains("old content"));
	assert_eq!(replacement.text(), "new");
}
