//! Shared wording for transport facts; callers choose the payload and edit policy.
use crate::libraries::clipboard::{Error, Receipt};

pub fn copied(result: Result<Receipt, Error>, payload: &str) -> String {
	match result {
		Ok(Receipt::Desktop) => format!("Copied {payload} to clipboard"),
		Ok(Receipt::Tmux) => format!("Sent {payload} via tmux; clipboard delivery unconfirmed"),
		Ok(Receipt::Osc52) => format!("Sent {payload} via OSC 52; clipboard delivery unconfirmed"),
		Err(error) => copy_error(error),
	}
}

fn copy_error(error: Error) -> String {
	match error {
		Error::UnsupportedTerminal => "Copy failed: this terminal has no supported clipboard transport; export the Code output instead".into(),
		Error::PayloadTooLarge => "Copy failed: terminal clipboard payload exceeds 100 KiB; export instead".into(),
		Error::TmuxWrite => "Copy failed: tmux clipboard forwarding failed; export instead".into(),
		Error::DesktopWrite => "Copy failed: use wl-copy on Wayland or xclip/xsel on X11; check your display connection.".into(),
		Error::TerminalOutput(error) => format!("Copy failed: {error}"),
		_ => "Clipboard copy failed".into(),
	}
}

pub fn paste_error(error: Error) -> String {
	match error {
		Error::DesktopRead => {
			"Paste failed: use terminal paste, or install wl-paste (Wayland) / xclip or xsel (X11)."
				.into()
		}
		_ => "Clipboard paste failed".into(),
	}
}
