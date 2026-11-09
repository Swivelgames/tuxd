//! PTY lifecycle, bounded input/output, screen state, and terminal restoration.
#[cfg(unix)]
use anyhow::Context;
use anyhow::{Result, ensure};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, SlavePty};
use std::{
	ffi::OsStr,
	io::{Read, Write},
	path::Path,
	sync::mpsc::{self, Receiver, RecvTimeoutError},
	thread,
	time::{Duration, Instant},
};

pub struct Session {
	child: Box<dyn Child + Send + Sync>,
	master: Box<dyn MasterPty + Send>,
	_slave: Box<dyn SlavePty + Send>,
	writer: Box<dyn Write + Send>,
	incoming: Receiver<std::io::Result<Vec<u8>>>,
	pub parser: vt100::Parser,
	pub output: Vec<u8>,
	_config: tempfile::TempDir,
	#[cfg(unix)]
	original: nix::sys::termios::Termios,
}

impl Session {
	pub fn command(binary: impl AsRef<OsStr>) -> CommandBuilder {
		CommandBuilder::new(binary)
	}

	pub fn new(binary: &Path, args: &[&OsStr], env: &[(&str, Option<&OsStr>)]) -> Result<Self> {
		let mut command = Self::command(binary);
		command.args(args);
		Self::spawn(command, env)
	}

	pub fn spawn(mut command: CommandBuilder, env: &[(&str, Option<&OsStr>)]) -> Result<Self> {
		let config = tempfile::tempdir()?;
		command.env("TERM", "xterm-256color");
		command.env("XDG_CONFIG_HOME", config.path());
		for key in ["TMUX", "TMUX_PANE", "SSH_CONNECTION", "SSH_TTY"] {
			command.env_remove(key);
		}
		for (key, value) in env {
			if let Some(value) = value {
				command.env(key, value);
			} else {
				command.env_remove(key);
			}
		}
		let pair = portable_pty::native_pty_system().openpty(PtySize {
			rows: 24,
			cols: 80,
			pixel_width: 0,
			pixel_height: 0,
		})?;
		#[cfg(unix)]
		let original = pair
			.master
			.get_termios()
			.context("PTY termios unavailable")?;
		let mut reader = pair.master.try_clone_reader()?;
		let writer = pair.master.take_writer()?;
		let child = pair.slave.spawn_command(command)?;
		let (sender, incoming) = mpsc::channel();
		thread::spawn(move || {
			let mut buffer = [0; 65536];
			loop {
				match reader.read(&mut buffer) {
					Ok(0) => break,
					Ok(count) => {
						if sender.send(Ok(buffer[..count].to_vec())).is_err() {
							break;
						}
					}
					Err(error) if error.raw_os_error() == Some(5) => break,
					Err(error) => {
						let _ = sender.send(Err(error));
						break;
					}
				}
			}
		});
		let mut session = Self {
			child,
			master: pair.master,
			_slave: pair.slave,
			writer,
			incoming,
			parser: vt100::Parser::new(24, 80, 0),
			output: Vec::new(),
			_config: config,
			#[cfg(unix)]
			original,
		};
		let deadline = Instant::now() + Duration::from_secs(5);
		while Instant::now() < deadline {
			session.drain()?;
			if session.text().contains("F6 focus") || !session.alive()? {
				break;
			}
		}
		Ok(session)
	}

	pub fn drain(&mut self) -> Result<()> {
		let deadline = Instant::now() + Duration::from_secs(3);
		loop {
			match self.incoming.recv_timeout(Duration::from_millis(150)) {
				Ok(bytes) => {
					let bytes = bytes?;
					self.parser.process(&bytes);
					self.output.extend(bytes);
				}
				Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => return Ok(()),
			}
			ensure!(Instant::now() < deadline, "Terminal did not become idle");
		}
	}

	pub fn send(&mut self, text: &str) -> Result<()> {
		self.writer.write_all(text.as_bytes())?;
		self.writer.flush()?;
		self.drain()
	}

	pub fn paste(&mut self, text: &str) -> Result<()> {
		self.send(&format!("\x1b[200~{text}\x1b[201~"))
	}

	pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
		self.parser.screen_mut().set_size(rows, cols);
		self.master.resize(PtySize {
			rows,
			cols,
			pixel_width: 0,
			pixel_height: 0,
		})?;
		self.drain()
	}

	pub fn alive(&mut self) -> Result<bool> {
		Ok(self.child.try_wait()?.is_none())
	}

	pub fn wait(&mut self) -> Result<u32> {
		let deadline = Instant::now() + Duration::from_secs(5);
		loop {
			if let Some(status) = self.child.try_wait()? {
				self.drain()?;
				return Ok(status.exit_code());
			}
			ensure!(Instant::now() < deadline, "Terminal child did not exit");
			self.drain()?;
		}
	}

	pub fn finish(&mut self, expected: u32) -> Result<()> {
		ensure!(self.wait()? == expected, "Unexpected terminal exit status");
		#[cfg(unix)]
		ensure!(
			self.master
				.get_termios()
				.context("Missing restored termios")?
				== self.original,
			"Terminal modes were not restored"
		);
		ensure!(
			self.output
				.windows(5)
				.filter(|bytes| *bytes == b"\x1b[<1u")
				.count() == 1,
			"Keyboard mode must be popped exactly once"
		);
		for sequence in [
			b"\x1b[?1049l".as_slice(),
			b"\x1b[?2004l",
			b"\x1b[?1000l",
			b"\x1b[?25h",
		] {
			ensure!(
				self.output.windows(sequence.len()).any(|s| s == sequence),
				"Missing terminal cleanup: {sequence:?}"
			);
		}
		Ok(())
	}

	pub fn row(&self, row: u16) -> String {
		let (_, cols) = self.parser.screen().size();
		(0..cols)
			.map(|col| {
				let cell = self.cell(row, col);
				if cell.contents().is_empty() {
					" "
				} else {
					cell.contents()
				}
			})
			.collect()
	}

	pub fn rows(&self) -> Vec<String> {
		(0..self.parser.screen().size().0)
			.map(|row| self.row(row))
			.collect()
	}

	pub fn text(&self) -> String {
		self.rows().join("\n")
	}

	pub fn cell(&self, row: u16, col: u16) -> &vt100::Cell {
		self.parser
			.screen()
			.cell(row, col)
			.expect("Test coordinate within terminal")
	}

	pub fn contains(&self, text: &str) -> Result<()> {
		ensure!(
			self.text().contains(text),
			"Missing {text:?}:\n{}",
			self.text()
		);
		Ok(())
	}

	pub fn excludes(&self, text: &str) -> Result<()> {
		ensure!(
			!self.text().contains(text),
			"Unexpected {text:?}:\n{}",
			self.text()
		);
		Ok(())
	}

	pub fn segment(&self, row: u16, col: usize, text: &str) -> Result<()> {
		ensure!(
			self.row(row)
				.chars()
				.skip(col)
				.take(text.chars().count())
				.collect::<String>()
				== text,
			"Wrong text on row {row}: {:?}",
			self.row(row)
		);
		Ok(())
	}

	pub fn click(&mut self, label: &str, min_col: usize) -> Result<()> {
		for direction in [64, 65] {
			for _ in 0..100 {
				for (row, text) in self.rows().iter().enumerate() {
					let suffix: String = text.chars().skip(min_col).collect();
					if let Some(index) = suffix.find(label) {
						let col = min_col + suffix[..index].chars().count();
						return self.send(&format!(
							"\x1b[<0;{};{}M\x1b[<0;{};{}m",
							col + 1,
							row + 1,
							col + 1,
							row + 1
						));
					}
				}
				let before = self.rows();
				self.send(&format!("\x1b[<{direction};70;15M"))?;
				if self.rows() == before {
					break;
				}
			}
		}
		anyhow::bail!("Missing control {label}:\n{}", self.text())
	}
}

impl Drop for Session {
	fn drop(&mut self) {
		if self.child.try_wait().ok().flatten().is_none() {
			let _ = self.child.kill();
			let _ = self.child.wait();
		}
	}
}
