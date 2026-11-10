use crate::{
	process::Runner,
	terminal::{clipboard::Clipboard, fixtures, session::Session},
};
use anyhow::{Context, Result, ensure};
use std::{fs, path::Path};
use vt100::Color;

pub fn run(runner: &Runner, binary: &Path) -> Result<()> {
	persistence(binary).context("persistence scenario")?;
	super::buffers::run(binary).context("buffers scenario")?;
	explorer(runner, binary).context("explorer scenario")?;
	super::directories::run(runner, binary).context("directory workspace scenario")?;
	#[cfg(unix)]
	super::super::unix::paths(runner, binary)?;
	println!("PASS: save/open/export/import, picker, overwrite, invalid startup paths");
	Ok(())
}

fn persistence(binary: &Path) -> Result<()> {
	let temp = tempfile::tempdir()?;
	let folder = temp.path();
	let document = folder.join("saved.tuxd.json");
	let raw = folder.join("snippet.ansi");
	let escaped = folder.join("snippet.txt");
	let bad = folder.join("review.tuxd.json");
	fs::write(&bad, r"\x1b[31mImported\x1b[2J\nUnicode 界\q")?;
	let mut s = Session::editor(binary, &[])?;
	s.paste("Save me\n界")?;
	s.send("\x13")?;
	s.contains("Save document as")?;
	s.paste(document.to_str().unwrap())?;
	s.send("\r")?;
	fixtures::text(&document, "Save me\n界")?;
	for (index, destination) in [&raw, &escaped].iter().enumerate() {
		s.send("\x05")?;
		if index == 0 {
			s.send("\t\x1b[D")?;
		}
		s.paste(destination.to_str().unwrap())?;
		s.send("\r")?;
	}
	ensure!(fs::read(&raw)? == "\x1b[0mSave me\n界\x1b[0m".as_bytes());
	ensure!(fs::read_to_string(&escaped)? == r"\x1b[0mSave me\n界\x1b[0m");
	s.send("\x0f")?;
	s.paste(bad.to_str().unwrap())?;
	s.send("\r")?;
	s.contains("Error:")?;
	s.send("\x1b")?;
	ensure!(s.row(1).contains("Save me"));
	s.send("X")?;
	let import = "\x1b[14~".to_owned() + &"\x1b[B".repeat(7) + "\r";
	s.send(&import)?;
	s.paste(bad.to_str().unwrap())?;
	s.send("\r")?;
	s.contains("Unsupported ANSI Sequences")?;
	s.contains("Seq.")?;
	s.send("a")?;
	ensure!(s.row(1).contains("Save me"));
	s.send(&import)?;
	s.paste(bad.to_str().unwrap())?;
	s.send("\r")?;
	s.send("s")?;
	s.excludes("Save changes before continuing?")?;
	ensure!(s.row(1).contains("Imported"));
	ensure!(matches!(
		s.cell(1, 7).fgcolor(),
		Color::Idx(1) | Color::Rgb(205, 0, 0)
	));
	s.send("\x13")?;
	s.contains("Save document as")?;
	s.send("\x1b")?;
	s.send("\x11dd")?;
	s.finish(0)?;
	let mut s = Session::new(binary, &[document.as_os_str()], &[])?;
	ensure!(s.row(1).contains("Save me"));
	s.send("\x11")?;
	s.finish(0)
}

fn explorer(runner: &Runner, binary: &Path) -> Result<()> {
	let clip = Clipboard::new("")?;
	let folder = clip.directory.path();
	let document = folder.join("browse.tuxd.json");
	fs::copy(runner.root.join("examples/welcome.tuxd.json"), &document)?;
	fs::write(folder.join("other.txt"), "not a document")?;
	fs::copy(&document, folder.join(".hidden.tuxd.json"))?;
	let destination = folder.join("clipboard path.tuxd.json");
	fs::write(&clip.path, destination.to_str().unwrap())?;
	let mut s = Session::editor(binary, &clip.env())?;
	s.send("\x0f\x0c")?;
	s.paste(folder.to_str().unwrap())?;
	s.send("\r")?;
	s.contains("browse.tuxd.json")?;
	s.excludes("other.txt")?;
	s.send("\x1b[46;5u")?;
	s.contains(".hidden.tuxd.json")?;
	s.send("\x1b[46;5u\x1f")?;
	s.contains("Settings")?;
	s.send("\x1b")?;
	s.excludes("Settings")?;
	s.send("\x06")?;
	s.contains("other.txt")?;
	s.send("\x06jggMG\r")?;
	ensure!(s.row(1).contains("Terminal UX Designer"));
	s.send("Z")?;
	let save_as = "\x1b[14~".to_owned() + &"\x1b[B".repeat(5) + "\r";
	s.send(&save_as)?;
	s.send("\x01\x16")?;
	s.send("\r")?;
	ensure!(
		fixtures::read(&destination)?["text"]
			.as_str()
			.unwrap()
			.starts_with("ZTerminal UX Designer")
	);
	s.send(&(save_as + "\t\r"))?;
	s.contains("Overwrite existing file?")?;
	s.send("n")?;
	for cancel in [true, false] {
		s.send("\x0f")?;
		s.paste(folder.join("other.txt").to_str().unwrap())?;
		s.send("\r")?;
		s.contains("Import this file as ANSI?")?;
		s.contains("Raw ANSI")?;
		s.contains("Escaped ANSI")?;
		if cancel {
			s.send("\x1b")?;
			ensure!(s.row(1).contains("ZTerminal UX Designer"));
		} else {
			s.send("\x1b[C\r")?;
			ensure!(s.row(1).contains("not a document"));
		}
	}
	s.send("\x11d")?;
	s.finish(0)
}
