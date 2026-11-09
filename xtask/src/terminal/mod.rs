//! Shared terminal testing infrastructure and capability-focused scenarios.
pub mod clipboard;
pub mod fixtures;
mod scenarios;
mod session;
pub mod tmux;
#[cfg(unix)]
pub mod unix;

use crate::process::Runner;
use anyhow::Result;
use clap::ValueEnum;
use std::path::Path;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Suite {
	Editor,
	Files,
	Shortcuts,
	Characters,
	Clipboard,
	Stdin,
	Themes,
	All,
}

pub fn run(runner: &Runner, suite: Suite) -> Result<()> {
	crate::check::linux()?;
	let binary = fixtures::binary(runner, false)?;
	match suite {
		Suite::Editor => scenarios::editor::run(runner, &binary),
		Suite::Files => scenarios::files::run(runner, &binary),
		Suite::Shortcuts => scenarios::settings::shortcuts(&binary),
		Suite::Characters => scenarios::characters::run(runner, &binary),
		Suite::Clipboard => scenarios::clipboard::run(&binary),
		Suite::Stdin => scenarios::stdin::run(&binary),
		Suite::Themes => scenarios::settings::themes(&binary),
		Suite::All => {
			scenarios::editor::run(runner, &binary)?;
			scenarios::files::run(runner, &binary)?;
			scenarios::settings::shortcuts(&binary)?;
			scenarios::characters::run(runner, &binary)?;
			scenarios::clipboard::run(&binary)?;
			scenarios::stdin::run(&binary)?;
			scenarios::settings::themes(&binary)
		}
	}
}

pub fn packaged(runner: &Runner, binary: &Path, version: &str) -> Result<()> {
	crate::check::linux()?;
	anyhow::ensure!(
		runner.output(runner.command(binary).arg("--version"))? == format!("tuxd {version}"),
		"Packaged version mismatch"
	);
	let temp = tempfile::tempdir()?;
	let path = temp.path().join("check.tuxd.json");
	std::fs::copy(runner.root.join("examples/welcome.tuxd.json"), &path)?;
	let original = fixtures::read(&path)?["text"].as_str().unwrap().to_owned();
	let mut s = session::Session::new(binary, &[path.as_os_str()], &[])?;
	s.contains("Editor")?;
	s.send("\x1b[1;5F")?;
	s.paste(" release café 界")?;
	s.send("\x13")?;
	fixtures::text(&path, &(original + " release café 界"))?;
	s.send("\x1bOP")?;
	s.contains("Settings")?;
	s.send("\x1b")?;
	s.resize(60, 24)?;
	s.contains("Code")?;
	s.send("\x11")?;
	s.finish(0)?;
	println!("PASS: packaged binary and terminal restoration");
	Ok(())
}
