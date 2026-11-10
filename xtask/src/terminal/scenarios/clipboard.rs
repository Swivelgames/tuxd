use crate::terminal::{clipboard::Clipboard, fixtures, session::Session, tmux::Tmux};
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::{ffi::OsStr, fs, path::Path};

pub fn run(binary: &Path) -> Result<()> {
	editor(binary)?;
	fields(binary)?;
	forwarding(binary)?;
	println!("PASS: editor/field clipboard, OSC52 payload and tmux forwarding/failure");
	Ok(())
}

fn editor(binary: &Path) -> Result<()> {
	let clip = Clipboard::new("")?;
	let document = clip.directory.path().join("test.tuxd.json");
	fixtures::empty(&document)?;
	let mut s = Session::new(binary, &[document.as_os_str()], &clip.env())?;
	s.paste("café\n界é")?;
	s.send("\x01\x02\x03")?;
	let copied = fs::read_to_string(&clip.path)?;
	ensure!(copied.contains("\\x1b[0;1m") && copied.contains("café\\n界é"));
	s.send("\x18")?;
	s.send("\x13")?;
	fixtures::text(&document, "")?;
	s.send("\x1a\x13")?;
	let original = fixtures::read(&document)?;
	ensure!(original["text"] == "café\n界é");
	s.send("\x01\x16")?;
	s.send("\x13")?;
	ensure!(fixtures::read(&document)? == original);
	fs::write(&clip.path, r"\x1b[31mnew\x1b[2J")?;
	s.send("\x01\x16")?;
	s.contains("Unsupported ANSI")?;
	s.contains("Strip and paste")?;
	s.send("\x1b")?;
	s.send("\x13")?;
	ensure!(fixtures::read(&document)? == original);
	s.send("\x16")?;
	s.send("s")?;
	s.send("\x13")?;
	fixtures::text(&document, "new")?;
	s.send("\x11")?;
	s.finish(0)
}

fn fields(binary: &Path) -> Result<()> {
	let clip = Clipboard::new("#aabbcc")?;
	let mut s = Session::editor(binary, &clip.env())?;
	s.click("RGB", 45)?;
	s.click("Hex:", 45)?;
	s.send("\x01\x16")?;
	s.contains("#aabbcc")?;
	s.send("\x01\x03")?;
	ensure!(fs::read_to_string(&clip.path)? == "#aabbcc");
	s.send("\t\x01\x03")?;
	ensure!(fs::read_to_string(&clip.path)? == "170");
	fs::write(&clip.path, "128")?;
	s.send("\x16")?;
	s.contains("#80bbcc")?;
	s.send("\x1b")?;
	s.send("\x0f\x0c")?;
	let directory = clip.directory.path().to_str().unwrap();
	s.paste(directory)?;
	s.send("\x01\x03")?;
	ensure!(fs::read_to_string(&clip.path)? == directory);
	s.send("\x1b")?;
	s.send("\x11")?;
	s.finish(0)
}

fn exercise(s: &mut Session, expected: &str) -> Result<()> {
	s.paste(expected)?;
	s.send("\x1b[17~\x1b[17~")?;
	let before = s.output.len();
	s.send("\x03")?;
	let regex = regex::bytes::Regex::new(r"\x1b\]52;[^;]*;([A-Za-z0-9+/=]*)(?:\x07|\x1b\\)")?;
	let payload = regex
		.captures_iter(&s.output[before..])
		.last()
		.context("No OSC52 forwarding")?;
	ensure!(STANDARD.decode(&payload[1])? == expected.as_bytes());
	s.resize(120, 24)?;
	s.contains("delivery unconfirmed")
}

fn forwarding(binary: &Path) -> Result<()> {
	let text = "escaped \\x1b[31m 界 ".to_owned() + &"long payload ".repeat(800);
	let expected = format!("\\x1b[0m{}\\x1b[0m", text.replace('\\', "\\\\"));
	let mut s = Session::editor(
		binary,
		&[
			("SSH_CONNECTION", Some(OsStr::new("test"))),
			("DISPLAY", None),
			("WAYLAND_DISPLAY", None),
		],
	)?;
	exercise(&mut s, &expected)?;
	s.send("\x11d")?;
	s.finish(0)?;
	let tmux = Tmux::new()?;
	let config = tempfile::tempdir()?;
	ensure!(
		tmux.command()
			.args(["-f", "/dev/null", "new-session", "-d", "-s", "test", "env"])
			.arg(format!("XDG_CONFIG_HOME={}", config.path().display()))
			.arg(binary)
			.status()?
			.success()
	);
	tmux.run(&["set-option", "-s", "set-clipboard", "external"])?;
	tmux.run(&[
		"set-option",
		"-as",
		"terminal-features",
		",xterm*:clipboard",
	])?;
	let mut command = Session::command("tmux");
	command.arg("-S");
	command.arg(&tmux.socket);
	command.args(["attach-session", "-t", "test"]);
	let mut s = Session::spawn(command, &[])?;
	s.hide_sidebar()?;
	exercise(&mut s, &expected)?;
	let names = tmux
		.command()
		.args(["list-buffers", "-F", "#{buffer_name}"])
		.output()?;
	ensure!(names.status.success());
	let names = String::from_utf8(names.stdout)?;
	let name = names
		.lines()
		.find(|name| name.starts_with("tuxd-"))
		.context("Missing tmux buffer")?;
	let saved = tmux
		.command()
		.args(["save-buffer", "-b", name, "-"])
		.output()?;
	ensure!(saved.status.success() && saved.stdout == expected.as_bytes());
	s.send("\x11d")?;
	ensure!(s.wait()? == 0);
	let empty = tempfile::tempdir()?;
	let mut s = Session::editor(
		binary,
		&[
			("TMUX", Some(OsStr::new("unavailable"))),
			("PATH", Some(empty.path().as_os_str())),
		],
	)?;
	s.send("\x1b[17~\x1b[17~\x03")?;
	s.contains("Copy failed: tmux")?;
	s.send("\x11")?;
	s.finish(0)
}
