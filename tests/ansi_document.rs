use terminal_ux_designer::libraries::ansi::*;

fn position(byte: usize) -> Position {
	Position { row: 0, byte }
}

fn assert_valid(document: &Document) {
	assert_eq!(Document::try_from(document.to_data()).unwrap(), *document);
	for (source, mode) in [
		(document.to_raw_ansi(), Mode::Raw),
		(document.to_escaped_ansi(), Mode::Escaped),
	] {
		let parsed = parse(&source, mode).unwrap();
		assert!(parsed.issues.is_empty());
		assert_eq!(parsed.document, *document);
	}
}

#[test]
fn invalid_edits_leave_the_document_unchanged() {
	let original = Document::from_text("e\u{301}界\nend", TextStyle::default()).unwrap();
	let fragment = Document::from_text("new", TextStyle::default()).unwrap();
	let mut document = original.clone();
	for invalid in [
		position(1),
		position(2),
		position(4),
		position(100),
		Position { row: 99, byte: 0 },
	] {
		assert_eq!(document.insert(invalid, "x"), Err(Error::InvalidPosition));
		assert_eq!(
			document.replace(position(0)..invalid, &fragment),
			Err(Error::InvalidPosition)
		);
		assert_eq!(
			document.remove(invalid, document.end()),
			Err(Error::InvalidPosition)
		);
		assert!(document.extract(position(0)..invalid).is_err());
		assert!(
			document
				.apply_style(position(0)..invalid, Action::Bold, true)
				.is_err()
		);
		assert_eq!(document, original);
	}
	assert_eq!(
		document.replace(document.end()..position(0), &fragment),
		Err(Error::ReversedRange)
	);
	assert_eq!(
		document.insert(position(0), "\x1b[31m"),
		Err(Error::UnsupportedControl)
	);
	assert_eq!(
		document.apply_style(
			position(0)..document.end(),
			Action::Foreground(Ink::Ansi(16)),
			true
		),
		Err(Error::InvalidColor)
	);
	assert_eq!(
		document.set_style(
			position(0)..document.end(),
			TextStyle {
				background: Ink::Ansi(255),
				..Default::default()
			}
		),
		Err(Error::InvalidColor)
	);
	assert_eq!(document, original);
}

#[test]
fn insertion_and_deletion_normalize_joined_graphemes_without_caller_repair() {
	let bold = TextStyle {
		bold: true,
		..Default::default()
	};
	let italic = TextStyle {
		italic: true,
		..Default::default()
	};
	let mut document = Document::from_text("e", bold).unwrap();
	let accent = Document::from_text("\u{301}", italic).unwrap();
	let cursor = document
		.replace(document.end()..document.end(), &accent)
		.unwrap();
	assert_eq!(document.text(), "e\u{301}");
	assert_eq!(cursor, document.end());
	assert_eq!(
		document.style_runs(),
		&[Run {
			start: 0,
			end: 3,
			style: bold
		}]
	);
	assert_valid(&document);

	let mut document = Document::from_text("e\n\u{301}", bold).unwrap();
	document
		.set_style(Position { row: 1, byte: 0 }..document.end(), italic)
		.unwrap();
	let cursor = document
		.remove(position(1), Position { row: 1, byte: 0 })
		.unwrap();
	assert_eq!(cursor, position(0));
	assert_eq!(
		document.style_runs(),
		&[Run {
			start: 0,
			end: 3,
			style: bold
		}]
	);
	assert_valid(&document);
}

#[test]
fn replacement_does_not_expose_an_intermediate_grapheme_join() {
	// Removing the newline alone would join the accent to e. Replacing it with X
	// must instead produce e followed by X+accent, inheriting X's fragment style.
	let mut document = Document::from_text("e\n\u{301}", TextStyle::default()).unwrap();
	let bold = TextStyle {
		bold: true,
		..Default::default()
	};
	let fragment = Document::from_text("X", bold).unwrap();
	let cursor = document
		.replace(position(1)..Position { row: 1, byte: 0 }, &fragment)
		.unwrap();
	assert_eq!(document.text(), "eX\u{301}");
	assert_eq!(cursor, document.end());
	assert_eq!(document.style_at(position(1)), bold);
	assert_valid(&document);
}

#[test]
fn overlapping_styles_and_replacements_keep_canonical_ranges() {
	let mut document = Document::from_text("0123456789", TextStyle::default()).unwrap();
	document
		.apply_style(position(2)..position(8), Action::Bold, true)
		.unwrap();
	document
		.apply_style(position(4)..position(10), Action::Italic, true)
		.unwrap();
	assert!(document.style_at(position(5)).bold && document.style_at(position(5)).italic);
	assert_eq!(
		document
			.style_state(position(0)..position(10), Action::Bold)
			.unwrap(),
		None
	);
	document.remove(position(1), position(9)).unwrap();
	assert_eq!(document.text(), "09");
	document
		.apply_style(position(0)..document.end(), Action::Clear, true)
		.unwrap();
	assert_eq!(document.style_runs().len(), 1);

	let fragment = Document::from_text(
		"abc",
		TextStyle {
			underline: true,
			..Default::default()
		},
	)
	.unwrap();
	document
		.replace(position(1)..position(1), &fragment)
		.unwrap();
	assert_eq!(
		document
			.style_runs()
			.iter()
			.map(|run| (run.start, run.end))
			.collect::<Vec<_>>(),
		[(0, 1), (1, 4), (4, 5)]
	);
	assert_eq!(
		document.extract(position(1)..position(4)).unwrap(),
		fragment
	);
	assert_valid(&document);
}

#[test]
fn raw_and_escaped_encoders_preserve_exact_output_and_empty_resets() {
	let mut document = Document::from_text("Hi\n\t\\n", TextStyle::default()).unwrap();
	document
		.apply_style(position(0)..position(2), Action::Bold, true)
		.unwrap();
	document
		.apply_style(
			position(0)..position(2),
			Action::Foreground(Ink::Ansi(6)),
			true,
		)
		.unwrap();
	assert_eq!(
		document.to_raw_ansi(),
		"\x1b[0m\x1b[0;1;36mHi\x1b[0m\n\t\\n\x1b[0m"
	);
	assert_eq!(
		document.to_escaped_ansi(),
		"\\x1b[0m\\x1b[0;1;36mHi\\x1b[0m\\n\\t\\\\n\\x1b[0m"
	);
	assert!(!document.to_escaped_ansi().contains('\n'));
	assert_eq!(Document::default().to_raw_ansi(), "\x1b[0m\x1b[0m");
	assert_valid(&document);
}

#[test]
fn structured_data_validates_before_it_becomes_a_document() {
	let document = Document::from_text("e\u{301}界\n", TextStyle::default()).unwrap();
	let data = document.to_data();
	let json = serde_json::to_string(&data).unwrap();
	let decoded: DocumentData = serde_json::from_str(&json).unwrap();
	assert_eq!(Document::try_from(decoded).unwrap(), document);
	for invalid in [
		DocumentData {
			text: data.text.clone(),
			styles: Styles::default(),
		},
		DocumentData {
			text: "\x1b".into(),
			styles: Styles::default(),
		},
		DocumentData {
			text: data.text.clone(),
			styles: Styles {
				runs: vec![Run {
					start: 0,
					end: 1,
					style: TextStyle::default(),
				}],
			},
		},
		DocumentData {
			text: "x".into(),
			styles: Styles {
				runs: vec![Run {
					start: 0,
					end: 1,
					style: TextStyle {
						foreground: Ink::Ansi(16),
						..Default::default()
					},
				}],
			},
		},
	] {
		assert!(Document::try_from(invalid).is_err());
	}
	// Mutating a DTO never mutates the source document.
	let mut detached = document.to_data();
	detached.text.clear();
	assert_eq!(document.to_data(), data);
}

#[test]
fn deterministic_public_operations_preserve_invariants_after_every_step() {
	let mut document = Document::default();
	let mut seed = 17u64;
	for _ in 0..300 {
		seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
		let mut positions = vec![Position::default()];
		while *positions.last().unwrap() != document.end() {
			positions.push(document.next(*positions.last().unwrap()));
		}
		let a = positions[(seed as usize >> 8) % positions.len()];
		let b = positions[(seed as usize >> 16) % positions.len()];
		let range = a.min(b)..a.max(b);
		match seed % 4 {
			0 => {
				let text =
					["e\u{301}", "界", "\n", "\u{200d}", "👩🏽‍💻", "\t"][(seed as usize >> 24) % 6];
				let fragment = Document::from_text(
					text,
					TextStyle {
						bold: seed & 32 != 0,
						..Default::default()
					},
				)
				.unwrap();
				document.replace(range, &fragment).unwrap();
			}
			1 => {
				document
					.apply_style(range, Action::Italic, seed & 32 != 0)
					.unwrap();
			}
			2 => {
				assert_valid(&document.extract(range).unwrap());
			}
			_ => {
				document.remove(range.start, range.end).unwrap();
			}
		}
		assert_valid(&document);
	}
}

#[test]
fn word_boundaries_skip_whitespace_cross_lines_and_keep_graphemes() {
	let mut doc = Document::default();
	doc.insert(Position::default(), "one  café!\n e\u{301}")
		.unwrap();
	let after_one = doc.word_target(Position::default(), false);
	assert_eq!(after_one, Position { row: 0, byte: 3 });
	let after_cafe = doc.word_target(after_one, false);
	assert_eq!(after_cafe, Position { row: 0, byte: 10 });
	let punctuation = doc.word_target(after_cafe, false);
	assert_eq!(punctuation, Position { row: 0, byte: 11 });
	assert_eq!(
		doc.word_target(punctuation, false),
		Position { row: 1, byte: 4 }
	);
	assert_eq!(
		doc.word_target(Position { row: 1, byte: 1 }, true),
		after_cafe
	);
	assert_eq!(
		doc.word_target(Position { row: 1, byte: 4 }, true),
		Position { row: 1, byte: 1 }
	);
}
#[test]
fn multiline_insert_remove_and_trailing_newline() {
	let mut doc = Document::default();
	let end = doc.insert(Position::default(), "headTAIL").unwrap();
	assert_eq!(end.byte, 8);
	let end = doc.insert(Position { row: 0, byte: 4 }, "a\nb\n").unwrap();
	assert_eq!(doc.lines(), ["heada", "b", "TAIL"]);
	assert_eq!(end, Position { row: 2, byte: 0 });
	doc.remove(Position { row: 0, byte: 4 }, end).unwrap();
	assert_eq!(doc.lines(), ["headTAIL"]);
}
#[test]
fn navigation_uses_whole_graphemes_and_crosses_newlines() {
	let mut doc = Document::default();
	let end = doc.insert(Position::default(), "e\u{301}界👩‍💻\n").unwrap();
	let newline = doc.previous(end);
	assert_eq!(
		newline,
		Position {
			row: 0,
			byte: "e\u{301}界👩‍💻".len()
		}
	);
	let emoji = doc.previous(newline);
	assert_eq!(emoji.byte, "e\u{301}界".len());
	assert_eq!(doc.next(emoji), newline);
	assert_eq!(doc.next(newline), end);
	let wide = doc.previous(emoji);
	assert_eq!(wide.byte, "e\u{301}".len());
	assert_eq!(doc.previous(wide), Position::default());
}
#[test]
fn editing_can_merge_graphemes_without_leaving_invalid_cursor() {
	let mut doc = Document::default();
	doc.insert(Position::default(), "👩💻").unwrap();
	let cursor = doc
		.insert(
			Position {
				row: 0,
				byte: "👩".len(),
			},
			"\u{200d}",
		)
		.unwrap();
	assert_eq!(cursor.byte, "👩‍💻".len());
	assert_eq!(doc.previous(cursor), Position::default());
	doc = Document::default();
	doc.insert(Position::default(), "e\n\u{301}").unwrap();
	let cursor = doc
		.remove(Position { row: 0, byte: 1 }, Position { row: 1, byte: 0 })
		.unwrap();
	assert_eq!(doc.lines(), ["e\u{301}"]);
	assert_eq!(cursor, Position::default());
}
