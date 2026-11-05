mod app;
mod document;
mod editor;
mod terminal;
mod ui;

use std::io;

use crossterm::event;

fn main() -> io::Result<()> {
	let diagnostic = match std::env::args().nth(1).as_deref() {
		None => None,
		Some("--diagnostic-error") => Some(false),
		Some("--diagnostic-panic") => Some(true),
		Some("--help" | "-h") => {
			println!(
				"Terminal UX Designer\n\nRun without arguments to edit.\nCtrl+Q quit · F1 help\n\nSlice 1: text is session-local; saving is not available yet."
			);
			return Ok(());
		}
		Some(arg) => return Err(io::Error::other(format!("Unknown argument: {arg}"))),
	};
	let _session = terminal::Session::enter()?;
	let mut terminal =
		ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(io::stdout()))?;
	let mut app = app::App::default();
	loop {
		terminal.draw(|frame| ui::draw(frame, &mut app))?;
		if let Some(panic) = diagnostic {
			if panic {
				panic!("Intentional terminal cleanup diagnostic");
			}
			return Err(io::Error::other("Intentional terminal cleanup diagnostic"));
		}
		if app.handle(event::read()?) {
			return Ok(());
		}
	}
}
