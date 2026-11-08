//! Environment-selected transports. Workers prepare terminal bytes but never emit them.
use super::process::{copy_with, paste_with};
use super::{Capabilities, Driver, Error, Receipt, Transport, WriteOutput};

#[derive(Default)]
pub struct System;

fn transport(tmux: bool, ssh: bool, term: &str) -> Result<Transport, Error> {
	if term == "linux" || term == "dumb" {
		return Err(Error::UnsupportedTerminal);
	}
	Ok(if tmux {
		Transport::Tmux
	} else if ssh {
		Transport::Osc52
	} else {
		Transport::Desktop
	})
}

fn selected_transport() -> Result<Transport, Error> {
	transport(
		std::env::var_os("TMUX").is_some(),
		std::env::var_os("SSH_CONNECTION").is_some() || std::env::var_os("SSH_TTY").is_some(),
		&std::env::var("TERM").unwrap_or_default(),
	)
}

fn osc52(text: &str) -> Result<Vec<u8>, Error> {
	use base64::Engine;
	if text.len() > 100 * 1024 {
		return Err(Error::PayloadTooLarge);
	}
	let encoded = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
	Ok(format!("\x1b]52;c;{encoded}\x07").into_bytes())
}

impl Driver for System {
	fn capabilities(&self) -> Capabilities {
		let desktop =
			std::env::var_os("WAYLAND_DISPLAY").is_some() || std::env::var_os("DISPLAY").is_some();
		Capabilities {
			write: selected_transport()
				.ok()
				.filter(|transport| *transport != Transport::Desktop || desktop),
			// Existing reads use only desktop helpers, even when writes target SSH/tmux.
			read: desktop,
		}
	}

	fn write(&self, text: &str) -> Result<WriteOutput, Error> {
		match selected_transport()? {
			Transport::Tmux => {
				let name = format!("tuxd-{}", std::process::id());
				copy_with("tmux", &["load-buffer", "-w", "-b", &name, "-"], text)
					.map_err(|_| Error::TmuxWrite)?;
				return Ok(WriteOutput::Delivered(Receipt::Tmux));
			}
			Transport::Osc52 => return osc52(text).map(WriteOutput::Terminal),
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
				return Ok(WriteOutput::Delivered(Receipt::Desktop));
			}
		}
		Err(Error::DesktopWrite)
	}

	fn read(&self) -> Result<String, Error> {
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
		Err(Error::DesktopRead)
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
			Transport::Osc52
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
