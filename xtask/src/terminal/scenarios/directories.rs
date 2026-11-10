//! CLI directory opening and the sidebar's real keyboard/mouse integration.
use crate::{process::Runner, terminal::session::Session};
use anyhow::{Result, ensure};
use std::{fs, path::Path};

pub fn run(runner: &Runner, binary: &Path) -> Result<()> {
	let folder = tempfile::tempdir()?;
	fs::create_dir(folder.path().join("nested"))?;
	for name in [
		"alpha.tuxd.json",
		".hidden.tuxd.json",
		"nested/child.tuxd.json",
	] {
		fs::copy(
			runner.root.join("examples/welcome.tuxd.json"),
			folder.path().join(name),
		)?;
	}
	fs::write(folder.path().join("notes.txt"), "imported text")?;
	default_directory(binary, folder.path())?;

	let mut session = Session::new(binary, &[folder.path().as_os_str()], &[])?;
	session.contains("Open Files")?;
	session.contains("alpha.tuxd.json")?;
	session.excludes(".hidden.tuxd.json")?;
	session.excludes("notes.txt")?;
	session.send("\x1b[46;5u")?;
	session.contains(".hidden.tuxd.json")?;
	session.send("\x1b[46;5u\x06")?;
	session.contains("notes.txt")?;
	session.send("\x06G\r")?;
	session.contains("Terminal UX Designer")?;
	session.send("Z")?;
	ensure!(session.row(1).contains("ZTerminal UX Designer"));
	// Focus the tree and reopen the existing path; its edited buffer survives.
	session.send("\x1b[101;7uG\r")?;
	ensure!(session.row(1).contains("ZTerminal UX Designer"));
	session.send("\x1b[101;7u")?;
	session.resize(60, 24)?;
	session.contains("Open Files")?;
	session.send("\x1b")?;
	session.excludes("Open Files")?;
	session.contains("alpha.tuxd.json")?;
	// The title is the mouse toggle, with the overlay consuming its own rows.
	session.send("\x1b[<0;5;1M\x1b[<0;5;1m")?;
	session.contains("Open Files")?;
	session.send("\x1b[<0;5;1M\x1b[<0;5;1m")?;
	session.excludes("Open Files")?;
	// Switch directories through the actual picker, retaining the dirty document.
	session.send("\x1b[111;6u")?;
	session.contains("Open directory")?;
	session.paste(folder.path().join("nested").to_str().unwrap())?;
	session.send("\r")?;
	session.contains("child.tuxd.json")?;
	session.contains("alpha.tuxd.json")?;
	session.send("\x1b")?;
	session.excludes("Open Files")?;
	session.send("\x11")?;
	session.contains("Save changes before continuing?")?;
	session.send("d")?;
	session.finish(0)?;
	println!(
		"PASS: directory startup, sidebar navigation/filtering, duplicate buffers, slim overlay, directory picker"
	);
	Ok(())
}

fn default_directory(binary: &Path, folder: &Path) -> Result<()> {
	// No argument must discover the child's CWD, not the test runner's directory.
	let mut command = Session::command(binary);
	command.cwd(folder);
	let mut session = Session::spawn(command, &[])?;
	session.contains("Open Files")?;
	session.contains("alpha.tuxd.json")?;
	session.excludes(".hidden.tuxd.json")?;
	session.excludes("notes.txt")?;
	session.send("G\r")?;
	session.contains("Terminal UX Designer")?;
	session.send("\x11")?;
	session.finish(0)?;

	Ok(())
}
