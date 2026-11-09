//! File-backed clipboard fixture and executable adapter; never the desktop clipboard.
use anyhow::{Context, Result, ensure};
use std::{
	fs,
	io::{Read, Write},
	path::PathBuf,
};

pub struct Clipboard {
	pub directory: tempfile::TempDir,
	pub path: PathBuf,
}
impl Clipboard {
	pub fn new(initial: &str) -> Result<Self> {
		let directory = tempfile::tempdir()?;
		let path = directory.path().join("clipboard.txt");
		fs::write(&path, initial)?;
		for name in ["xclip", "xsel"] {
			fs::copy(
				std::env::current_exe()?,
				directory
					.path()
					.join(format!("{name}{}", std::env::consts::EXE_SUFFIX)),
			)?;
		}
		Ok(Self { directory, path })
	}

	pub fn env(&self) -> Vec<(&str, Option<&std::ffi::OsStr>)> {
		vec![
			("PATH", Some(self.directory.path().as_os_str())),
			("DISPLAY", Some(std::ffi::OsStr::new(":test"))),
			("WAYLAND_DISPLAY", None),
			("TUXD_TEST_CLIPBOARD", Some(self.path.as_os_str())),
		]
	}

	pub fn fail(&self) -> Result<()> {
		fs::write(self.directory.path().join("fail"), "")?;
		Ok(())
	}
}

/// The test executable doubles as an isolated clipboard helper; no shell scripts.
pub fn helper() -> Result<bool> {
	let exe = std::env::current_exe()?;
	let name = exe.file_stem().unwrap_or_default().to_string_lossy();
	if name != "xclip" && name != "xsel" {
		return Ok(false);
	}
	ensure!(
		name != "xsel" && !exe.parent().unwrap().join("fail").exists(),
		"Simulated clipboard failure"
	);
	let path = PathBuf::from(
		std::env::var_os("TUXD_TEST_CLIPBOARD").context("Clipboard fixture missing")?,
	);
	if std::env::args().any(|arg| arg == "-out") {
		std::io::stdout().write_all(&fs::read(path)?)?;
	} else {
		let mut bytes = Vec::new();
		std::io::stdin().read_to_end(&mut bytes)?;
		fs::write(path, bytes)?;
	}
	Ok(true)
}
