use super::display::Display;
use crate::process::ManagedChild;
use crate::{
	process::Runner,
	terminal::{fixtures, tmux::Tmux},
};
use anyhow::{Result, ensure};
use std::{ffi::OsString, fs, time::Duration};

pub fn check(runner: &Runner, raw: bool) -> Result<()> {
	crate::check::prerequisites(runner)?;
	let mut command = runner.command("cargo");
	command.args([
		"run",
		"--quiet",
		"--locked",
		"--example",
		"character_widths",
	]);
	if raw {
		command.args(["--", "--raw"]);
	}
	let samples = runner.output(&mut command)?;
	let temp = tempfile::tempdir()?;
	let folder = temp.path();
	let inputs = folder.join("samples.jsonl");
	fs::write(&inputs, samples)?;
	let config = folder.join("alacritty.toml");
	fs::write(
		&config,
		"[window]\ndimensions = { columns = 100, lines = 30 }\n",
	)?;
	let display = Display::new(folder, 1400, 1000)?;
	let tmux = Tmux::new()?;
	let mut failed = false;
	for mode in ["bare", "tmux"] {
		let output = folder.join(format!("{mode}.json"));
		let mut child: Vec<OsString> = Vec::new();
		if mode == "tmux" {
			child.extend([
				"tmux".into(),
				"-S".into(),
				tmux.socket.clone().into_os_string(),
				"-f".into(),
				"/dev/null".into(),
				"new-session".into(),
			]);
		}
		child.extend([
			std::env::current_exe()?.into_os_string(),
			"__width-probe".into(),
			inputs.clone().into_os_string(),
			output.clone().into_os_string(),
		]);
		let mut app = ManagedChild::spawn(
			display
				.command("alacritty")
				.arg("--config-file")
				.arg(&config)
				.arg("-e")
				.args(child)
				.stdout(std::process::Stdio::null())
				.stderr(std::process::Stdio::null()),
		)?;
		ensure!(
			app.wait(Duration::from_secs(180))?.success(),
			"Width probe process failed"
		);
		let result = fixtures::read(&output)?;
		let failures = result["failures"].as_array().unwrap();
		println!(
			"{mode}: {} samples, {} width mismatches",
			result["count"],
			failures.len()
		);
		if !failures.is_empty() {
			failed = true;
			println!("{result}");
		}
	}
	ensure!(!failed, "Character width mismatch");
	Ok(())
}

#[cfg(unix)]
pub fn probe(samples: &std::path::Path, output: &std::path::Path) -> Result<()> {
	use crate::terminal::unix::RawMode;
	use std::{io::Write, time::Instant};
	let raw = RawMode::stdin()?;
	let cursor = regex::bytes::Regex::new(r"\x1b\[(\d+);(\d+)R")?;
	let mut count = 0;
	let mut failures = Vec::new();
	for line in fs::read_to_string(samples)?.lines() {
		let mut sample: serde_json::Value = serde_json::from_str(line)?;
		write!(
			std::io::stdout(),
			"\x1b[1;1H\x1b[2K{}\x1b[6n",
			sample["display"].as_str().unwrap()
		)?;
		std::io::stdout().flush()?;
		let mut reply = Vec::new();
		let deadline = Instant::now() + Duration::from_secs(2);
		let mut buffer = [0; 1024];
		while !cursor.is_match(&reply) {
			ensure!(
				Instant::now() < deadline,
				"Terminal did not answer cursor-position query"
			);
			let count = raw.read_ready(&mut buffer)?;
			reply.extend_from_slice(&buffer[..count]);
		}
		let matched = cursor.captures(&reply).unwrap();
		let row: usize = std::str::from_utf8(&matched[1])?.parse()?;
		let actual: usize = std::str::from_utf8(&matched[2])?.parse::<usize>()? - 1;
		if row != 1 || sample["width"] != actual {
			sample["actual"] = actual.into();
			failures.push(sample);
		}
		count += 1;
	}
	drop(raw);
	crate::receipt::write_json(
		output,
		&serde_json::json!({"count":count,"failures":failures}),
	)
}
