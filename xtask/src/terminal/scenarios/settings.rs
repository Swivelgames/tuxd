use crate::terminal::{fixtures, session::Session};
use anyhow::{Result, ensure};
use serde_json::json;
use std::{fs, path::Path};
use vt100::Color;

fn category(s: &mut Session, index: usize) -> Result<()> {
	s.send(&("\x0cgg".to_owned() + &"j".repeat(index + 3) + "\r"))
}

fn click(s: &mut Session, label: &str) -> Result<()> {
	for _ in 0..40 {
		for (row, text) in s.rows().iter().enumerate() {
			if let Some(byte) = text.find(label) {
				let col = text[..byte].chars().count();
				return s.send(&format!(
					"\x1b[<0;{};{}M\x1b[<0;{};{}m",
					col + 1,
					row + 1,
					col + 1,
					row + 1
				));
			}
		}
		s.send("\x1b[<65;76;7M")?;
	}
	anyhow::bail!("Missing setting {label}: {}", s.text())
}

pub fn shortcuts(binary: &Path) -> Result<()> {
	let temp = tempfile::tempdir()?;
	let path = temp.path().join("tuxd/shortcuts.json");
	let env = [("XDG_CONFIG_HOME", Some(temp.path().as_os_str()))];
	let mut s = Session::new(binary, &[], &env)?;
	s.send("\x1bOP")?;
	category(&mut s, 1)?;
	click(&mut s, "New document")?;
	s.send("1\x1b[19~")?;
	s.contains("unsaved")?;
	ensure!(!path.exists());
	s.send("\x13")?;
	ensure!(fixtures::read(&path)?["bindings"]["new"][0][0]["code"] == json!({"F":8}));
	s.send("\x1b[14~")?;
	ensure!(
		s.rows()
			.iter()
			.any(|r| r.contains("New document") && r.contains("F8"))
	);
	s.send("\x1b")?;
	s.send("\x1bOP")?;
	category(&mut s, 1)?;
	click(&mut s, "Focus Code")?;
	s.send("2\x17\x1b[C")?;
	s.contains("Recorded: Ctrl+W Right")?;
	s.send("\x14250\r\x13")?;
	ensure!(fixtures::read(&path)?["timeout_ms"] == 250);
	s.send("\x17\x1b[C\x1bOR")?;
	ensure!(s.rows().iter().filter(|r| r.contains("Wrap: off")).count() == 1);
	s.send("\x1b")?;
	s.send("\x1bOP")?;
	category(&mut s, 1)?;
	click(&mut s, "Focus Tools")?;
	s.send("1\x17")?;
	s.contains("conflicts")?;
	s.send("\x13")?;
	s.contains("Settings")?;
	category(&mut s, 1)?;
	s.send("\r\x1b[3~")?;
	s.send("\x1b")?;
	s.excludes("Settings")?;
	s.send("\x0f")?;
	for opener in ["\x1bOP", "\x1f", "\x1b[44;5u"] {
		s.send(opener)?;
		ensure!(s.text().contains("File Explorer") || s.text().contains("Select all"));
		for (cols, rows) in [(140, 40), (64, 20), (24, 8), (80, 24)] {
			s.resize(cols, rows)?;
		}
		s.send("\x1b")?;
		s.contains("Open document")?;
	}
	s.send("\x1b")?;
	s.send("\x11")?;
	s.finish(0)?;
	let mut s = Session::new(binary, &[], &env)?;
	s.paste("preserved")?;
	s.send("\x1b[19~")?;
	s.excludes("Save changes before continuing?")?;
	s.send("\x1b[5;5~")?;
	s.contains("preserved")?;
	let before = fs::read(&path)?;
	s.send("\x1bOP")?;
	category(&mut s, 1)?;
	click(&mut s, "New document")?;
	s.send("\r\x1b[3~")?;
	s.send("\x1b")?;
	s.contains("Discard unsaved settings changes?")?;
	s.send("\r")?;
	s.contains("Settings")?;
	s.send("\x1b")?;
	s.send("\x1b[D\r")?;
	ensure!(fs::read(&path)? == before);
	s.send("\x11d")?;
	s.finish(0)?;
	fs::write(&path, "malformed settings")?;
	let mut s = Session::new(binary, &[], &env)?;
	s.contains("Shortcuts:")?;
	ensure!(fs::read_to_string(&path)? == "malformed settings");
	s.send("\x11")?;
	s.finish(0)?;
	leader(binary)?;
	println!("PASS: shortcut capture, sequences, conflicts, persistence, context return, leader");
	Ok(())
}

fn leader(binary: &Path) -> Result<()> {
	let temp = tempfile::tempdir()?;
	let mut s = Session::new(
		binary,
		&[],
		&[("XDG_CONFIG_HOME", Some(temp.path().as_os_str()))],
	)?;
	s.send("\x1bOP\x0cggjjj\r")?;
	s.send("\r")?;
	s.send("\x19")?;
	s.contains("Redo")?;
	s.contains("unset")?;
	s.send("y\x13")?;
	let path = temp.path().join("tuxd/shortcuts.json");
	ensure!(!fixtures::read(&path)?["leader"].is_null());
	s.send("\x1bOP\x0cggjjj\r")?;
	s.send("\r")?;
	s.send("\x1b")?;
	s.send("\x1b[3~")?;
	s.excludes("Unset Leader?")?;
	s.send("\x13")?;
	ensure!(fixtures::read(&path)?["leader"].is_null());
	s.send("\x11")?;
	s.finish(0)
}

pub fn themes(binary: &Path) -> Result<()> {
	theme_loading(binary)?;
	theme_settings(binary)?;
	println!("PASS: theme overrides, fallback, staged preview, save and restart");
	Ok(())
}

fn theme_loading(binary: &Path) -> Result<()> {
	let temp = tempfile::tempdir()?;
	let config = temp.path().join("tuxd/theme.json");
	fs::create_dir(config.parent().unwrap())?;
	let doc = temp.path().join("test.tuxd.json");
	let document = json!({"version":1,"text":"Document","styles":{"runs":[{"start":0,"end":8,"style":{"bold":false,"italic":false,"underline":false,"dim":false,"strike":false,"foreground":"Default","background":"Default"}}]}});
	fixtures::write(&doc, &document)?;
	for valid in [true, false] {
		fs::write(
			&config,
			if valid {
				json!({"version":1,"preset":"midnight","colors":{"accent":"#123456","surface":"#334455","text":"#abcdef"}}).to_string()
			} else {
				"{invalid".into()
			},
		)?;
		let original = fs::read(&config)?;
		let mut s = Session::new(
			binary,
			&[doc.as_os_str()],
			&[("XDG_CONFIG_HOME", Some(temp.path().as_os_str()))],
		)?;
		if valid {
			ensure!(s.cell(0, 3).fgcolor() == Color::Rgb(18, 52, 86));
			let (y, row) = s
				.rows()
				.into_iter()
				.enumerate()
				.find(|(_, row)| row.contains("Document"))
				.unwrap();
			let x = row[..row.find("Document").unwrap()].chars().count();
			ensure!(
				s.cell(y as u16, x as u16).fgcolor() == Color::Default
					&& s.cell(y as u16, x as u16).bgcolor() == Color::Default
			);
		} else {
			s.contains("Theme:")?;
		}
		s.send("\x13\x11")?;
		s.finish(0)?;
		ensure!(fixtures::read(&doc)? == document && fs::read(&config)? == original);
	}
	Ok(())
}

fn appearance(s: &mut Session) -> Result<()> {
	s.send("\x1bOP\x0cggj\r")?;
	s.contains("Appearance")?;
	s.send("\x1b[B")
}

fn theme_settings(binary: &Path) -> Result<()> {
	let temp = tempfile::tempdir()?;
	let path = temp.path().join("tuxd/theme.json");
	let env = [("XDG_CONFIG_HOME", Some(temp.path().as_os_str()))];
	let mut s = Session::new(binary, &[], &env)?;
	let baseline = s.cell(0, 3).fgcolor();
	appearance(&mut s)?;
	s.send("\r")?;
	s.paste("#123456")?;
	ensure!(!path.exists());
	s.send("\r\x1b")?;
	s.contains("Discard unsaved settings")?;
	s.send("\x1b[D\r")?;
	ensure!(!path.exists() && s.cell(0, 3).fgcolor() == baseline);
	appearance(&mut s)?;
	s.send("\r")?;
	s.paste("#123456")?;
	s.send("\r\x13")?;
	ensure!(
		fixtures::read(&path)?["colors"]["accent"] == "#123456"
			&& s.cell(0, 3).fgcolor() == Color::Rgb(18, 52, 86)
	);
	let shortcuts_path = path.with_file_name("shortcuts.json");
	let shortcuts = fs::read(&shortcuts_path)?;
	s.send("\x1bOP\x0cggj\r\x1b[C\x13")?;
	ensure!(
		fixtures::read(&path)?["preset"] == "light"
			&& fixtures::read(&path)?["colors"]["accent"] == "#123456"
	);
	s.send("\x1bOP\x0cggj\r\t\r\x13")?;
	ensure!(
		fixtures::read(&path)?["preset"] == "light"
			&& fixtures::read(&path)?["colors"] == json!({})
			&& fs::read(&shortcuts_path)? == shortcuts
			&& s.cell(0, 3).fgcolor() == Color::Rgb(0, 90, 128)
	);
	s.send("\x1bOP\x0cggj\r\x1b[C")?;
	s.send("\x1b")?;
	s.send("\x1b[D\r")?;
	ensure!(s.cell(0, 3).fgcolor() == Color::Rgb(0, 90, 128));
	s.send("\x1bOP\x0cggj\r\rG")?;
	s.contains("Catppuccin Mocha")?;
	s.send("\r\x13")?;
	ensure!(
		fixtures::read(&path)?["preset"] == "catppuccin-mocha"
			&& s.cell(0, 3).fgcolor() == Color::Rgb(137, 180, 250)
	);
	s.send("\x11")?;
	s.finish(0)?;
	let mut s = Session::new(binary, &[], &env)?;
	ensure!(s.cell(0, 3).fgcolor() == Color::Rgb(137, 180, 250));
	s.send("\x11")?;
	s.finish(0)
}
