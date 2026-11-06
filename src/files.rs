//! File actions and modal state. Candidate documents are validated before replacement.
use crate::{
	ansi_import::{self, Import, Mode},
	app::{App, Overlay},
	document::{Document, sanitize},
	editor::Editor,
	layout::Layout,
	persistence,
	scrollbar::{self, Bar, Target},
	styling,
};
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::{
	Frame,
	layout::Rect,
	style::{Color, Modifier, Style},
	widgets::{Block, BorderType, Clear, Paragraph},
};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
	New,
	Open,
	Save,
	SaveAs,
	ImportRaw,
	ImportEscaped,
	ExportRaw,
	ExportEscaped,
	Quit,
}
pub const MENU: [(Action, &str); 7] = [
	(Action::New, "New document          Ctrl+N"),
	(Action::Open, "Open document         Ctrl+O"),
	(Action::Save, "Save                  Ctrl+S"),
	(Action::SaveAs, "Save as         Ctrl+Shift+S"),
	(Action::ImportEscaped, "Import"),
	(Action::ExportEscaped, "Export                Ctrl+E"),
	(Action::Quit, "Quit                  Ctrl+Q"),
];

pub enum Pending {
	Replace {
		document: Document,
		path: Option<PathBuf>,
		imported: bool,
	},
	Quit,
}

pub enum Dialog {
	Menu {
		index: usize,
	},
	Path {
		action: Action,
		picker: Box<crate::file_picker::Picker>,
	},
	Dirty,
	Overwrite {
		action: Action,
		path: PathBuf,
	},
	Review(Import),
}

pub struct Files {
	pub review: crate::import_review::Review,
	origin: Option<PathBuf>,
	pub saved: Option<Document>,
	pub dialog: Option<Dialog>,
	pending: Option<Pending>,
	pub error: Option<String>,
	pub scroll: usize,
	pub viewport: usize,
	follow_focus: bool,
	pub bar: Option<Bar>,
	pub buttons: Vec<(Rect, usize)>,
	choice: usize,
	drag: Option<usize>,
	exit: bool,
}
impl Default for Files {
	fn default() -> Self {
		Self {
			review: Default::default(),
			origin: None,
			saved: Some(Document::default()),
			dialog: None,
			pending: None,
			error: None,
			scroll: 0,
			viewport: 1,
			follow_focus: true,
			bar: None,
			buttons: Vec::new(),
			choice: 0,
			drag: None,
			exit: false,
		}
	}
}

impl App {
	pub fn dirty(&self) -> bool {
		self.files.saved.as_ref() != Some(&self.editor.document)
	}

	pub fn load_document(&mut self, document: Document, path: Option<PathBuf>, imported: bool) {
		self.editor_open = true;
		self.files.saved = if imported {
			None
		} else {
			Some(document.clone())
		};
		let wrap = self.editor.wrap;
		self.editor = Editor::default();
		self.editor.wrap = wrap;
		self.editor.document = document;
		self.files.origin = path.clone();
		self.document_path = if imported { None } else { path };
		self.focus = 0;
		self.code_top = 0;
		self.code_left = 0;
		self.dragging = false;
		self.scrollbar_drag = None;
		self.panel.gauge_drag = None;
		self.notice = Some(
			if imported {
				"ANSI imported · unsaved document"
			} else {
				"Document opened"
			}
			.into(),
		);
	}

	fn file_dialog(&mut self, dialog: Dialog) {
		self.files.dialog = Some(dialog);
		self.files.choice = if matches!(self.files.dialog, Some(Dialog::Dirty)) {
			2
		} else {
			0
		};
		self.files.error = None;
		self.files.scroll = 0;
		self.files.follow_focus = true;
		self.files.bar = None;
		self.files.drag = None;
		self.dragging = false;
		self.scrollbar_drag = None;
		self.panel.gauge_drag = None;
		self.editor.break_group();
	}
	fn cancel_file(&mut self) {
		self.files.dialog = None;
		self.files.pending = None;
		self.files.error = None;
		self.files.drag = None;
	}
	fn finish_pending(&mut self) {
		self.files.dialog = None;
		if let Some(pending) = self.files.pending.take() {
			match pending {
				Pending::Replace {
					document,
					path,
					imported,
				} => self.load_document(document, path, imported),
				Pending::Quit => self.files.exit = true,
			}
		}
	}
	fn replace_or_prompt(&mut self, pending: Pending) {
		self.files.pending = Some(pending);
		if self.dirty() {
			self.file_dialog(Dialog::Dirty);
		} else {
			self.finish_pending();
		}
	}
	fn path_dialog(&mut self, action: Action) {
		let text = match action {
			Action::Save | Action::SaveAs => self
				.document_path
				.as_ref()
				.map(|p| p.to_string_lossy().into_owned())
				.unwrap_or_default(),
			_ => String::new(),
		};
		let picker = crate::file_picker::Picker::new(
			action,
			&text,
			self.document_path
				.as_deref()
				.or(self.files.origin.as_deref()),
		);
		self.file_dialog(Dialog::Path {
			action,
			picker: Box::new(picker),
		});
	}
	pub fn file_action(&mut self, action: Action) {
		self.files.error = None;
		match action {
			Action::New => self.replace_or_prompt(Pending::Replace {
				document: Document::default(),
				path: None,
				imported: false,
			}),
			Action::Quit => self.replace_or_prompt(Pending::Quit),
			Action::Save => {
				if let Some(path) = self.document_path.clone() {
					self.write_file(Action::Save, path, true);
				} else {
					self.path_dialog(Action::SaveAs);
				}
			}
			_ => self.path_dialog(action),
		}
	}
	fn file_failure(&mut self, action: Action, path: PathBuf, error: impl std::fmt::Display) {
		let text = path.to_string_lossy().into_owned();
		let picker = crate::file_picker::Picker::new(
			action,
			&text,
			self.document_path
				.as_deref()
				.or(self.files.origin.as_deref()),
		);
		self.file_dialog(Dialog::Path {
			action,
			picker: Box::new(picker),
		});
		self.files.error = Some(format!("{error}"));
		self.files.follow_focus = false;
		self.files.scroll = usize::MAX;
	}
	fn write_file(&mut self, action: Action, path: PathBuf, replace: bool) {
		let export = matches!(action, Action::ExportRaw | Action::ExportEscaped);
		let result = (|| {
			if export && let Some(current) = &self.document_path {
				let same = path == *current
					|| path
						.canonicalize()
						.ok()
						.zip(current.canonicalize().ok())
						.is_some_and(|(a, b)| a == b);
				if same {
					return Err(std::io::Error::other(
						"Choose an export destination other than the open document",
					));
				}
			}
			let bytes = match action {
				Action::ExportRaw => styling::ansi(&self.editor.document).into_bytes(),
				Action::ExportEscaped => {
					styling::clipboard_text(&self.editor.document).into_bytes()
				}
				_ => persistence::encode(&self.editor.document)?,
			};
			persistence::write_atomic(&path, &bytes, replace)
		})();
		match result {
			Ok(()) => {
				if self.document_path.is_none() && self.files.origin.is_none() {
					self.files.origin = Some(path.clone());
				}
				if !export {
					self.document_path = Some(path);
					self.files.saved = Some(self.editor.document.clone());
				}
				self.files.error = None;
				self.notice = Some(
					if export {
						"ANSI exported"
					} else {
						"Document saved"
					}
					.into(),
				);
				self.finish_pending();
			}
			Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && !replace => {
				self.file_dialog(Dialog::Overwrite { action, path })
			}
			Err(error) => self.file_failure(action, path, error),
		}
	}
	fn submit_path(&mut self, action: Action, text: String) {
		if text.is_empty() {
			self.files.error = Some("Enter a file path".into());
			return;
		}
		let mut path = PathBuf::from(text);
		if matches!(action, Action::Save | Action::SaveAs) && path.extension().is_none() {
			path.set_extension("tuxd.json");
		}
		if action == Action::Open
			&& !path
				.file_name()
				.is_some_and(|name| name.to_string_lossy().ends_with(".tuxd.json"))
		{
			let mut picker = crate::file_picker::Picker::new(
				Action::ImportEscaped,
				&path.to_string_lossy(),
				self.document_path.as_deref(),
			);
			picker.offer_import();
			self.file_dialog(Dialog::Path {
				action: Action::ImportEscaped,
				picker: Box::new(picker),
			});
			return;
		}
		match action {
			Action::Open => match persistence::load(&path) {
				Ok(document) => self.replace_or_prompt(Pending::Replace {
					document,
					path: Some(path),
					imported: false,
				}),
				Err(error) => self.file_failure(action, path, error),
			},
			Action::ImportRaw | Action::ImportEscaped => {
				let result = persistence::read_file(&path)
					.map_err(|e| e.to_string())
					.and_then(|bytes| {
						String::from_utf8(bytes).map_err(|_| "Import requires UTF-8 text".into())
					})
					.and_then(|source| {
						ansi_import::parse(
							&source,
							if action == Action::ImportRaw {
								Mode::Raw
							} else {
								Mode::Escaped
							},
						)
					});
				match result {
					Ok(import) if !import.issues.is_empty() => {
						self.files.review = crate::import_review::Review::new(path.clone());
						self.file_dialog(Dialog::Review(import))
					}
					Ok(import) => self.replace_or_prompt(Pending::Replace {
						document: import.document,
						path: Some(path.clone()),
						imported: true,
					}),
					Err(error) => self.file_failure(action, path, error),
				}
			}
			_ => self.write_file(action, path, false),
		}
	}
	fn activate_file(&mut self, choice: usize) {
		let Some(dialog) = self.files.dialog.take() else {
			return;
		};
		match dialog {
			Dialog::Menu { .. } => self.file_action(MENU[choice.min(MENU.len() - 1)].0),

			Dialog::Dirty => match choice {
				0 => self.file_action(Action::Save),
				1 => self.finish_pending(),
				_ => self.cancel_file(),
			},
			Dialog::Overwrite { action, path } => {
				if choice == 0 {
					self.write_file(action, path, true);
				} else {
					self.cancel_file();
				}
			}
			Dialog::Review(import) => {
				if choice == 0 {
					self.replace_or_prompt(Pending::Replace {
						document: import.document,
						path: Some(self.files.review.source_path().to_path_buf()),
						imported: true,
					});
				} else {
					self.cancel_file();
				}
			}
			Dialog::Path { action, picker } => {
				let text = picker.destination().to_string_lossy().into_owned();
				self.files.dialog = Some(Dialog::Path { action, picker });
				self.submit_path(action, text);
			}
		}
	}

	/// Some means the file workflow consumed this event; the bool requests exit.
	pub fn file_event(&mut self, event: &Event) -> Option<bool> {
		if self.files.dialog.is_none() {
			if self.overlay != Overlay::None {
				return None;
			}
			let Event::Key(key) = event else {
				return None;
			};
			if key.kind == KeyEventKind::Release {
				return None;
			}
			let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
			match key.code {
				KeyCode::F(4) => self.file_dialog(Dialog::Menu { index: 0 }),
				KeyCode::Char('e' | 'E') if ctrl => self.file_action(Action::ExportEscaped),
				KeyCode::Char(ch) if ctrl => {
					let action = match ch.to_ascii_lowercase() {
						'n' => Action::New,
						'o' => Action::Open,
						'q' => Action::Quit,
						's' if key.modifiers.contains(KeyModifiers::SHIFT) => Action::SaveAs,
						's' => Action::Save,
						_ => return None,
					};
					self.file_action(action);
				}
				_ => return None,
			}
			return Some(self.files.exit);
		}
		if let Some(Dialog::Path { action, picker }) = &mut self.files.dialog {
			let outcome = picker.event(event);
			*action = picker.action();
			let action = *action;
			match outcome {
				crate::file_picker::Outcome::Submit(path) => {
					self.submit_path(action, path.to_string_lossy().into_owned());
				}
				crate::file_picker::Outcome::Cancel => self.cancel_file(),
				crate::file_picker::Outcome::None => (),
			}
			return Some(self.files.exit);
		}

		if matches!(self.files.dialog, Some(Dialog::Review(_))) {
			if let Some(choice) = self.files.review.event(event) {
				self.activate_file(choice);
			}
			return Some(self.files.exit);
		}

		match event {
			Event::Key(key) if key.kind != KeyEventKind::Release => {
				if key.code == KeyCode::Esc {
					self.cancel_file();
					return Some(false);
				}
				self.files.follow_focus = true;
				let menu_count = if matches!(self.files.dialog, Some(Dialog::Menu { .. })) {
					MENU.len()
				} else {
					2
				};
				let mut activate = None;
				match self.files.dialog.as_mut().unwrap() {
					Dialog::Path { .. } => unreachable!(),
					Dialog::Menu { index } => {
						let count = menu_count;
						match key.code {
							KeyCode::Up | KeyCode::BackTab => *index = index.saturating_sub(1),
							KeyCode::Down | KeyCode::Tab => {
								*index = (*index + 1).min(count.saturating_sub(1))
							}
							KeyCode::Enter => activate = Some(*index),
							_ => (),
						}
					}
					Dialog::Dirty => match key.code {
						KeyCode::Tab | KeyCode::Right | KeyCode::Down => {
							self.files.choice = (self.files.choice + 1) % 3
						}
						KeyCode::BackTab | KeyCode::Left | KeyCode::Up => {
							self.files.choice = (self.files.choice + 2) % 3
						}
						KeyCode::Enter => activate = Some(self.files.choice),
						KeyCode::Char('s' | 'S') => activate = Some(0),
						KeyCode::Char('d' | 'D' | 'y' | 'Y') => activate = Some(1),
						KeyCode::Char('c' | 'C' | 'n' | 'N') => activate = Some(2),
						_ => (),
					},
					Dialog::Overwrite { .. } => match key.code {
						KeyCode::Char('y' | 'Y') => activate = Some(0),
						KeyCode::Char('n' | 'N') => activate = Some(1),
						_ => (),
					},
					Dialog::Review(_) => match key.code {
						KeyCode::Char('s' | 'S') => activate = Some(0),
						KeyCode::Char('a' | 'A') => activate = Some(1),
						_ => (),
					},
				}
				if let Some(choice) = activate {
					self.activate_file(choice);
				} else if !matches!(self.files.dialog, Some(Dialog::Path { .. })) {
					match key.code {
						KeyCode::Up => self.files.scroll = self.files.scroll.saturating_sub(1),
						KeyCode::Down => self.files.scroll = self.files.scroll.saturating_add(1),
						KeyCode::PageUp => {
							self.files.scroll =
								self.files.scroll.saturating_sub(self.files.viewport)
						}
						KeyCode::PageDown => {
							self.files.scroll =
								self.files.scroll.saturating_add(self.files.viewport)
						}
						KeyCode::Home => self.files.scroll = 0,
						KeyCode::End => self.files.scroll = usize::MAX,
						_ => (),
					}
				}
			}
			Event::Mouse(mouse) => match mouse.kind {
				MouseEventKind::ScrollUp => self.files.scroll = self.files.scroll.saturating_sub(3),
				MouseEventKind::ScrollDown => {
					self.files.scroll = self.files.scroll.saturating_add(3)
				}
				MouseEventKind::Up(MouseButton::Left) => self.files.drag = None,
				MouseEventKind::Down(MouseButton::Left) => {
					if let Some(bar) = self
						.files
						.bar
						.filter(|bar| bar.area.contains((mouse.column, mouse.row).into()))
					{
						let at = bar.coordinate(mouse.column, mouse.row);
						let (start, size) = bar.thumb();
						let grab = if (start..start + size).contains(&at) {
							at - start
						} else {
							size / 2
						};
						self.files.drag = Some(grab);
						self.files.scroll = bar.offset_at(at, grab);
					} else if let Some((_, choice)) = self
						.files
						.buttons
						.iter()
						.find(|(rect, _)| rect.contains((mouse.column, mouse.row).into()))
						.copied()
					{
						self.activate_file(choice);
					}
				}
				MouseEventKind::Drag(MouseButton::Left) => {
					if let (Some(bar), Some(grab)) = (self.files.bar, self.files.drag) {
						self.files.scroll =
							bar.offset_at(bar.coordinate(mouse.column, mouse.row), grab);
					}
				}
				_ => (),
			},
			_ => (),
		}
		Some(self.files.exit)
	}
}

fn title(action: Action) -> &'static str {
	match action {
		Action::Open => "Open document",
		Action::Save | Action::SaveAs => "Save document as",
		Action::ImportRaw | Action::ImportEscaped => "Import ANSI",
		Action::ExportRaw | Action::ExportEscaped => "Export ANSI",
		_ => "File",
	}
}

fn draw_dirty(frame: &mut Frame, app: &mut App) {
	let area = frame.area();
	let width = area.width.min(40);
	let content_width = width.saturating_sub(4).max(1) as usize;
	let question = vec!["Save changes before continuing?".to_owned()];
	let layout = Layout::new(&question, content_width, true, false);
	let stacked = width < 30;
	let height = (layout.rows.len() as u16 + if stacked { 6 } else { 4 }).min(area.height);
	let popup = Rect::new(
		area.x + (area.width - width) / 2,
		area.y + (area.height - height) / 2,
		width,
		height,
	);
	frame.render_widget(Clear, popup);
	let block = Block::bordered().border_type(BorderType::Rounded);
	let inner = block.inner(popup);
	frame.render_widget(block, popup);
	app.files.buttons.clear();
	app.files.bar = None;
	if inner.width == 0 || inner.height == 0 {
		return;
	}
	for (i, row) in layout.rows.iter().enumerate().take(inner.height as usize) {
		frame.render_widget(
			Paragraph::new(&question[0][row.start..row.end]),
			Rect::new(
				inner.x + 1.min(inner.width),
				inner.y + i as u16,
				inner.width.saturating_sub(2),
				1,
			),
		);
	}
	let labels = ["Save", "Discard", "Cancel"];
	let mut x = inner.right().saturating_sub(28).max(inner.x);
	for (i, label) in labels.iter().enumerate() {
		let y = if stacked {
			inner.y + layout.rows.len() as u16 + 1 + i as u16
		} else {
			inner.y + layout.rows.len() as u16 + 1
		};
		if stacked {
			x = inner.x + 1.min(inner.width);
		}
		if y >= inner.bottom() || x >= inner.right() {
			continue;
		}
		let rect = Rect::new(x, y, (label.len() as u16 + 2).min(inner.right() - x), 1);
		frame.render_widget(
			Paragraph::new(format!(" {label} ")).style(
				Style::default()
					.fg(Color::Cyan)
					.add_modifier(Modifier::BOLD)
					.bg(if app.files.choice == i {
						Color::Rgb(41, 54, 83)
					} else {
						Color::Rgb(23, 28, 44)
					}),
			),
			rect,
		);
		app.files.buttons.push((rect, i));
		x += label.len() as u16 + 4;
	}
}

fn draw_menu(frame: &mut Frame, app: &mut App, index: usize) {
	let area = frame.area();
	let width = area.width.min(33);
	let height = area.height.min(12);
	let popup = Rect::new(
		(area.width - width) / 2,
		(area.height - height) / 2,
		width,
		height,
	);
	frame.render_widget(Clear, popup);
	let block = Block::bordered()
		.border_type(BorderType::Rounded)
		.title(" File ");
	let inner = block.inner(popup);
	frame.render_widget(block, popup);
	app.files.buttons.clear();
	app.files.bar = None;
	let rows: Vec<usize> = (0..MENU.len())
		.map(|i| i + usize::from(i >= 2) + usize::from(i >= 4) + usize::from(i >= 6))
		.collect();
	let top = rows[index].saturating_sub(inner.height.saturating_sub(1) as usize);
	for (i, (_, label)) in MENU.iter().enumerate() {
		let y = rows[i];
		if y < top || y - top >= inner.height as usize {
			continue;
		}
		let rect = Rect::new(inner.x, inner.y + (y - top) as u16, inner.width, 1);
		frame.render_widget(
			Paragraph::new(format!("{} {label}", if i == index { ">" } else { " " })).style(
				Style::default().bg(if i == index {
					Color::Rgb(41, 54, 83)
				} else {
					Color::Reset
				}),
			),
			rect,
		);
		app.files.buttons.push((rect, i));
	}
	for y in [2, 5, 8] {
		if y >= top && y - top < inner.height as usize {
			frame.render_widget(
				Paragraph::new(format!(
					"├{}┤",
					"─".repeat(width.saturating_sub(2) as usize)
				)),
				Rect::new(popup.x, inner.y + (y - top) as u16, width, 1),
			);
		}
	}
}

pub fn draw(frame: &mut Frame, app: &mut App) {
	if let Some(Dialog::Menu { index }) = app.files.dialog {
		draw_menu(frame, app, index);
		return;
	}
	if let Some(Dialog::Review(import)) = &app.files.dialog {
		app.files.review.draw(frame, import);
		return;
	}
	if matches!(app.files.dialog, Some(Dialog::Dirty)) {
		draw_dirty(frame, app);
		return;
	}
	if let Some(Dialog::Path { action, picker }) = &mut app.files.dialog {
		picker.draw(frame, title(*action), app.files.error.as_deref());
		return;
	}

	let Some(dialog) = &app.files.dialog else {
		return;
	};
	let area = frame.area();
	if area.width == 0 || area.height == 0 {
		return;
	}
	let popup = if area.width < 35 || area.height < 12 {
		area
	} else {
		let width = area.width.saturating_sub(4).min(88);
		let height = area.height.saturating_sub(4).min(24);
		Rect::new(
			area.x + (area.width - width) / 2,
			area.y + (area.height - height) / 2,
			width,
			height,
		)
	};
	frame.render_widget(Clear, popup);
	let caption = match dialog {
		Dialog::Menu { .. } => " File · Esc closes ",
		Dialog::Path { action, .. } => title(*action),
		Dialog::Dirty => " Unsaved changes ",
		Dialog::Overwrite { .. } => " Overwrite existing file? ",
		Dialog::Review(_) => " Import controls review ",
	};
	let block = Block::bordered()
		.border_type(BorderType::Rounded)
		.title(caption);
	let inner = block.inner(popup);
	frame.render_widget(block, popup);
	app.files.buttons.clear();
	app.files.bar = None;
	if inner.width == 0 || inner.height == 0 {
		return;
	}
	let mut lines = Vec::new();
	let mut actions: Vec<(usize, usize)> = Vec::new();
	match dialog {
		Dialog::Menu { index } => {
			for (i, (_, label)) in MENU.iter().enumerate() {
				actions.push((lines.len(), i));
				lines.push(format!("{} {label}", if i == *index { ">" } else { " " }));
			}
		}

		Dialog::Dirty => unreachable!(),
		Dialog::Overwrite { path, .. } => {
			lines.push(sanitize(&path.to_string_lossy()));
			for (i, label) in ["Y · Overwrite", "N / Esc · Cancel"].iter().enumerate() {
				actions.push((lines.len(), i));
				lines.push(label.to_string());
			}
		}
		Dialog::Review(_) => unreachable!(),
		Dialog::Path { .. } => unreachable!(),
	}
	if let Some(error) = &app.files.error {
		lines.push(format!("Error: {}", sanitize(error)));
	}
	let layout = Layout::new(&lines, inner.width as usize, true, false);
	let count = layout.rows.len();
	app.files.viewport = inner.height as usize;
	if app.files.follow_focus
		&& let Dialog::Menu { index } = dialog
		&& let Some(row) = layout.rows.iter().position(|row| row.line == *index)
	{
		if row < app.files.scroll {
			app.files.scroll = row;
		}
		if row >= app.files.scroll.saturating_add(app.files.viewport) {
			app.files.scroll = row + 1 - app.files.viewport;
		}
	}
	app.files.follow_focus = false;
	app.files.scroll = app
		.files
		.scroll
		.min(count.saturating_sub(app.files.viewport));
	for (visual, row) in layout
		.rows
		.iter()
		.enumerate()
		.skip(app.files.scroll)
		.take(app.files.viewport)
	{
		let rect = Rect::new(
			inner.x,
			inner.y + (visual - app.files.scroll) as u16,
			inner.width,
			1,
		);
		let text = &lines[row.line][row.start..row.end];
		let style = if actions.iter().any(|(line, _)| *line == row.line) {
			Style::default()
				.fg(Color::Cyan)
				.add_modifier(Modifier::BOLD)
		} else {
			Style::default()
		};
		frame.render_widget(Paragraph::new(text).style(style), rect);
		if let Some((_, choice)) = actions.iter().find(|(line, _)| *line == row.line) {
			app.files.buttons.push((rect, *choice));
		}
	}
	let mut bars = Vec::new();
	scrollbar::draw(
		frame,
		&mut bars,
		Bar {
			target: Target::Tools,
			area: Rect::new(popup.right() - 1, inner.y, 1, inner.height),
			content: count,
			viewport: app.files.viewport,
			offset: app.files.scroll,
		},
	);
	app.files.bar = bars.first().copied();
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		document::Position,
		editor::Movement,
		styling::{Action as StyleAction, Ink},
	};
	use crossterm::event::KeyEvent;

	fn type_text(app: &mut App, text: &str) {
		app.handle(Event::Paste(text.into()));
	}
	fn key(app: &mut App, code: KeyCode) -> bool {
		app.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)))
	}
	fn save_as(app: &mut App, path: &std::path::Path) {
		app.file_action(Action::SaveAs);
		app.submit_path(Action::SaveAs, path.to_string_lossy().into_owned());
	}

	#[test]
	fn imported_document_dialogs_keep_source_directory_and_cwd_shortcut() {
		let folder = tempfile::tempdir().unwrap();
		let source = folder.path().join("source.txt");
		std::fs::write(&source, "Imported").unwrap();
		let mut app = App::default();
		app.submit_path(Action::ImportEscaped, source.to_string_lossy().into_owned());
		assert!(app.document_path.is_none());
		for action in [
			Action::SaveAs,
			Action::Open,
			Action::ImportEscaped,
			Action::ExportEscaped,
		] {
			app.file_action(action);
			type_text(&mut app, "destination");
			let Some(Dialog::Path { picker, .. }) = &app.files.dialog else {
				panic!("expected picker")
			};
			assert_eq!(picker.destination(), folder.path().join("destination"));
			app.handle(Event::Key(KeyEvent::new(
				KeyCode::Char('g'),
				KeyModifiers::CONTROL,
			)));
			let Some(Dialog::Path { picker, .. }) = &app.files.dialog else {
				panic!("expected picker")
			};
			// Navigation preserves an export/save filename but clears open/import names.
			if matches!(action, Action::SaveAs | Action::ExportEscaped) {
				assert_eq!(
					picker.destination(),
					std::env::current_dir().unwrap().join("destination")
				);
			}
			key(&mut app, KeyCode::Esc);
		}
	}

	#[test]
	fn open_non_document_offers_import_and_uses_selected_format() {
		let folder = tempfile::tempdir().unwrap();
		let path = folder.path().join("styled.txt");
		std::fs::write(&path, r"\x1b[31mRed\x1b[0m").unwrap();
		let mut app = App::default();
		type_text(&mut app, "keep");
		let before = app.editor.document.clone();
		app.submit_path(Action::Open, path.to_string_lossy().into_owned());
		assert!(matches!(
			app.files.dialog,
			Some(Dialog::Path {
				action: Action::ImportEscaped,
				..
			})
		));
		key(&mut app, KeyCode::Esc);
		assert_eq!(app.editor.document, before);
		app.submit_path(Action::Open, path.to_string_lossy().into_owned());
		key(&mut app, KeyCode::Right); // Raw -> Escaped, still no import.
		assert_eq!(app.editor.document, before);
		key(&mut app, KeyCode::Enter); // Import directly from the selected format.
		assert!(matches!(app.files.dialog, Some(Dialog::Dirty)));
		key(&mut app, KeyCode::Char('d'));
		assert_eq!(app.editor.document.lines, ["Red"]);
		assert_eq!(app.editor.document.styles.at(0).foreground, Ink::Ansi(1));
		assert!(app.document_path.is_none());
		assert!(app.dirty());
	}

	#[test]
	fn save_dirty_undo_redo_and_export_are_independent() {
		let folder = tempfile::tempdir().unwrap();
		let path = folder.path().join("styled.tuxd.json");
		let mut app = App::default();
		type_text(&mut app, "Hello\n界");
		app.editor.navigate(Movement::Start, true);
		app.editor
			.apply_tool(StyleAction::Background(Ink::Rgb(1, 2, 3)));
		app.editor.anchor = None;
		assert!(app.dirty());
		save_as(&mut app, &path);
		assert!(!app.dirty());
		assert!(app.files.dialog.is_none());
		assert_eq!(persistence::load(&path).unwrap(), app.editor.document);
		type_text(&mut app, "X");
		assert!(app.dirty());
		app.editor.undo(false);
		assert!(!app.dirty());
		app.editor.undo(true);
		assert!(app.dirty());
		let raw = folder.path().join("output.ansi");
		app.write_file(Action::ExportRaw, raw.clone(), false);
		assert_eq!(
			std::fs::read(raw).unwrap(),
			styling::ansi(&app.editor.document).as_bytes()
		);
		let escaped = folder.path().join("output.txt");
		app.write_file(Action::ExportEscaped, escaped.clone(), false);
		assert_eq!(
			std::fs::read(escaped).unwrap(),
			styling::clipboard_text(&app.editor.document).as_bytes()
		);
		assert!(app.dirty());
		assert_eq!(app.document_path, Some(path));
	}

	#[test]
	fn pending_new_can_cancel_save_or_discard_without_losing_history() {
		let folder = tempfile::tempdir().unwrap();
		let mut app = App::default();
		type_text(&mut app, "keep");
		let before = app.editor.document.clone();
		app.file_action(Action::New);
		assert!(matches!(app.files.dialog, Some(Dialog::Dirty)));
		assert_eq!(app.files.choice, 2);
		key(&mut app, KeyCode::Enter); // Default Cancel leaves the document unchanged.
		assert!(app.files.dialog.is_none());
		assert_eq!(app.editor.document, before);
		app.file_action(Action::New);
		key(&mut app, KeyCode::Char('s'));
		assert!(matches!(app.files.dialog, Some(Dialog::Path { .. })));
		let path = folder.path().join("saved.tuxd.json");
		type_text(&mut app, path.to_str().unwrap());
		key(&mut app, KeyCode::Enter);
		assert_eq!(persistence::load(&path).unwrap(), before);
		assert_eq!(app.editor.document, Document::default());
		assert!(!app.dirty());
		app.editor.undo(false);
		assert_eq!(app.editor.document, Document::default());
	}

	#[test]
	fn failures_and_overwrite_cancel_preserve_document_path_and_dirty_state() {
		let folder = tempfile::tempdir().unwrap();
		let existing = folder.path().join("existing.tuxd.json");
		std::fs::write(&existing, b"keep bytes").unwrap();
		let mut app = App::default();
		type_text(&mut app, "unsaved");
		let before = app.editor.document.clone();
		save_as(&mut app, &existing);
		assert!(matches!(app.files.dialog, Some(Dialog::Overwrite { .. })));
		key(&mut app, KeyCode::Char('n'));
		assert_eq!(std::fs::read(&existing).unwrap(), b"keep bytes");
		app.file_action(Action::Open);
		app.submit_path(Action::Open, existing.to_str().unwrap().into());
		assert!(app.files.error.is_some());
		key(&mut app, KeyCode::Esc);
		app.file_action(Action::Quit);
		key(&mut app, KeyCode::Char('s'));
		app.submit_path(
			Action::SaveAs,
			folder
				.path()
				.join("missing/sub/doc")
				.to_str()
				.unwrap()
				.into(),
		);
		assert!(app.files.error.is_some());
		assert!(!app.files.exit);
		assert!(app.document_path.is_none());
		assert!(app.dirty());
		assert_eq!(app.editor.document, before);
		key(&mut app, KeyCode::Esc);
		app.editor.undo(false);
		assert!(app.editor.document.is_empty());
	}

	#[test]
	fn import_review_abort_strip_and_dirty_protection() {
		let folder = tempfile::tempdir().unwrap();
		let source = folder.path().join("import.txt");
		std::fs::write(&source, r"\x1b[31mRed\x1b[2J\n界\q").unwrap();
		let mut app = App::default();
		type_text(&mut app, "original");
		let original = app.editor.document.clone();
		for abort in [true, false] {
			app.file_action(Action::ImportEscaped);
			app.submit_path(Action::ImportEscaped, source.to_str().unwrap().into());
			if let Some(Dialog::Review(import)) = &app.files.dialog {
				assert_eq!(import.issues.len(), 2);
			} else {
				panic!("expected review");
			}
			// Pasting a response cannot answer a confirmation.
			type_text(&mut app, "s");
			assert_eq!(app.editor.document, original);
			key(&mut app, KeyCode::Char(if abort { 'a' } else { 's' }));
			assert_eq!(app.editor.document, original);
		}
		assert!(matches!(app.files.dialog, Some(Dialog::Dirty)));
		key(&mut app, KeyCode::Char('d'));
		assert_eq!(app.editor.document.lines, ["Red", "界"]);
		assert_eq!(app.editor.document.styles.at(0).foreground, Ink::Ansi(1));
		assert!(app.dirty());
		assert!(app.document_path.is_none());
		app.editor.undo(false);
		assert_eq!(app.editor.document.lines, ["Red", "界"]);
	}

	#[test]
	fn clean_document_quit_and_save_before_quit() {
		let folder = tempfile::tempdir().unwrap();
		let path = folder.path().join("doc.tuxd.json");
		let mut app = App::default();
		let mut doc = Document::default();
		doc.insert(Position::default(), "saved");
		app.load_document(doc, Some(path.clone()), false);
		assert!(!app.dirty());
		type_text(&mut app, "new");
		app.file_action(Action::Quit);
		assert!(key(&mut app, KeyCode::Char('s')));
		assert_eq!(persistence::load(&path).unwrap(), app.editor.document);
		assert!(!app.dirty());
	}

	#[test]
	fn export_refuses_to_replace_active_document() {
		let folder = tempfile::tempdir().unwrap();
		let path = folder.path().join("doc.tuxd.json");
		let mut app = App::default();
		type_text(&mut app, "content");
		save_as(&mut app, &path);
		app.write_file(Action::ExportRaw, path.clone(), true);
		assert!(app.files.error.is_some());
		assert_eq!(persistence::load(&path).unwrap(), app.editor.document);
	}

	#[test]
	fn long_review_and_narrow_menus_remain_navigable() {
		use ratatui::{Terminal, backend::TestBackend};
		let mut app = App::default();
		app.file_dialog(Dialog::Review(
			ansi_import::parse(&"\x1b[2J".repeat(80), Mode::Raw).unwrap(),
		));
		let mut terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
		terminal.draw(|frame| draw(frame, &mut app)).unwrap();
		assert!(app.files.review.has_scrollbar());
		key(&mut app, KeyCode::End);
		terminal.draw(|frame| draw(frame, &mut app)).unwrap();
		assert!(app.files.review.scrolled());
		key(&mut app, KeyCode::Char('a'));
		app.file_dialog(Dialog::Menu { index: 0 });
		for _ in 0..8 {
			terminal.draw(|frame| draw(frame, &mut app)).unwrap();
			key(&mut app, KeyCode::Down);
		}
		assert!(matches!(app.files.dialog, Some(Dialog::Menu { index: 6 })));
	}
}
