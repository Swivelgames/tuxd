use std::{
	io::Write,
	process::{Command, Stdio},
	thread,
	time::{Duration, Instant},
};

/// Forwarded transports cannot acknowledge delivery to the client clipboard.
pub enum Copied {
	Desktop,
	Tmux,
	Terminal(Vec<u8>),
}

#[derive(Debug, PartialEq, Eq)]
enum Transport {
	Desktop,
	Tmux,
	Terminal,
}
fn transport(tmux: bool, ssh: bool, term: &str) -> Result<Transport, String> {
	if term == "linux" || term == "dumb" {
		return Err("Copy failed: this terminal has no supported clipboard transport; export the Code output instead".into());
	}
	Ok(if tmux {
		Transport::Tmux
	} else if ssh {
		Transport::Terminal
	} else {
		Transport::Desktop
	})
}
fn osc52(text: &str) -> Result<Vec<u8>, String> {
	use base64::Engine;
	// Never truncate or split: each OSC 52 write replaces the clipboard.
	if text.len() > 100 * 1024 {
		return Err(
			"Copy failed: terminal clipboard payload exceeds 100 KiB; export instead".into(),
		);
	}
	let encoded = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
	Ok(format!("\x1b]52;c;{encoded}\x07").into_bytes())
}

pub fn copy(text: &str) -> Result<Copied, String> {
	let selected = transport(
		std::env::var_os("TMUX").is_some(),
		std::env::var_os("SSH_CONNECTION").is_some() || std::env::var_os("SSH_TTY").is_some(),
		&std::env::var("TERM").unwrap_or_default(),
	)?;
	match selected {
		Transport::Tmux => {
			// stdin avoids shell quoting and argument-size limits. A named buffer
			// avoids replacing unrelated tmux history. -w requests client forwarding.
			let name = format!("tuxd-{}", std::process::id());
			copy_with("tmux", &["load-buffer", "-w", "-b", &name, "-"], text).map_err(|_| {
				"Copy failed: tmux clipboard forwarding failed; export instead".to_owned()
			})?;
			return Ok(Copied::Tmux);
		}
		Transport::Terminal => return osc52(text).map(Copied::Terminal),
		Transport::Desktop => (),
	}
	let mut helpers: Vec<(&str, &[&str])> = Vec::new();
	if std::env::var_os("WAYLAND_DISPLAY").is_some() {
		helpers.push(("wl-copy", &["--type", "text/plain;charset=utf-8"]));
	}
	if std::env::var_os("DISPLAY").is_some() {
		helpers.push(("xclip", &["-selection", "clipboard", "-in"]));
		helpers.push(("xsel", &["--clipboard", "--input"]));
	}
	for (program, args) in helpers {
		if copy_with(program, args, text).is_ok() {
			return Ok(Copied::Desktop);
		}
	}
	Err(
		"Copy failed: use wl-copy on Wayland or xclip/xsel on X11; check your display connection."
			.into(),
	)
}

/// Called by the UI thread so the OSC cannot interleave with a rendered frame.
pub fn finish(result: Copied, ansi: bool) -> Result<String, String> {
	let payload = if ansi { "escaped ANSI" } else { "selection" };
	match result {
		Copied::Desktop => Ok(format!("Copied {payload} to clipboard")),
		Copied::Tmux => Ok(format!(
			"Sent {payload} via tmux; clipboard delivery unconfirmed"
		)),
		Copied::Terminal(bytes) => {
			let mut stdout = std::io::stdout().lock();
			stdout
				.write_all(&bytes)
				.and_then(|_| stdout.flush())
				.map_err(|e| format!("Copy failed: {e}"))?;
			Ok(format!(
				"Sent {payload} via OSC 52; clipboard delivery unconfirmed"
			))
		}
	}
}

fn copy_with(program: &str, args: &[&str], text: &str) -> Result<(), ()> {
	let mut child = Command::new(program)
		.args(args)
		.stdin(Stdio::piped())
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.spawn()
		.map_err(|_| ())?;
	let mut stdin = child.stdin.take().ok_or(())?;
	let bytes = text.as_bytes().to_vec();
	let (sent, written) = std::sync::mpsc::channel();
	thread::spawn(move || {
		let _ = sent.send(stdin.write_all(&bytes));
	});
	let deadline = Instant::now() + Duration::from_secs(3);
	let result = loop {
		match child.try_wait() {
			Ok(Some(status)) => break if status.success() { Ok(()) } else { Err(()) },
			Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
			_ => {
				let _ = child.kill();
				let _ = child.wait();
				break Err(());
			}
		}
	};
	// On timeout, do not wait on a pipe that an ill-behaved descendant may still hold.
	result?;
	written
		.recv_timeout(deadline.saturating_duration_since(Instant::now()))
		.map_err(|_| ())?
		.map_err(|_| ())
}

/// Read on a worker thread; terminal bracketed paste remains available without helpers.
pub fn paste() -> Result<String, String> {
	let mut helpers: Vec<(&str, &[&str])> = Vec::new();
	if std::env::var_os("WAYLAND_DISPLAY").is_some() {
		helpers.push(("wl-paste", &["--no-newline"]));
	}
	if std::env::var_os("DISPLAY").is_some() {
		helpers.push(("xclip", &["-selection", "clipboard", "-out"]));
		helpers.push(("xsel", &["--clipboard", "--output"]));
	}
	for (program, args) in helpers {
		if let Ok(text) = paste_with(program, args) {
			return Ok(text);
		}
	}
	Err(
		"Paste failed: use terminal paste, or install wl-paste (Wayland) / xclip or xsel (X11)."
			.into(),
	)
}

fn paste_with(program: &str, args: &[&str]) -> Result<String, ()> {
	use std::io::Read;
	const LIMIT: usize = 64 * 1024;
	let mut child = Command::new(program)
		.args(args)
		.stdin(Stdio::null())
		.stdout(Stdio::piped())
		.stderr(Stdio::null())
		.spawn()
		.map_err(|_| ())?;
	let stdout = child.stdout.take().ok_or(())?;
	let (sender, receiver) = std::sync::mpsc::channel();
	thread::spawn(move || {
		let mut bytes = Vec::new();
		let result = stdout
			.take((LIMIT + 1) as u64)
			.read_to_end(&mut bytes)
			.map_err(|_| ())
			.and_then(|_| {
				if bytes.len() > LIMIT {
					Err(())
				} else {
					String::from_utf8(bytes).map_err(|_| ())
				}
			});
		let _ = sender.send(result);
	});
	let deadline = Instant::now() + Duration::from_secs(3);
	loop {
		match child.try_wait() {
			Ok(Some(status)) if status.success() => {
				return receiver
					.recv_timeout(deadline.saturating_duration_since(Instant::now()))
					.map_err(|_| ())?;
			}
			Ok(Some(_)) => return Err(()),
			Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
			_ => {
				let _ = child.kill();
				let _ = child.wait();
				return Err(());
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn routing_respects_remote_client_and_console_limitations() {
		assert_eq!(
			transport(true, true, "tmux-256color").unwrap(),
			Transport::Tmux
		);
		assert_eq!(
			transport(false, true, "xterm-256color").unwrap(),
			Transport::Terminal
		);
		assert_eq!(
			transport(false, false, "xterm-256color").unwrap(),
			Transport::Desktop
		);
		assert!(transport(false, true, "linux").is_err());
		assert!(transport(false, false, "dumb").is_err());
	}
	#[test]
	fn osc_payload_roundtrips_without_truncation_or_control_injection() {
		use base64::Engine;
		for text in [
			String::new(),
			"界\\x1b[31m\n\x1b]52;c;evil\x07".into(),
			"x".repeat(100 * 1024),
		] {
			let bytes = osc52(&text).unwrap();
			assert_eq!(&bytes[..7], b"\x1b]52;c;");
			assert_eq!(*bytes.last().unwrap(), 7);
			assert_eq!(
				base64::engine::general_purpose::STANDARD
					.decode(&bytes[7..bytes.len() - 1])
					.unwrap(),
				text.as_bytes()
			);
		}
		assert!(osc52(&"x".repeat(100 * 1024 + 1)).is_err());
	}
}
