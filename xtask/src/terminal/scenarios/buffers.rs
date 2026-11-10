//! Multi-document acceptance through the real terminal input and file workflows.
use crate::terminal::{fixtures, session::Session};
use anyhow::{Result, ensure};
use std::path::Path;

pub fn run(binary: &Path) -> Result<()> {
	let folder = tempfile::tempdir()?;
	let first = folder.path().join("first.tuxd.json");
	let second = folder.path().join("second.tuxd.json");
	let mut s = Session::editor(binary, &[])?;
	s.paste("first buffer")?;
	s.send("\x0e")?;
	s.paste("second buffer")?;
	s.excludes("Save changes before continuing?")?;
	s.send("\x1b[5;5~")?;
	ensure!(s.row(1).contains("first buffer"));
	s.send("\x1a")?;
	ensure!(!s.row(1).contains("first buffer"));
	s.send("\x19\x1b[6;5~")?;
	ensure!(s.row(1).contains("second buffer"));
	s.send("\x1b[115;7u")?;
	for path in [&first, &second] {
		s.contains("Save document as")?;
		s.paste(path.to_str().unwrap())?;
		s.send("\r")?;
	}
	fixtures::text(&first, "first buffer")?;
	fixtures::text(&second, "second buffer")?;
	ensure!(s.row(1).contains("second buffer"));
	s.send("\x0f")?;
	s.paste(first.to_str().unwrap())?;
	s.send("\r")?;
	ensure!(s.row(1).contains("first buffer"));
	s.send("\x1b[6;5~")?;
	ensure!(s.row(1).contains("second buffer"));
	s.send("\x1b[14;5~")?;
	ensure!(s.row(1).contains("first buffer"));
	s.send("\x1b[14;5~")?;
	ensure!(s.row(0).contains("untitled"));
	s.send("\x11")?;
	s.finish(0)?;
	println!("PASS: document tabs, independent undo, Save All, duplicate open, close last");
	Ok(())
}
