mod app;
mod clipboard;
mod document;
mod editor;
mod layout;
mod panel;
mod panel_ui;
mod scrollbar;
mod styling;
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
				"Terminal UX Designer\n\nRun without arguments to edit.\nCtrl+Q quit · F1 help\n\nText is session-local; saving is not available yet."
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
		app.poll_clipboard();
		terminal.draw(|frame| ui::draw(frame, &mut app))?;
		if let Some(panic) = diagnostic {
			if panic {
				panic!("Intentional terminal cleanup diagnostic");
			}
			return Err(io::Error::other("Intentional terminal cleanup diagnostic"));
		}
		loop {
			if event::poll(std::time::Duration::from_millis(100))? {
				if app.handle(event::read()?) {
					return Ok(());
				}
				break;
			}
			if app.poll_clipboard() {
				break;
			}
		}
	}
}
