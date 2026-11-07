mod ansi_import;
mod app;
mod clipboard;
mod code_highlight;
mod document;
mod editor;
mod editor_clipboard;
mod file_picker;
mod files;
#[cfg(test)]
mod hardening;
mod import_review;
mod layout;
mod panel;
mod panel_ui;
mod persistence;
mod scrollbar;
mod shortcuts;
mod styling;
mod terminal;
mod ui;

use std::io;

use crossterm::event;

fn main() -> io::Result<()> {
	if std::env::args_os().len() > 2 {
		return Err(io::Error::other("Expected at most one document path"));
	}
	let mut document_path = None;
	let argument = std::env::args_os().nth(1);
	let diagnostic = match argument.as_deref() {
		None => None,
		Some(arg) if arg == "--diagnostic-error" => Some(false),
		Some(arg) if arg == "--diagnostic-panic" => Some(true),
		Some(arg) if arg == "--help" || arg == "-h" => {
			println!(
				"Terminal UX Designer\n\nUsage: terminal-ux-designer [document.tuxd.json]\nF4 files · Ctrl+S save · Ctrl+Q quit · F1 shortcuts"
			);
			return Ok(());
		}
		Some(arg) if !arg.as_encoded_bytes().starts_with(b"-") => {
			document_path = Some(std::path::PathBuf::from(arg));
			None
		}
		Some(arg) => {
			return Err(io::Error::other(format!(
				"Unknown argument: {}",
				arg.to_string_lossy()
			)));
		}
	};
	let loaded = document_path
		.as_deref()
		.map(persistence::load)
		.transpose()?;
	let _session = terminal::Session::enter()?;
	let mut terminal =
		ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(io::stdout()))?;
	let mut app = app::App::default();
	if let Some(document) = loaded {
		app.load_document(document, document_path.clone(), false);
	}
	app.document_path = document_path;
	if let Err(error) = app.shortcuts.load() {
		app.notice = Some(format!(
			"Shortcuts: {}; using defaults",
			crate::document::sanitize(&error)
		));
	}
	loop {
		app.poll_clipboard();
		terminal.draw(|frame| {
			ui::draw(frame, &mut app);
			files::draw(frame, &mut app);
			if let Some(settings) = &mut app.settings {
				settings.draw(frame);
			}
		})?;
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
			if app.poll_clipboard() || app.shortcuts.expire(std::time::Instant::now()) {
				break;
			}
		}
	}
}
