//! Unix-only contracts: redirected stdin with a controlling TTY, raw mode and paths.
use super::session::Session;
use crate::process::ManagedChild;
use crate::process::Runner;
use anyhow::{Context, Result, ensure};
use std::{
	ffi::OsString,
	fs,
	io::{Read, Write},
	os::{
		fd::{AsRawFd, FromRawFd},
		unix::{ffi::OsStringExt, process::CommandExt},
	},
	path::Path,
	process::{Command, Stdio},
	time::Duration,
};

pub fn stdin_helper() -> Result<()> {
	let args: Vec<_> = std::env::args_os().skip(2).collect();
	ensure!(args.len() >= 3, "stdin helper arguments missing");
	let input = if args[0] == "pipe" {
		let bytes = fs::read(&args[1])?;
		ensure!(bytes.len() <= 4096, "Pipe fixture exceeds atomic capacity");
		let mut fds = [0; 2];
		// Owned pipe ends are closed before exec except the end installed as stdin.
		ensure!(unsafe { libc::pipe(fds.as_mut_ptr()) } == 0, "pipe failed");
		let read = unsafe { fs::File::from_raw_fd(fds[0]) };
		let mut write = unsafe { fs::File::from_raw_fd(fds[1]) };
		for fd in [read.as_raw_fd(), write.as_raw_fd()] {
			ensure!(
				unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) } >= 0,
				"Could not mark pipe close-on-exec"
			);
		}
		write.write_all(&bytes)?;
		drop(write);
		read
	} else {
		fs::File::open(&args[1])?
	};
	Err(Command::new(&args[2])
		.args(&args[3..])
		.stdin(input)
		.exec()
		.into())
}

pub fn imported(binary: &Path, mode: &str, source: &Path, pipe: bool) -> Result<Session> {
	let mut command = Session::command(std::env::current_exe()?);
	command.arg("__stdin");
	command.arg(if pipe { "pipe" } else { "file" });
	command.arg(source);
	command.arg(binary);
	command.args([mode, "-"]);
	Session::spawn(command, &[])
}

pub fn paths(runner: &Runner, binary: &Path) -> Result<()> {
	let temp = tempfile::tempdir()?;
	let path = temp
		.path()
		.join(OsString::from_vec(b"document-\xff.tuxd.json".to_vec()));
	fs::copy(runner.root.join("examples/welcome.tuxd.json"), &path)?;
	let mut s = Session::new(binary, &[path.as_os_str()], &[])?;
	ensure!(s.row(1).contains("Terminal UX Designer"));
	s.send("\x11")?;
	s.finish(0)?;
	let fifo = temp.path().join("input.fifo");
	let name = std::ffi::CString::new(fifo.as_os_str().as_encoded_bytes())?;
	ensure!(
		unsafe { libc::mkfifo(name.as_ptr(), 0o600) } == 0,
		"mkfifo failed"
	);
	for source in [fifo, temp.path().join("missing")] {
		let mut child = ManagedChild::spawn(
			Command::new(binary)
				.arg(source)
				.stdout(Stdio::piped())
				.stderr(Stdio::piped()),
		)?;
		ensure!(!child.wait(Duration::from_secs(3))?.success());
		let mut stderr = String::new();
		child.0.stderr.take().unwrap().read_to_string(&mut stderr)?;
		ensure!(!stderr.contains("panicked"));
		let mut stdout = String::new();
		child.0.stdout.take().unwrap().read_to_string(&mut stdout)?;
		ensure!(!stdout.contains("\x1b[?1049h"));
	}
	Ok(())
}

pub fn no_controlling_terminal(binary: &Path) -> Result<()> {
	let pair = portable_pty::native_pty_system().openpty(portable_pty::PtySize {
		rows: 24,
		cols: 80,
		pixel_width: 0,
		pixel_height: 0,
	})?;
	let tty = pair
		.master
		.tty_name()
		.context("PTY slave path unavailable")?;
	let output = fs::OpenOptions::new().write(true).open(tty)?;
	let mut command = Command::new(binary);
	command
		.args(["--raw", "-"])
		.stdin(Stdio::piped())
		.stdout(output)
		.stderr(Stdio::piped());
	// setsid is async-signal-safe; no allocation or locking in the child hook.
	unsafe {
		command.pre_exec(|| {
			if libc::setsid() < 0 {
				return Err(std::io::Error::last_os_error());
			}
			Ok(())
		});
	}
	let mut child = ManagedChild::spawn(&mut command)?;
	child.0.stdin.take().unwrap().write_all(b"text")?;
	ensure!(!child.wait(Duration::from_secs(5))?.success());
	let mut stderr = String::new();
	child.0.stderr.take().unwrap().read_to_string(&mut stderr)?;
	ensure!(stderr.contains("controlling terminal"), "{stderr}");
	Ok(())
}

pub struct RawMode {
	fd: i32,
	original: libc::termios,
}
impl RawMode {
	pub fn stdin() -> Result<Self> {
		let fd = std::io::stdin().as_raw_fd();
		let mut original = std::mem::MaybeUninit::uninit();
		ensure!(
			unsafe { libc::tcgetattr(fd, original.as_mut_ptr()) } == 0,
			"tcgetattr failed"
		);
		let original = unsafe { original.assume_init() };
		let mut raw = original;
		unsafe { libc::cfmakeraw(&mut raw) };
		ensure!(
			unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) } == 0,
			"tcsetattr failed"
		);
		Ok(Self { fd, original })
	}

	pub fn read_ready(&self, buffer: &mut [u8]) -> Result<usize> {
		let mut poll = libc::pollfd {
			fd: self.fd,
			events: libc::POLLIN,
			revents: 0,
		};
		let ready = unsafe { libc::poll(&mut poll, 1, 100) };
		ensure!(ready >= 0, "poll failed");
		if ready == 0 {
			return Ok(0);
		}
		Ok(std::io::stdin().read(buffer)?)
	}
}
impl Drop for RawMode {
	fn drop(&mut self) {
		unsafe { libc::tcsetattr(self.fd, libc::TCSANOW, &self.original) };
	}
}
