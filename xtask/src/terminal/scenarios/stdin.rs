use anyhow::Result;
use std::path::Path;

#[cfg(not(unix))]
pub fn run(_binary: &Path) -> Result<()> {
	anyhow::bail!("stdin controlling-terminal checks require Unix")
}

#[cfg(unix)]
pub fn run(binary: &Path) -> Result<()> {
	use crate::terminal::{fixtures, session::Session, unix};
	use anyhow::ensure;
	use std::{
		fs,
		io::Write,
		process::{Command, Stdio},
		time::Duration,
	};
	let temp = tempfile::tempdir()?;
	for (index, (mode, data, review, accept, file_source)) in [
		("--raw", "\x1b[1;31mImported\x1b[0m", false, true, false),
		(
			"--escaped",
			r"\x1b[1;31mImported\x1b[0m",
			false,
			true,
			false,
		),
		("--raw", "Imported\x1b[2J", true, true, false),
		("--escaped", r"Imported\x1b[2J", true, false, false),
		("--raw", "", false, true, false),
		("--raw", "Imported", false, true, true),
		("--escaped", r"Imported\x1b[2J", true, true, true),
	]
	.iter()
	.enumerate()
	{
		let source = temp.path().join(format!("source-{index}.txt"));
		fs::write(&source, data)?;
		let mut s = if *file_source {
			Session::new(binary, &[mode.as_ref(), source.as_os_str()], &[])?
		} else {
			unix::imported(binary, mode, &source, true)?
		};
		if *review {
			s.contains("Unsupported ANSI")?;
			s.send(if *accept { "s" } else { "\x1b" })?;
		}
		s.send("\x13")?;
		s.contains("Save document as")?;
		let destination = temp.path().join(format!("saved-{index}.tuxd.json"));
		s.paste(destination.to_str().unwrap())?;
		s.send("\r")?;
		fixtures::text(
			&destination,
			if !data.is_empty() && *accept {
				"Imported"
			} else {
				""
			},
		)?;
		if index < 2 {
			ensure!(
				!fixtures::read(&destination)?["styles"]["runs"]
					.as_array()
					.unwrap()
					.is_empty()
			);
		}
		ensure!(fs::read(&source)? == data.as_bytes());
		s.send("\x11")?;
		s.finish(0)?;
	}
	for (payload, message) in [
		(vec![255], "UTF-8"),
		(vec![b'x'; 16 * 1024 * 1024 + 1], "16 MiB"),
	] {
		let source = temp.path().join("invalid");
		fs::write(&source, payload)?;
		let mut s = unix::imported(binary, "--raw", &source, false)?;
		ensure!(s.wait()? != 0);
		let output = String::from_utf8_lossy(&s.output);
		ensure!(output.contains(message) && !output.contains("\x1b[?1049h"));
	}
	let mut s = Session::new(binary, &["--raw".as_ref(), "-".as_ref()], &[])?;
	ensure!(s.wait()? != 0);
	ensure!(String::from_utf8_lossy(&s.output).contains("Expected piped or redirected stdin"));
	unix::no_controlling_terminal(binary)?;
	for args in [
		vec!["-"],
		vec!["--raw"],
		vec!["--raw", "--escaped"],
		vec!["--raw", "-"],
	] {
		let mut child = crate::process::ManagedChild::spawn(
			Command::new(binary)
				.args(args)
				.stdin(Stdio::piped())
				.stdout(Stdio::piped())
				.stderr(Stdio::piped()),
		)?;
		let _ = child.0.stdin.take().unwrap().write_all(b"text");
		ensure!(!child.wait(Duration::from_secs(5))?.success());
		let mut stdout = String::new();
		std::io::Read::read_to_string(&mut child.0.stdout.take().unwrap(), &mut stdout)?;
		ensure!(!stdout.contains("\x1b[?1049h"));
	}
	println!(
		"PASS: raw/escaped pipes and files, EOF keyboard handoff, review, limits, terminal cleanup"
	);
	Ok(())
}
