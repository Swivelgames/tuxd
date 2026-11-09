//! A private tmux server whose lifetime is scoped to one check.
use anyhow::{Result, ensure};
use std::{path::PathBuf, process::Command};

pub struct Tmux {
	_directory: tempfile::TempDir,
	pub socket: PathBuf,
}
impl Tmux {
	pub fn new() -> Result<Self> {
		let directory = tempfile::tempdir()?;
		let socket = directory.path().join("socket");
		Ok(Self {
			_directory: directory,
			socket,
		})
	}

	pub fn command(&self) -> Command {
		let mut command = Command::new("tmux");
		command.arg("-S").arg(&self.socket);
		command
	}

	pub fn run(&self, args: &[&str]) -> Result<()> {
		ensure!(self.command().args(args).status()?.success(), "tmux failed");
		Ok(())
	}
}
impl Drop for Tmux {
	fn drop(&mut self) {
		let _ = self
			.command()
			.arg("kill-server")
			.stdout(std::process::Stdio::null())
			.stderr(std::process::Stdio::null())
			.status();
	}
}
