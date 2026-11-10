use super::display::{Display, Window};
use crate::{
	process::Runner,
	terminal::{fixtures, tmux::Tmux},
};
use anyhow::{Result, ensure};
use clap::ValueEnum;
use std::{ffi::OsString, fs, time::Duration};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Mode {
	Workspace,
	Basic,
	Unicode,
	Characters,
	Symbols,
}

pub fn render(runner: &Runner, mode: Mode, with_tmux: bool) -> Result<()> {
	crate::check::prerequisites(runner)?;
	let binary = fixtures::binary(runner, true)?;
	let evidence = tempfile::Builder::new()
		.prefix("tuxd-alacritty-evidence-")
		.tempdir()?
		.keep();
	println!("Rendering evidence: {}", evidence.display());
	let temp = tempfile::tempdir()?;
	let folder = temp.path();
	let config = folder.join("alacritty.toml");
	fs::write(
		&config,
		"[window]\ndimensions = { columns = 110, lines = 36 }\n[font]\nsize = 12\n[cursor]\nstyle = { shape = \"Block\", blinking = \"Never\" }\n",
	)?;
	let document = folder.join("check.tuxd.json");
	let mut original = fixtures::read(&runner.root.join("examples/welcome.tuxd.json"))?;
	if matches!(mode, Mode::Unicode | Mode::Symbols) {
		let text = if matches!(mode, Mode::Symbols) {
			"☝ ⛹ ✌ ✍ 🏋 🏌 🕴 🕵 🖐"
		} else {
			"café é 界 👩🏽‍💻 🇺🇳"
		};
		let mut style = original["styles"]["runs"][0]["style"].clone();
		style["bold"] = false.into();
		original["text"] = text.into();
		original["styles"]["runs"] =
			serde_json::json!([{"start":0,"end":text.len(),"style":style}]);
	}
	fixtures::write(&document, &original)?;
	let display = Display::new(folder, 1400, 1000)?;
	let tmux = Tmux::new()?;
	let mut command: Vec<OsString> = Vec::new();
	if with_tmux {
		command.extend([
			"tmux".into(),
			"-S".into(),
			tmux.socket.clone().into_os_string(),
			"-f".into(),
			"/dev/null".into(),
			"new-session".into(),
			"-s".into(),
			"test".into(),
		]);
	}
	let source = if matches!(mode, Mode::Workspace) {
		fs::create_dir(folder.join("nested"))?;
		fs::copy(&document, folder.join("nested/child.tuxd.json"))?;
		folder.to_path_buf()
	} else {
		document.clone()
	};
	command.extend([binary.into_os_string(), source.into_os_string()]);
	let mut window = Window::open(
		&display,
		&config,
		&runner.root,
		&command,
		&evidence.join("alacritty.log"),
	)?;
	if with_tmux {
		tmux.run(&["set-option", "prefix", "C-a"])?;
		tmux.run(&["set-option", "-s", "escape-time", "10"])?;
	}
	// Dismiss the startup notice so restoration compares the same status line.
	window.keys(&["Home"])?;
	window.capture(&evidence.join("initial.png"))?;
	if matches!(mode, Mode::Workspace) {
		window.keys(&["G", "Return"])?;
		window.capture(&evidence.join("workspace-wide.png"))?;
		window.keys(&["ctrl+alt+e"])?;
		window.resize(600, 740)?;
		window.capture(&evidence.join("workspace-slim.png"))?;
		window.keys(&["Escape"])?;
		// Allow tmux Escape forwarding to settle before capture and the next key.
		std::thread::sleep(Duration::from_millis(700));
		window.capture(&evidence.join("workspace-slim-hidden.png"))?;

		window.keys(&["ctrl+q"])?;
		ensure!(window.app.wait(Duration::from_secs(5))?.success());
		println!("PASS: Alacritty workspace sidebar and slim overlay");
		return Ok(());
	}
	if matches!(mode, Mode::Characters) {
		characters(&window, &evidence)?;
	}
	if matches!(mode, Mode::Unicode | Mode::Symbols) {
		window.keys(&["ctrl+End", "shift+Home"])?;
		window.capture(&evidence.join("unicode-selected.png"))?;
		let count = if matches!(mode, Mode::Symbols) {
			30
		} else {
			10
		};
		let mut keys = vec!["Home"];
		keys.extend(vec!["Return"; count]);
		window.keys(&keys)?;
		window.capture(&evidence.join("unicode-newlines.png"))?;
		window.keys(&vec!["BackSpace"; count])?;
		window.capture(&evidence.join("unicode-restored.png"))?;
		let size = runner.output(
			runner
				.command("magick")
				.args(["identify", "-format", "%w %h"])
				.arg(evidence.join("initial.png")),
		)?;
		let values: Vec<u32> = size
			.split_whitespace()
			.map(str::parse)
			.collect::<std::result::Result<_, _>>()?;
		let (w, h) = (values[0], values[1]);
		let cell = h / 36;
		let crop = format!("[{}x{}+0+{}]", w, h - 4 * cell, 2 * cell);
		runner.run(
			runner
				.command("magick")
				.args(["compare", "-metric", "AE"])
				.arg(format!("{}{crop}", evidence.join("initial.png").display()))
				.arg(format!(
					"{}{crop}",
					evidence.join("unicode-restored.png").display()
				))
				.arg("null:"),
		)?;
	}
	window.keys(&["ctrl+End"])?;
	window.xdo(&["type", "--clearmodifiers", "--delay", "10", " GUI verified"])?;
	let mut keys = vec!["shift+Left"; 13];
	keys.extend(["ctrl+b", "ctrl+s"]);
	window.keys(&keys)?;
	let saved = fixtures::read(&document)?;
	ensure!(saved["text"] == original["text"].as_str().unwrap().to_owned() + " GUI verified");
	ensure!(saved["styles"]["runs"].as_array().unwrap().last().unwrap()["style"]["bold"] == true);
	window.capture(&evidence.join("edited.png"))?;
	window.keys(&["F1"])?;
	window.capture(&evidence.join("settings.png"))?;
	window.keys(&["Escape"])?;
	std::thread::sleep(Duration::from_millis(700));
	for (width, height) in [(430, 360), (280, 220), (1100, 740)] {
		window.resize(width, height)?;
		window.keys(&["F2", "F2", "F5", "F5"])?;
		ensure!(window.app.0.try_wait()?.is_none());
		window.capture(&evidence.join(format!("resize-{width}.png")))?;
	}
	window.capture(&evidence.join("before-exit.png"))?;
	window.keys(&["ctrl+q"])?;
	ensure!(window.app.wait(Duration::from_secs(5))?.success());
	ensure!(fixtures::read(&document)? == saved);
	println!("PASS: Alacritty {mode:?}, tmux={with_tmux}");
	Ok(())
}

fn characters(window: &Window<'_>, evidence: &std::path::Path) -> Result<()> {
	window.resize(640, 800)?;
	window.keys(&["F6", "alt+Right", "alt+Right", "alt+Right", "F5", "ctrl+f"])?;
	window.xdo(&["type", "--clearmodifiers", "arrow"])?;
	window.keys(&["Return", "Right", "Down"])?;
	window.capture(&evidence.join("characters-search.png"))?;
	window.resize(1100, 800)?;
	window.keys(&["ctrl+f", "ctrl+a"])?;
	window.xdo(&["type", "--clearmodifiers", "U+2500"])?;
	window.keys(&["Return"])?;
	window.capture(&evidence.join("characters-wide.png"))?;
	window.keys(&["ctrl+f", "ctrl+a", "BackSpace", "shift+Tab", "Return"])?;
	window.capture(&evidence.join("characters-groups.png"))?;
	window.xdo(&["type", "--clearmodifiers", "Emoji"])?;
	window.keys(&["Return", "ctrl+f", "Return", "Home"])?;
	window.capture(&evidence.join("characters-emoji.png"))?;
	window.keys(&["Return"])?;
	window.capture(&evidence.join("characters-types.png"))?;
	window.keys(&["Escape", "Escape"])
}
