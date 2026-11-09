use crate::{
	process::Runner,
	terminal::{fixtures, session::Session},
};
use anyhow::{Result, ensure};
use std::path::Path;

pub fn run(runner: &Runner, binary: &Path) -> Result<()> {
	let temp = tempfile::tempdir()?;
	let path = temp.path().join("characters.tuxd.json");
	let mut doc = fixtures::read(&runner.root.join("examples/welcome.tuxd.json"))?;
	doc["text"] = "".into();
	doc["styles"]["runs"] = serde_json::json!([]);
	fixtures::write(&path, &doc)?;
	let mut s = Session::new(binary, &[path.as_os_str()], &[])?;
	s.resize(64, 36)?;
	s.send("\x1b[17~\x1b[1;3C\x1b[1;3C\x1b[1;3C\x1b[15~")?;
	s.contains("Group:")?;
	s.send("\x06")?;
	s.paste("U+2500")?;
	s.send("\r")?;
	s.contains("U+2500")?;
	s.send("\x1b[H\r")?;
	s.contains("▸ Box Drawing")?;
	s.send("\r\x1b[B")?;
	s.contains("▾ Box Drawing")?;
	s.send("\r\r\x1b")?;
	s.send("\x13")?;
	fixtures::text(&path, "──")?;
	s.send("\x1a\x13")?;
	fixtures::text(&path, "─")?;
	s.send("\x1b[17~\x1b[15~\x06\x01")?;
	s.paste("md home")?;
	s.send("\r\x1b[F\x1b[5~\x1b[H")?;
	s.contains("Nerd Icons")?;
	for (cols, rows) in [(40, 12), (120, 40), (64, 36)] {
		s.resize(cols, rows)?;
		ensure!(s.alive()?);
	}
	s.send("\x06\x01")?;
	s.paste("U+1F680")?;
	s.contains("Emoji")?;
	s.send("\r\r\x1b")?;
	s.send("\x13")?;
	fixtures::text(&path, "─🚀")?;
	s.send("\x1a\x13")?;
	fixtures::text(&path, "─")?;
	s.send("\x1b[17~\x1b[15~\x06\x01")?;
	s.paste("U+200D")?;
	s.contains("No matching characters")?;
	s.send("\r\x1b")?;
	s.send("\x11")?;
	s.finish(0)?;
	println!("PASS: character discovery, insertion, undo, resize, exclusion");
	Ok(())
}
