use std::{
	io::{self, IsTerminal},
	sync::atomic::{AtomicBool, Ordering},
};

use crossterm::{
	cursor::Show,
	event::{
		DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
		EnableFocusChange, EnableMouseCapture, KeyboardEnhancementFlags,
		PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
	},
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

pub struct Session;

static KEYBOARD_PUSHED: AtomicBool = AtomicBool::new(false);

impl Session {
	pub fn enter() -> io::Result<Self> {
		if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
			return Err(io::Error::other(
				"Run this application in an interactive terminal",
			));
		}
		// Colors are document content in this editor, even when NO_COLOR is set.
		crossterm::style::force_color_output(true);
		let previous = std::panic::take_hook();
		std::panic::set_hook(Box::new(move |info| {
			restore();
			previous(info);
		}));
		// Construct the guard first so partial setup failures also restore the terminal.
		let session = Self;
		enable_raw_mode()?;
		execute!(
			io::stdout(),
			EnterAlternateScreen,
			EnableBracketedPaste,
			EnableMouseCapture,
			EnableFocusChange
		)?;
		KEYBOARD_PUSHED.store(true, Ordering::SeqCst);
		execute!(
			io::stdout(),
			PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
		)?;
		Ok(session)
	}
}

fn restore() {
	// Panic hook and Drop can both run; pop the keyboard stack only once.
	if KEYBOARD_PUSHED.swap(false, Ordering::SeqCst) {
		let _ = execute!(io::stdout(), PopKeyboardEnhancementFlags);
	}
	// Attempt each restoration even if an earlier write fails.
	let _ = execute!(io::stdout(), DisableMouseCapture, DisableFocusChange);
	let _ = execute!(io::stdout(), DisableBracketedPaste);
	let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
	let _ = disable_raw_mode();
}

impl Drop for Session {
	fn drop(&mut self) {
		restore();
	}
}
