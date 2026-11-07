mod ansi_import;
mod app;
mod cli;
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
	let command = cli::parse(std::env::args_os().skip(1).collect())?;
	let mut document_path = None;
	let mut startup_import = None;
	let mut diagnostic = None;
	match command {
		cli::Command::Help => {
			println!(
				"Terminal UX Designer\n\nUsage: terminal-ux-designer [document.tuxd.json]\n       terminal-ux-designer --raw INPUT\n       terminal-ux-designer --escaped INPUT\n\nINPUT is a file path or - for piped/redirected stdin. Import requires UTF-8, at most 16 MiB.\nImported content opens unsaved. F4 files · Ctrl+S save · Ctrl+Q quit · F1 shortcuts"
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
			let text = String::from_utf8(bytes)
				.map_err(|_| io::Error::other("Import requires UTF-8 text"))?;
			let import = ansi_import::parse(&text, mode).map_err(io::Error::other)?;
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
	app.document_path = document_path;
	if let Some((import, path)) = startup_import {
		if import.issues.is_empty() {
			app.load_document(import.document, path, true);
		} else {
			app.files.review = match path {
				Some(path) => import_review::Review::new(path),
				None => import_review::Review::stdin(),
			};
			app.file_dialog(files::Dialog::Review(import));
		}
	}
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
