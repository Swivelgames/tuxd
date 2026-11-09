//! A child process that is always reaped, including on failed checks.
use anyhow::{Result, ensure};
use std::{
	process::{Child, Command},
	thread,
	time::{Duration, Instant},
};

pub struct ManagedChild(pub Child);
impl ManagedChild {
	pub fn spawn(command: &mut Command) -> Result<Self> {
		Ok(Self(command.spawn()?))
	}

	pub fn wait(&mut self, timeout: Duration) -> Result<std::process::ExitStatus> {
		let deadline = Instant::now() + timeout;
		loop {
			if let Some(status) = self.0.try_wait()? {
				return Ok(status);
			}
			ensure!(Instant::now() < deadline, "Child process timed out");
			thread::sleep(Duration::from_millis(20));
		}
	}
}
impl Drop for ManagedChild {
	fn drop(&mut self) {
		if self.0.try_wait().ok().flatten().is_none() {
			let _ = self.0.kill();
			let _ = self.0.wait();
		}
	}
}
