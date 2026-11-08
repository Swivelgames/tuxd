use terminal_ux_designer::libraries;
mod adapters;
mod app;
mod cli;

mod components;
mod configuration;
mod features;
mod files;
#[cfg(test)]
mod hardening;
mod panel;
mod persistence;
mod session;
mod settings;
mod shortcuts;
mod terminal;
#[cfg(test)]
mod test_clipboard;
mod theme;
mod workflows;
mod workspace;

use std::io;

use crossterm::event;

fn main() -> io::Result<()> {
	let command = cli::parse(std::env::args_os().skip(1).collect())?;
	let mut document_path = None;
	let mut startup_import = None;
	let mut diagnostic = None;
	match command {
		cli::Command::Help => {
			println!(
				"Terminal UX Designer\n\nUsage: terminal-ux-designer [document.tuxd.json]\n       terminal-ux-designer --raw INPUT\n       terminal-ux-designer --escaped INPUT\n\nINPUT is a file path or - for piped/redirected stdin. Import requires UTF-8, at most 16 MiB.\nImported content opens unsaved. F4 files · Ctrl+S save · Ctrl+Q quit · F1 settings"
			);
			return Ok(());
		}
		cli::Command::Edit(path) => document_path = path,
		cli::Command::Diagnostic(panic) => diagnostic = Some(panic),
		cli::Command::Import { mode, source } => {
			let stdin = source.as_os_str() == "-";
			let bytes = if stdin {
				cli::read_stdin()?
			} else {
				persistence::read_file(&source)?
			};
			let import = workflows::parse_bytes(bytes, mode).map_err(io::Error::other)?;
			startup_import = Some((import, if stdin { None } else { Some(source) }));
		}
	}
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
	if let Some((import, path)) = startup_import {
		let source = path
			.map(workflows::Source::File)
			.unwrap_or(workflows::Source::Stdin);
		app.import_document(import, source);
	}
	app.load_configuration();
	loop {
		app.poll_clipboard();
		terminal.draw(|frame| app.draw(frame))?;
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

#[cfg(test)]
mod test_editor;
