use crate::process::ManagedChild;
use anyhow::{Context, Result, ensure};
use std::{
	ffi::{OsStr, OsString},
	fs,
	io::{BufRead, BufReader},
	path::Path,
	process::{Command, Stdio},
	sync::mpsc,
	thread,
	time::Duration,
};

pub struct Display {
	_server: ManagedChild,
	number: String,
	config: std::path::PathBuf,
}
impl Display {
	pub fn new(config: &Path, width: u16, height: u16) -> Result<Self> {
		crate::check::linux()?;
		let mut server = ManagedChild::spawn(
			Command::new("Xvfb")
				.args([
					"-displayfd",
					"1",
					"-screen",
					"0",
					&format!("{width}x{height}x24"),
					"-nolisten",
					"tcp",
					"-ac",
				])
				.stdout(Stdio::piped())
				.stderr(Stdio::null()),
		)?;
		let stdout = server.0.stdout.take().unwrap();
		let (sender, receiver) = mpsc::channel();
		thread::spawn(move || {
			let mut line = String::new();
			let result = BufReader::new(stdout).read_line(&mut line).map(|_| line);
			let _ = sender.send(result);
		});
		let number = receiver
			.recv_timeout(Duration::from_secs(5))
			.context("Xvfb startup timed out")??;
		ensure!(number.trim().parse::<u32>().is_ok(), "Invalid Xvfb display");
		Ok(Self {
			_server: server,
			number: format!(":{}", number.trim()),
			config: config.to_owned(),
		})
	}

	pub fn command(&self, program: impl AsRef<OsStr>) -> Command {
		let mut command = Command::new(program);
		for key in [
			"WAYLAND_DISPLAY",
			"TMUX",
			"TMUX_PANE",
			"SSH_CONNECTION",
			"SSH_TTY",
			"XAUTHORITY",
		] {
			command.env_remove(key);
		}
		command
			.env("DISPLAY", &self.number)
			.env("XDG_CONFIG_HOME", &self.config)
			.env("LIBGL_ALWAYS_SOFTWARE", "1")
			.env("WINIT_UNIX_BACKEND", "x11");
		command
	}
}

pub struct Window<'a> {
	pub app: ManagedChild,
	display: &'a Display,
	id: String,
}
impl<'a> Window<'a> {
	pub fn open(
		display: &'a Display,
		config: &Path,
		args: &[OsString],
		log: &Path,
	) -> Result<Self> {
		let log = fs::File::create(log)?;
		let app = ManagedChild::spawn(
			display
				.command("alacritty")
				.arg("--config-file")
				.arg(config)
				.args(["--title", "tuxd-isolated-check", "-e"])
				.args(args)
				.stdout(log.try_clone()?)
				.stderr(log),
		)?;
		let mut id = None;
		for _ in 0..60 {
			let result = display
				.command("xdotool")
				.args(["search", "--onlyvisible", "--class", "Alacritty"])
				.output()?;
			if result.status.success() {
				id = String::from_utf8(result.stdout)?
					.lines()
					.next()
					.map(str::to_owned);
				break;
			}
			thread::sleep(Duration::from_millis(100));
		}
		let window = Self {
			app,
			display,
			id: id.context("Alacritty did not start; see retained log")?,
		};
		window.xdo(&["windowfocus", "--sync", &window.id])?;
		thread::sleep(Duration::from_millis(700));
		Ok(window)
	}

	pub fn xdo(&self, args: &[&str]) -> Result<String> {
		let output = self.display.command("xdotool").args(args).output()?;
		ensure!(
			output.status.success(),
			"xdotool failed: {}",
			String::from_utf8_lossy(&output.stderr)
		);
		Ok(String::from_utf8(output.stdout)?.trim().into())
	}

	pub fn keys(&self, keys: &[&str]) -> Result<()> {
		let mut args = vec!["key", "--clearmodifiers"];
		args.extend(keys);
		self.xdo(&args)?;
		thread::sleep(Duration::from_millis(250));
		Ok(())
	}

	pub fn capture(&self, path: &Path) -> Result<()> {
		ensure!(
			self.display
				.command("import")
				.args(["-window", &self.id])
				.arg(path)
				.status()?
				.success(),
			"Image capture failed"
		);
		Ok(())
	}

	pub fn resize(&self, width: u16, height: u16) -> Result<()> {
		self.xdo(&[
			"windowsize",
			"--sync",
			&self.id,
			&width.to_string(),
			&height.to_string(),
		])?;
		thread::sleep(Duration::from_millis(200));
		Ok(())
	}
}
