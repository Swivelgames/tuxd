use super::super::{clipboard::Clipboard, session::Session};
use crate::process::Runner;
use anyhow::{Result, ensure};
use std::{fs, path::Path};
use vt100::Color;

pub fn run(_runner: &Runner, binary: &Path) -> Result<()> {
	typing(binary)?;
	styles(binary)?;
	colors(binary)?;
	code_clipboard(binary)?;
	for (argument, expected) in [("--diagnostic-error", 1), ("--diagnostic-panic", 101)] {
		let mut s = Session::new(binary, &[argument.as_ref()], &[])?;
		s.finish(expected)?;
	}
	let mut s = Session::new(binary, &[], &[])?;
	s.send("\x11")?;
	s.finish(0)?;
	println!("PASS: editor, Unicode, styling, code clipboard, resize, error/panic restoration");
	Ok(())
}

fn typing(binary: &Path) -> Result<()> {
	let mut s = Session::new(binary, &[], &[])?;
	ensure!(s.row(1).contains("Editor"));
	s.send("abc")?;
	s.segment(2, 7, "abc")?;
	s.send("\x1a")?;
	s.segment(2, 7, "   ")?;
	s.send("\x19")?;
	s.segment(2, 7, "abc")?;
	s.send("\x1b[HX\x1b[F\x7f")?;
	s.segment(2, 7, "Xab")?;
	s.send("\x1a")?;
	s.segment(2, 7, "Xabc")?;
	s.send("\x1b[122;6u")?;
	s.segment(2, 7, "Xab")?;
	s.send("\x1b[<0;8;3M!\x1b[<0;8;3m")?;
	s.segment(2, 7, "!Xab")?;
	s.send(&"\x1a".repeat(8))?;
	s.paste("e\u{301}\t界👩‍💻\r\nsecond\rthird\\u{1b}[31m")?;
	s.segment(3, 7, "second")?;
	ensure!(s.row(4).contains("third\\u{1b}[31m"));
	s.send("\x1a")?;
	s.segment(2, 7, "       ")?;
	s.send("\x19")?;
	ensure!(s.row(3).contains("second"));
	s.send("\x1b[1;5H")?;
	s.send("\x1b[3~")?;
	ensure!(s.parser.screen().cursor_position().1 == 7);
	s.send("\x1bOP")?;
	s.contains("Settings")?;
	s.paste("ignored")?;
	s.send("\x1b")?;
	s.excludes("ignored")?;
	s.send("\x1b[1;5F")?;
	s.paste(&format!(
		"\n{}\n{}",
		"L".repeat(120),
		(0..40)
			.map(|i| format!("line {i}"))
			.collect::<Vec<_>>()
			.join("\n")
	))?;
	s.contains("line 39")?;
	s.send("\x1b[<64;10;10M")?;
	s.excludes("line 39")?;
	s.resize(12, 3)?;
	ensure!(s.row(0).contains("Resize"));
	s.resize(100, 30)?;
	s.send("\x1b[1;5H")?;
	ensure!(s.row(3).contains("second"));
	s.send("\x11")?;
	s.contains("Discard")?;
	s.send("\r")?;
	ensure!(s.row(1).contains("Editor"));
	s.send("\x11d")?;
	s.finish(0)
}

fn styles(binary: &Path) -> Result<()> {
	let mut s = Session::new(binary, &[], &[])?;
	s.paste("Hello\nworld")?;
	s.send(&("\x1b[1;5H".to_owned() + &"\x1b[1;2C".repeat(5)))?;
	s.send("\x02\x1b[105;5u\x15")?;
	let cell = s.cell(2, 7);
	ensure!(cell.bold() && cell.italic() && cell.underline());
	s.click("06", 45)?;
	ensure!(matches!(
		s.cell(2, 7).fgcolor(),
		Color::Idx(6) | Color::Rgb(0, 205, 205)
	));
	s.send("\x1b")?;
	ensure!(s.cell(2, 7).bold());
	let rows = s.rows();
	let first = rows
		.iter()
		.position(|row| row.contains("Hello") && row.contains("\\n"))
		.unwrap();
	ensure!(rows[first + 1].contains("world"));
	s.send("\x1b[<0;13;4M\x1b[<32;8;4M\x1b[<0;8;4m")?;
	s.send("\x02")?;
	ensure!(s.cell(3, 7).bold());
	s.send("X")?;
	s.segment(3, 7, "X    ")?;
	s.send("\x1a")?;
	ensure!(s.row(3).contains("world") && s.cell(3, 7).bold());
	s.send("\x1b")?;
	s.send("\x1b[17~")?;
	s.resize(40, 12)?;
	s.contains("Characters")?;
	s.send("\x1b")?;
	s.resize(80, 24)?;
	s.send("\x11d")?;
	s.finish(0)
}

fn colors(binary: &Path) -> Result<()> {
	let mut s = Session::new(binary, &[], &[])?;
	ensure!(s.row(0).contains("Code: on"));
	ensure!(
		s.rows()
			.iter()
			.filter(|row| row.contains("Wrap: on"))
			.count() == 2
	);
	s.paste("sample")?;
	s.send(&("\x1b[H".to_owned() + &"\x1b[1;2C".repeat(6)))?;
	s.send("\x1b[<0;63;3M\x1b[<0;63;3m")?;
	s.send("\x1b[<0;68;3M\x1b[<0;68;3m")?;
	s.click("Background", 45)?;
	s.click("256", 45)?;
	for _ in 0..3 {
		s.send("\x1b[<65;70;15M")?;
	}
	s.click("42", 45)?;
	s.send("\x1b")?;
	ensure!(s.cell(2, 7).bgcolor() == Color::Idx(42));
	s.send("\x1b[17~\x1b[H")?;
	s.click("RGB", 45)?;
	s.click("Hex:", 45)?;
	s.paste("#0080ff")?;
	ensure!(s.cell(2, 7).bgcolor() == Color::Idx(42));
	s.send("\r")?;
	s.send("\x1b")?;
	s.send("Z")?;
	ensure!(s.cell(2, 13).bgcolor() == Color::Rgb(0, 128, 255));
	s.click("Hex:", 45)?;
	s.paste("#oops")?;
	s.send("\r")?;
	s.contains("6 hex digits")?;
	s.click("Clear", 45)?;
	s.send("\x1b")?;
	s.send("\x1b[17~")?;
	s.resize(40, 12)?;
	s.send("\x1b[1;3C\x1b[1;3C")?;
	s.send("\x06U+256C\r\r\r")?;
	s.contains("Characters")?;
	s.send("\x1b")?;
	s.resize(80, 24)?;
	s.contains("sampleZ╬╬")?;
	ensure!(s.row(2).contains("sampleZ╬╬"));
	s.send("\x1a")?;
	ensure!(s.row(2).contains("sampleZ╬"));
	s.send("\x11d")?;
	s.finish(0)
}

fn code_clipboard(binary: &Path) -> Result<()> {
	let clip = Clipboard::new("")?;
	let mut s = Session::new(binary, &[], &clip.env())?;
	s.paste("alpha\nbeta\t\\n")?;
	s.send("\x1b[17~\x1b[17~")?;
	s.send("\x03")?;
	let expected = r"\x1b[0malpha\nbeta\t\\n\x1b[0m";
	ensure!(fs::read_to_string(&clip.path)? == expected);
	ensure!(s.row(23).contains("Copied escaped ANSI"));
	s.send("\x1b[13~")?;
	s.resize(30, 24)?;
	s.send("\x03")?;
	ensure!(fs::read_to_string(&clip.path)? == expected);
	s.contains("Wrap: on")?;
	s.send("\x1b")?;
	s.send("\x1b[1;5F")?;
	s.paste(&format!("\n{}", "a".repeat(70)))?;
	s.send("\x1b[H")?;
	ensure!(s.parser.screen().cursor_position().1 == 7);
	s.send("\x1b[B")?;
	ensure!(s.row(23).contains("Ln 3"));
	s.send("\x1b[13~")?;
	s.resize(80, 24)?;
	s.send("\x1b[1;5F")?;
	s.paste(" one two")?;
	s.send("\x1b[1;5D\x1b[1;3C\x1b\x7f\x1b[1;5H")?;
	clip.fail()?;
	s.send("\x1b[17~\x1b[17~\x03")?;
	ensure!(s.row(23).contains("Copy failed"));
	s.send("\x1b")?;
	ensure!(s.row(2).contains("alpha"));
	s.send("\x11d")?;
	s.finish(0)
}
