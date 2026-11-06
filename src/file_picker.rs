//! Shared local file picker. Explorer owns directory discovery/filtering; our rows
//! keep mouse hit targets and scrollbar offsets consistent with the other panels.
use crate::{
	document::sanitize,
	files::Action,
	scrollbar::{self, Bar, Target},
};
use crossterm::event::{
	Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use ratatui::{
	Frame,
	layout::Rect,
	style::{Color, Modifier, Style},
	text::{Line, Span},
	widgets::{Block, BorderType, Clear, Paragraph},
};
use ratatui_explorer::FileExplorer;
use std::{
	path::{Path, PathBuf},
	sync::mpsc::{Receiver, TryRecvError},
	time::{Duration, Instant},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const SELECTED: Color = Color::Rgb(41, 54, 83);
#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
	Location,
	List,
	Format,
	Name,
	Submit,
	Cancel,
}
impl Focus {
	fn next(self, back: bool, format: bool) -> Self {
		let mut items = vec![Self::Location, Self::List];
		if format {
			items.push(Self::Format);
		}
		items.extend([Self::Name, Self::Submit, Self::Cancel]);
		let at = items.iter().position(|x| *x == self).unwrap_or(0);
		items[(at + if back { items.len() - 1 } else { 1 }) % items.len()]
	}
}
#[derive(Default)]
struct Field {
	text: String,
	cursor: usize,
	selected: bool,
}
impl Field {
	fn set(&mut self, text: String) {
		self.cursor = text.len();
		self.text = text;
		self.selected = false;
	}
	fn insert(&mut self, text: &str) {
		if self.selected {
			self.text.clear();
			self.cursor = 0;
			self.selected = false;
		}
		let clean: String = text.chars().filter(|ch| !ch.is_control()).collect();
		self.text.insert_str(self.cursor, &clean);
		self.cursor += clean.len();
	}
	fn key(&mut self, key: KeyEvent) {
		if key.modifiers.contains(KeyModifiers::CONTROL)
			&& matches!(key.code, KeyCode::Char('a' | 'A'))
		{
			self.selected = true;
			return;
		}
		if matches!(key.code, KeyCode::Backspace | KeyCode::Delete) && self.selected {
			self.set(String::new());
			return;
		}
		match key.code {
			KeyCode::Char(ch)
				if !key
					.modifiers
					.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
					&& !ch.is_control() =>
			{
				self.insert(&ch.to_string())
			}
			KeyCode::Home => {
				self.cursor = 0;
				self.selected = false;
			}
			KeyCode::End => {
				self.cursor = self.text.len();
				self.selected = false;
			}
			KeyCode::Left | KeyCode::Backspace if self.cursor > 0 => {
				let previous = self.text[..self.cursor]
					.grapheme_indices(true)
					.next_back()
					.map_or(0, |(i, _)| i);
				if key.code == KeyCode::Backspace {
					self.text.replace_range(previous..self.cursor, "");
				}
				self.cursor = previous;
				self.selected = false;
			}
			KeyCode::Right | KeyCode::Delete if self.cursor < self.text.len() => {
				let next = self.cursor
					+ self.text[self.cursor..]
						.graphemes(true)
						.next()
						.unwrap()
						.len();
				if key.code == KeyCode::Delete {
					self.text.replace_range(self.cursor..next, "");
				} else {
					self.cursor = next;
				}
				self.selected = false;
			}
			_ => (),
		}
	}
	fn draw(&self, frame: &mut Frame, area: Rect, focused: bool) {
		let before = sanitize(&self.text[..self.cursor]);
		let text = format!(
			"{}{}{}",
			before,
			if focused { "▏" } else { "" },
			sanitize(&self.text[self.cursor..])
		);
		let skip = if focused {
			before
				.width()
				.saturating_sub(area.width.saturating_sub(1) as usize)
		} else {
			0
		};
		frame.render_widget(
			Paragraph::new(text)
				.scroll((0, skip.min(u16::MAX as usize) as u16))
				.style(
					Style::default()
						.bg(if focused {
							SELECTED
						} else {
							Color::Rgb(23, 28, 44)
						})
						.add_modifier(if focused && self.selected {
							Modifier::REVERSED
						} else {
							Modifier::empty()
						}),
				),
			area,
		);
	}
}

pub enum Outcome {
	None,
	Submit(PathBuf),
	Cancel,
}
pub struct Picker {
	explorer: Option<FileExplorer>,
	cwd: PathBuf,
	action: Action,
	location: Field,
	name: Field,
	focus: Focus,
	projects_only: bool,
	show_shortcuts: bool,
	import_prompt: bool,
	help_area: Rect,
	help_top: usize,
	format_buttons: Vec<(Rect, bool)>,
	pending_g: bool,
	error: Option<String>,
	top: usize,
	list_area: Rect,
	location_area: Rect,
	name_area: Rect,
	buttons: Vec<(Rect, Focus)>,
	bar: Option<Bar>,
	drag: Option<usize>,
	last_click: Option<(usize, Instant)>,
	paste: Option<(Focus, Receiver<Result<String, String>>)>,
}
impl Picker {
	pub fn new(action: Action, text: &str, document: Option<&Path>) -> Self {
		let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
		let document = document.map(|path| base.join(path));
		let cwd = document
			.as_deref()
			.and_then(Path::parent)
			.filter(|p| p.is_dir())
			.unwrap_or(&base)
			.to_path_buf();
		let mut picker = Self {
			explorer: None,
			cwd,
			action,
			location: Field::default(),
			name: Field::default(),
			focus: Focus::List,
			projects_only: matches!(action, Action::Open | Action::Save | Action::SaveAs),
			show_shortcuts: false,
			import_prompt: false,
			help_area: Rect::default(),
			help_top: 0,
			format_buttons: Vec::new(),
			pending_g: false,
			error: None,
			top: 0,
			list_area: Rect::default(),
			location_area: Rect::default(),
			name_area: Rect::default(),
			buttons: Vec::new(),
			bar: None,
			drag: None,
			last_click: None,
			paste: None,
		};
		match FileExplorer::new() {
			Ok(explorer) => picker.explorer = Some(explorer),
			Err(error) => picker.error = Some(error.to_string()),
		}
		picker.navigate(picker.cwd.clone());
		if !text.is_empty() {
			let path = base.join(text);
			if let Some(parent) = path.parent().filter(|p| p.is_dir()) {
				picker.navigate(parent.to_path_buf());
				picker.name.set(
					path.file_name()
						.unwrap_or_default()
						.to_string_lossy()
						.into_owned(),
				);
			} else {
				picker.name.set(path.to_string_lossy().into_owned());
			}
			picker.name.selected = true;
		}
		picker.filter();
		picker
	}
	fn filter(&mut self) {
		let only = self.projects_only;
		if let Some(explorer) = &mut self.explorer
			&& let Err(error) = explorer.set_filter_map(move |mut file| {
				let keep = !only
					|| file.is_dir || file
					.path
					.file_name()
					.is_some_and(|name| name.to_string_lossy().ends_with(".tuxd.json"));
				file.name = sanitize(&file.name);
				keep.then_some(file)
			}) {
			self.error = Some(error.to_string());
		}
		self.top = 0;
	}
	fn resolve(&self, text: &str) -> PathBuf {
		if (text == "~" || text.starts_with("~/"))
			&& let Some(home) = std::env::var_os("HOME")
		{
			return PathBuf::from(home).join(text.strip_prefix("~/").unwrap_or(""));
		}
		self.cwd.join(text)
	}
	pub fn destination(&self) -> PathBuf {
		if self.name.text.is_empty() {
			PathBuf::new()
		} else {
			self.resolve(&self.name.text)
		}
	}
	fn navigate(&mut self, path: PathBuf) {
		let result = (|| {
			let path = path.canonicalize()?;
			if self.explorer.is_none() {
				self.explorer = Some(FileExplorer::new()?);
			}
			self.explorer.as_mut().unwrap().set_cwd(&path)?;
			Ok::<_, std::io::Error>(path)
		})();
		match result {
			Ok(path) => {
				if matches!(
					self.action,
					Action::Open | Action::ImportRaw | Action::ImportEscaped
				) {
					self.name.set(String::new());
				}
				self.cwd = path;
				self.location.set(self.cwd.to_string_lossy().into_owned());
				self.top = 0;
				self.error = None;
				self.last_click = None;
			}
			Err(error) => self.error = Some(error.to_string()),
		}
	}
	fn selected(&self) -> Option<(PathBuf, bool)> {
		let explorer = self.explorer.as_ref()?;
		explorer
			.files()
			.get(explorer.selected_idx())
			.map(|file| (file.path.clone(), file.is_dir))
	}
	fn choose(&mut self) -> Outcome {
		if let Some((path, directory)) = self.selected() {
			if directory {
				self.navigate(path);
			} else {
				self.name.set(
					path.file_name()
						.unwrap_or_default()
						.to_string_lossy()
						.into_owned(),
				);
				if matches!(
					self.action,
					Action::Open | Action::ImportRaw | Action::ImportEscaped
				) {
					return Outcome::Submit(path);
				}
				self.focus = Focus::Name;
				self.name.selected = true;
			}
		}
		Outcome::None
	}
	fn submit(&mut self) -> Outcome {
		let path = self.destination();
		if path.as_os_str().is_empty() {
			self.error = Some("Enter a filename or choose a file".into());
			return Outcome::None;
		}
		if path.is_dir() {
			self.navigate(path);
			self.name.set(String::new());
			self.focus = Focus::List;
			Outcome::None
		} else {
			Outcome::Submit(path)
		}
	}
	fn move_selection(&mut self, code: KeyCode) {
		if let Some(explorer) = &mut self.explorer {
			let count = explorer.files().len();
			if count == 0 {
				return;
			}
			let at = explorer.selected_idx();
			let page = self.list_area.height.max(1) as usize;
			let next = match code {
				KeyCode::Up => at.saturating_sub(1),
				KeyCode::Down => (at + 1).min(count - 1),
				KeyCode::PageUp => at.saturating_sub(page),
				KeyCode::PageDown => (at + page).min(count - 1),
				KeyCode::Home => 0,
				KeyCode::End => count - 1,
				KeyCode::Char('M') => {
					self.top.min(count - 1)
						+ count.saturating_sub(self.top).min(page).saturating_sub(1) / 2
				}
				_ => at,
			};
			explorer.set_selected_idx(next);
			if !explorer.files()[next].is_dir {
				self.name.set(
					explorer.files()[next]
						.path
						.file_name()
						.unwrap_or_default()
						.to_string_lossy()
						.into_owned(),
				);
			}
			if next < self.top {
				self.top = next;
			}
			if next >= self.top + page {
				self.top = next + 1 - page;
			}
		}
	}
	fn insert(&mut self, focus: Focus, text: &str) {
		match focus {
			Focus::Location => self.location.insert(text),
			_ => {
				self.name.insert(text);
				self.focus = Focus::Name;
			}
		}
	}
	pub fn poll_paste(&mut self) -> bool {
		let Some((focus, receiver)) = &self.paste else {
			return false;
		};
		let focus = *focus;
		match receiver.try_recv() {
			Ok(Ok(text)) => {
				self.insert(focus, &text);
				self.error = None;
			}
			Ok(Err(error)) => self.error = Some(error),
			Err(TryRecvError::Disconnected) => self.error = Some("Clipboard helper failed".into()),
			Err(TryRecvError::Empty) => return false,
		}
		self.paste = None;
		true
	}
	pub fn offer_import(&mut self) {
		self.import_prompt = true;
		self.focus = Focus::Format;
	}
	pub fn action(&self) -> Action {
		self.action
	}
	fn has_format(&self) -> bool {
		matches!(
			self.action,
			Action::ImportRaw | Action::ImportEscaped | Action::ExportRaw | Action::ExportEscaped
		)
	}
	fn escaped(&self) -> bool {
		matches!(self.action, Action::ImportEscaped | Action::ExportEscaped)
	}
	fn set_format(&mut self, escaped: bool) {
		self.action = match (self.action, escaped) {
			(Action::ImportRaw | Action::ImportEscaped, false) => Action::ImportRaw,
			(Action::ImportRaw | Action::ImportEscaped, true) => Action::ImportEscaped,
			(Action::ExportRaw | Action::ExportEscaped, false) => Action::ExportRaw,
			(Action::ExportRaw | Action::ExportEscaped, true) => Action::ExportEscaped,
			(action, _) => action,
		};
	}
	pub fn event(&mut self, event: &Event) -> Outcome {
		match event {
			Event::Paste(text) => self.insert(self.focus, text),
			Event::Key(key) if key.kind != KeyEventKind::Release => {
				let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
				let plain = !key
					.modifiers
					.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
				let second_g = self.pending_g
					&& self.focus == Focus::List
					&& plain && key.code == KeyCode::Char('g');
				self.pending_g = false;
				match key.code {
					// Legacy Ctrl+/ (0x1f) is decoded by Crossterm as Ctrl+7.
					KeyCode::Char('/' | '?' | '_' | '7') if ctrl => {
						self.show_shortcuts = !self.show_shortcuts;
						self.help_top = 0;
					}
					KeyCode::F(1) => {
						self.show_shortcuts = !self.show_shortcuts;
						self.help_top = 0;
					}
					KeyCode::Char('M') if self.focus == Focus::List && plain => {
						self.move_selection(KeyCode::Char('M'))
					}
					KeyCode::Char('g') if self.focus == Focus::List && plain => {
						if second_g {
							self.move_selection(KeyCode::Home);
						} else {
							self.pending_g = true;
						}
					}
					KeyCode::Char('G') if self.focus == Focus::List && plain => {
						self.move_selection(KeyCode::End)
					}
					KeyCode::Char('j' | 'k') if self.focus == Focus::List && plain => self
						.move_selection(if key.code == KeyCode::Char('j') {
							KeyCode::Down
						} else {
							KeyCode::Up
						}),
					KeyCode::Char('h') if self.focus == Focus::List && plain => {
						if let Some(parent) = self.cwd.parent() {
							self.navigate(parent.to_path_buf());
						}
					}
					KeyCode::Char('l') if self.focus == Focus::List && plain => {
						if let Some((path, true)) = self.selected() {
							self.navigate(path);
						}
					}
					KeyCode::Left | KeyCode::Right | KeyCode::Char(' ')
						if self.focus == Focus::Format =>
					{
						self.set_format(match key.code {
							KeyCode::Left => false,
							KeyCode::Right => true,
							_ => !self.escaped(),
						})
					}
					KeyCode::Esc => return Outcome::Cancel,
					KeyCode::Tab | KeyCode::BackTab => {
						self.focus = self.focus.next(
							key.code == KeyCode::BackTab
								|| key.modifiers.contains(KeyModifiers::SHIFT),
							self.has_format(),
						)
					}
					KeyCode::Char('g' | 'G') if ctrl => match std::env::current_dir() {
						Ok(path) => {
							self.navigate(path);
							self.focus = Focus::List;
						}
						Err(error) => self.error = Some(error.to_string()),
					},
					KeyCode::Char('l' | 'L') if ctrl => {
						self.focus = Focus::Location;
						self.location.selected = true;
					}
					KeyCode::Char('v' | 'V') if ctrl && self.paste.is_none() => {
						let (sender, receiver) = std::sync::mpsc::channel();
						self.paste = Some((self.focus, receiver));
						std::thread::spawn(move || {
							let _ = sender.send(crate::clipboard::paste());
						});
					}
					KeyCode::Char('h' | 'H' | '.') if ctrl => {
						if let Some(explorer) = &mut self.explorer {
							if let Err(error) = explorer.set_show_hidden(!explorer.show_hidden()) {
								self.error = Some(error.to_string());
							}
							self.top = 0;
						}
					}
					KeyCode::Char('f' | 'F') if ctrl => {
						self.projects_only = !self.projects_only;
						self.filter();
					}
					KeyCode::Enter => match self.focus {
						Focus::Location => {
							self.navigate(self.resolve(&self.location.text));
							self.focus = Focus::List;
						}
						Focus::List => return self.choose(),
						Focus::Cancel => return Outcome::Cancel,
						Focus::Format => return self.submit(),
						_ => return self.submit(),
					},
					KeyCode::Left | KeyCode::Backspace if self.focus == Focus::List => {
						if let Some(parent) = self.cwd.parent() {
							self.navigate(parent.to_path_buf());
						}
					}
					KeyCode::Right if self.focus == Focus::List => {
						if let Some((path, true)) = self.selected() {
							self.navigate(path);
						}
					}
					code if self.focus == Focus::List => self.move_selection(code),
					_ if self.focus == Focus::Location => self.location.key(*key),
					_ if self.focus == Focus::Name => self.name.key(*key),
					_ => (),
				}
			}
			Event::Mouse(mouse) => {
				let point = (mouse.column, mouse.row).into();
				self.pending_g = false;
				match mouse.kind {
					MouseEventKind::ScrollUp if self.help_area.contains(point) => {
						self.help_top = self.help_top.saturating_sub(3)
					}
					MouseEventKind::ScrollDown if self.help_area.contains(point) => {
						self.help_top = self.help_top.saturating_add(3)
					}
					MouseEventKind::Up(MouseButton::Left) => self.drag = None,
					MouseEventKind::Drag(MouseButton::Left) => {
						if let (Some(bar), Some(grab)) = (self.bar, self.drag) {
							self.top = bar.offset_at(bar.coordinate(mouse.column, mouse.row), grab);
						}
					}
					MouseEventKind::ScrollUp if self.list_area.contains(point) => {
						self.top = self.top.saturating_sub(3)
					}
					MouseEventKind::ScrollDown if self.list_area.contains(point) => {
						self.top = self.top.saturating_add(3)
					}
					MouseEventKind::Down(MouseButton::Left) => {
						if let Some(bar) = self.bar.filter(|bar| bar.area.contains(point)) {
							let at = bar.coordinate(mouse.column, mouse.row);
							let (start, size) = bar.thumb();
							let grab = if (start..start + size).contains(&at) {
								at - start
							} else {
								size / 2
							};
							self.top = bar.offset_at(at, grab);
							self.drag = Some(grab);
						} else if let Some((_, escaped)) = self
							.format_buttons
							.iter()
							.find(|(rect, _)| rect.contains(point))
						{
							let escaped = *escaped;
							self.focus = Focus::Format;
							self.set_format(escaped);
						} else if self.location_area.contains(point) {
							self.focus = Focus::Location;
							self.location.selected = true;
						} else if self.name_area.contains(point) {
							self.focus = Focus::Name;
							self.name.selected = true;
						} else if let Some((_, focus)) =
							self.buttons.iter().find(|(area, _)| area.contains(point))
						{
							self.focus = *focus;
							return if *focus == Focus::Cancel {
								Outcome::Cancel
							} else {
								self.submit()
							};
						} else if self.list_area.contains(point) {
							self.focus = Focus::List;
							let index = self.top + (mouse.row - self.list_area.y) as usize;
							if let Some(explorer) = &mut self.explorer
								&& index < explorer.files().len()
							{
								explorer.set_selected_idx(index);
								if let Some((path, false)) = self.selected() {
									self.name.set(
										path.file_name()
											.unwrap_or_default()
											.to_string_lossy()
											.into_owned(),
									);
								}
								let twice = self.last_click.is_some_and(|(last, time)| {
									last == index && time.elapsed() < Duration::from_millis(400)
								});
								self.last_click = Some((index, Instant::now()));
								if twice {
									return self.choose();
								}
							}
						}
					}
					_ => (),
				}
			}
			_ => (),
		}
		Outcome::None
	}
	pub fn draw(&mut self, frame: &mut Frame, title: &str, error: Option<&str>) {
		let area = frame.area();
		let width = area
			.width
			.saturating_sub(if area.width > 40 { 4 } else { 0 })
			.min(96);
		let available = area
			.height
			.saturating_sub(if area.height > 16 { 2 } else { 0 });
		let help_lines = self.shortcut_lines(width.saturating_sub(4) as usize);
		let help_height = if self.show_shortcuts {
			(help_lines.len() as u16 + 2).min(available.saturating_sub(10))
		} else {
			0
		};
		let height = available.saturating_sub(help_height).min(34);
		let popup = Rect::new(
			(area.width - width) / 2,
			(area.height - height - help_height) / 2,
			width,
			height,
		);
		self.help_area = Rect::new(popup.x, popup.bottom(), width, help_height);
		if help_height > 0 {
			frame.render_widget(Clear, self.help_area);
			let block = Block::bordered()
				.border_type(BorderType::Rounded)
				.title(" Shortcuts ");
			let inner = block.inner(self.help_area);
			frame.render_widget(block, self.help_area);
			self.help_top = self
				.help_top
				.min(help_lines.len().saturating_sub(inner.height as usize));
			frame.render_widget(
				Paragraph::new(help_lines).scroll((self.help_top.min(u16::MAX as usize) as u16, 0)),
				Rect::new(
					inner.x + 1.min(inner.width),
					inner.y,
					inner.width.saturating_sub(2),
					inner.height,
				),
			);
			let mut bars = Vec::new();
			scrollbar::draw(
				frame,
				&mut bars,
				Bar {
					target: Target::Tools,
					area: Rect::new(
						self.help_area.right().saturating_sub(1),
						inner.y,
						1,
						inner.height,
					),
					content: self.shortcut_lines(width.saturating_sub(4) as usize).len(),
					viewport: inner.height as usize,
					offset: self.help_top,
				},
			);
		}
		frame.render_widget(Clear, popup);
		let block =
			Block::bordered()
				.border_type(BorderType::Rounded)
				.title(if self.import_prompt {
					" Import this file as ANSI? ".to_owned()
				} else {
					format!(" {title} ")
				});
		let inner = block.inner(popup);
		frame.render_widget(block, popup);
		self.buttons.clear();
		self.format_buttons.clear();
		self.bar = None;
		self.list_area = Rect::default();
		self.name_area = Rect::default();
		self.location_area = Rect::default();
		if inner.width == 0 || inner.height == 0 {
			return;
		}
		let row = |y: u16| {
			Rect::new(
				inner.x + 1.min(inner.width),
				inner.y + y.min(inner.height),
				inner.width.saturating_sub(2),
				u16::from(y < inner.height),
			)
		};
		if inner.height < 10 {
			if matches!(self.focus, Focus::List | Focus::Location) {
				self.focus = Focus::Name;
			}
			self.name_area = row(0);
			self.name
				.draw(frame, self.name_area, self.focus == Focus::Name);
			if inner.height > 1 {
				frame.render_widget(Paragraph::new("Enter: choose · Esc: cancel"), row(1));
			}
			if self.has_format() && inner.height > 2 {
				self.draw_format(frame, row(2));
			}
			return;
		}
		let label_width = 11.min(row(0).width);
		frame.render_widget(Paragraph::new("Location:"), row(0));
		self.location_area = Rect::new(
			row(0).x + label_width,
			row(0).y,
			row(0).width - label_width,
			1,
		);
		self.location
			.draw(frame, self.location_area, self.focus == Focus::Location);
		let list_start = if self.has_format() {
			self.draw_format(frame, row(2));
			4
		} else {
			1
		};
		let list_box = Rect::new(
			inner.x,
			inner.y + list_start,
			inner.width,
			inner.height.saturating_sub(list_start + 3),
		);
		let block = Block::bordered()
			.border_type(BorderType::Rounded)
			.border_style(Style::default().fg(if self.focus == Focus::List {
				Color::Cyan
			} else {
				Color::Rgb(62, 78, 106)
			}));
		let list_inner = block.inner(list_box);
		frame.render_widget(block, list_box);
		self.list_area = Rect::new(
			list_inner.x + 1.min(list_inner.width),
			list_inner.y,
			list_inner.width.saturating_sub(2),
			list_inner.height,
		);
		if let Some(explorer) = &self.explorer {
			let count = explorer.files().len();
			self.top = self
				.top
				.min(count.saturating_sub(self.list_area.height as usize));
			for (index, file) in explorer
				.files()
				.iter()
				.enumerate()
				.skip(self.top)
				.take(self.list_area.height as usize)
			{
				let rect = Rect::new(
					self.list_area.x,
					self.list_area.y + (index - self.top) as u16,
					self.list_area.width,
					1,
				);
				let selected = explorer.selected_idx() == index;
				let style = Style::default()
					.fg(if file.is_dir {
						Color::Cyan
					} else {
						Color::Reset
					})
					.bg(if selected { SELECTED } else { Color::Reset });
				frame.render_widget(
					Paragraph::new(format!(
						"{} {}",
						if selected && self.focus == Focus::List {
							"›"
						} else {
							" "
						},
						file.name
					))
					.style(style),
					rect,
				);
			}
			let mut bars = Vec::new();
			scrollbar::draw(
				frame,
				&mut bars,
				Bar {
					target: Target::Tools,
					area: Rect::new(
						self.list_area.right(),
						self.list_area.y,
						1,
						self.list_area.height,
					),
					content: count,
					viewport: self.list_area.height as usize,
					offset: self.top,
				},
			);
			self.bar = bars.first().copied();
		}
		let y = inner.height - 3;
		frame.render_widget(Paragraph::new("File/path:"), row(y));
		self.name_area = Rect::new(
			row(y).x + label_width,
			row(y).y,
			row(y).width - label_width,
			1,
		);
		self.name
			.draw(frame, self.name_area, self.focus == Focus::Name);
		let label = match self.action {
			Action::Open => "Open",
			Action::Save | Action::SaveAs => "Save",
			Action::ImportRaw | Action::ImportEscaped => "Import",
			_ => "Export",
		};
		let button_row = row(y + 2);
		let start = button_row.width.saturating_sub(label.len() as u16 + 12);
		for (offset, text, focus) in [
			(start, label, Focus::Submit),
			(start + label.len() as u16 + 4, "Cancel", Focus::Cancel),
		] {
			if offset >= button_row.width {
				continue;
			}
			let rect = Rect::new(
				button_row.x + offset,
				button_row.y,
				(text.len() as u16 + 2).min(button_row.width - offset),
				1,
			);
			frame.render_widget(
				Paragraph::new(format!(" {text} ")).style(
					Style::default()
						.fg(Color::Cyan)
						.add_modifier(Modifier::BOLD)
						.bg(if self.focus == focus {
							SELECTED
						} else {
							Color::Rgb(23, 28, 44)
						}),
				),
				rect,
			);
			self.buttons.push((rect, focus));
		}
		if let Some(message) = self.error.as_deref().or(error) {
			frame.render_widget(
				Paragraph::new(format!("Error: {}", sanitize(message)))
					.style(Style::default().fg(Color::LightRed)),
				row(y + 1),
			);
		} else if self.paste.is_some() {
			frame.render_widget(Paragraph::new("Reading clipboard…"), row(inner.height - 1));
		}
	}
	fn draw_format(&mut self, frame: &mut Frame, row: Rect) {
		for (offset, label, escaped) in [(0, "Raw ANSI", false), (14, "Escaped ANSI", true)] {
			if offset >= row.width {
				continue;
			}
			let selected = escaped == self.escaped();
			let rect = Rect::new(
				row.x + offset,
				row.y,
				(label.len() as u16 + 3).min(row.width - offset),
				row.height,
			);
			let bg = if selected { SELECTED } else { Color::Reset };
			let line = Line::from(vec![
				Span::styled(
					"● ",
					Style::default().fg(if selected {
						Color::Cyan
					} else {
						Color::Rgb(48, 62, 83)
					}),
				),
				Span::styled(
					label,
					Style::default().add_modifier(if selected {
						Modifier::BOLD
					} else {
						Modifier::empty()
					}),
				),
			]);
			frame.render_widget(
				Paragraph::new(line).style(Style::default().bg(bg).add_modifier(
					if self.focus == Focus::Format && selected {
						Modifier::UNDERLINED
					} else {
						Modifier::empty()
					},
				)),
				rect,
			);
			self.format_buttons.push((rect, escaped));
		}
	}
	fn shortcut_lines(&self, width: usize) -> Vec<Line<'static>> {
		let hidden = self
			.explorer
			.as_ref()
			.is_some_and(FileExplorer::show_hidden);
		let entries = [
			("Ctrl+L", "Focus Location".to_owned()),
			("Ctrl+G", "Go back to CWD".to_owned()),
			("Backspace", "Parent".to_owned()),
			(
				"Ctrl+H / Ctrl+.",
				format!("Hidden files: {}", if hidden { "on" } else { "off" }),
			),
			(
				"Ctrl+F",
				format!(
					"Filter: {}",
					if self.projects_only {
						"documents"
					} else {
						"all"
					}
				),
			),
			("Esc", "Cancel".to_owned()),
		];
		let columns = if width >= 72 { 2 } else { 1 };
		let cell_width = width / columns;
		let cell = |index: usize| {
			let (key, description) = &entries[index];
			let key = format!(" {key:<14} ");
			let key: String = key.chars().take(cell_width).collect();
			let remaining = cell_width.saturating_sub(key.width());
			let label: String = format!(" {description}").chars().take(remaining).collect();
			let padding = cell_width.saturating_sub(key.width() + label.width());
			vec![
				Span::styled(key, Style::default().bg(SELECTED).fg(Color::Cyan)),
				Span::raw(label),
				Span::raw(" ".repeat(padding)),
			]
		};
		if columns == 2 {
			(0..3)
				.map(|i| {
					let mut spans = cell(i);
					spans.extend(cell(i + 3));
					Line::from(spans)
				})
				.collect()
		} else {
			(0..6).map(|i| Line::from(cell(i))).collect()
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use ratatui::{Terminal, backend::TestBackend};
	fn key(picker: &mut Picker, code: KeyCode, modifiers: KeyModifiers) -> Outcome {
		picker.event(&Event::Key(KeyEvent::new(code, modifiers)))
	}
	fn selected(picker: &mut Picker, name: &str) {
		let explorer = picker.explorer.as_mut().unwrap();
		let index = explorer
			.files()
			.iter()
			.position(|file| file.name == name)
			.unwrap();
		explorer.set_selected_idx(index);
		picker.focus = Focus::List;
	}
	#[test]
	fn shortcut_encodings_work_from_every_picker_focus() {
		let mut picker = Picker::new(Action::ExportRaw, "", None);
		for focus in [
			Focus::List,
			Focus::Location,
			Focus::Format,
			Focus::Name,
			Focus::Submit,
			Focus::Cancel,
		] {
			picker.focus = focus;
			for code in [
				KeyCode::Char('7'),
				KeyCode::Char('/'),
				KeyCode::Char('?'),
				KeyCode::Char('_'),
			] {
				key(&mut picker, code, KeyModifiers::CONTROL);
				assert!(picker.show_shortcuts);
				key(&mut picker, KeyCode::F(1), KeyModifiers::NONE);
				assert!(!picker.show_shortcuts);
			}
			assert!(picker.focus == focus);
		}
	}
	#[test]
	fn shift_m_selects_middle_of_visible_entries_without_scrolling() {
		let folder = tempfile::tempdir().unwrap();
		for i in 0..30 {
			std::fs::write(folder.path().join(format!("{i:02}.tuxd.json")), "").unwrap();
		}
		let mut picker = Picker::new(Action::Open, "", Some(&folder.path().join("doc.tuxd.json")));
		let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
		terminal
			.draw(|frame| picker.draw(frame, "Open", None))
			.unwrap();
		picker.top = 5;
		let expected = 5 + (picker.list_area.height as usize - 1) / 2;
		key(&mut picker, KeyCode::Char('M'), KeyModifiers::SHIFT);
		assert_eq!(picker.explorer.as_ref().unwrap().selected_idx(), expected);
		assert_eq!(picker.top, 5);
		picker.focus = Focus::Name;
		key(&mut picker, KeyCode::Char('a'), KeyModifiers::CONTROL);
		key(&mut picker, KeyCode::Char('M'), KeyModifiers::SHIFT);
		assert_eq!(picker.name.text, "M");
	}

	#[test]
	fn explorer_focus_vim_navigation_and_shortcut_aliases() {
		let folder = tempfile::tempdir().unwrap();
		std::fs::create_dir(folder.path().join("child")).unwrap();
		std::fs::write(folder.path().join("one.tuxd.json"), "").unwrap();
		let mut picker = Picker::new(Action::Open, "", Some(&folder.path().join("doc.tuxd.json")));
		assert!(picker.focus == Focus::List);
		key(&mut picker, KeyCode::Char('j'), KeyModifiers::NONE);
		assert_eq!(picker.selected().unwrap().0, folder.path().join("child"));
		key(&mut picker, KeyCode::Char('l'), KeyModifiers::NONE);
		assert_eq!(picker.cwd, folder.path().join("child"));
		key(&mut picker, KeyCode::Char('h'), KeyModifiers::NONE);
		assert_eq!(picker.cwd, folder.path());
		key(&mut picker, KeyCode::Char('G'), KeyModifiers::SHIFT);
		assert_eq!(
			picker.selected().unwrap().0,
			folder.path().join("one.tuxd.json")
		);
		key(&mut picker, KeyCode::Char('g'), KeyModifiers::NONE);
		assert_eq!(
			picker.selected().unwrap().0,
			folder.path().join("one.tuxd.json")
		);
		key(&mut picker, KeyCode::Char('g'), KeyModifiers::NONE);
		assert_eq!(picker.explorer.as_ref().unwrap().selected_idx(), 0);
		key(&mut picker, KeyCode::Char('.'), KeyModifiers::CONTROL);
		assert!(picker.explorer.as_ref().unwrap().show_hidden());
		key(&mut picker, KeyCode::Char('/'), KeyModifiers::CONTROL);
		let mut terminal = Terminal::new(TestBackend::new(90, 32)).unwrap();
		terminal
			.draw(|frame| picker.draw(frame, "Open", None))
			.unwrap();
		assert!(picker.help_area.height > 0);
		assert!(
			picker
				.shortcut_lines(86)
				.iter()
				.flat_map(|line| &line.spans)
				.any(|span| span.style.bg == Some(SELECTED))
		);
		key(&mut picker, KeyCode::Char('?'), KeyModifiers::CONTROL);
		assert!(!picker.show_shortcuts);
		picker.focus = Focus::Name;
		key(&mut picker, KeyCode::Char('a'), KeyModifiers::CONTROL);
		for ch in "hjklggG".chars() {
			key(&mut picker, KeyCode::Char(ch), KeyModifiers::NONE);
		}
		assert_eq!(picker.name.text, "hjklggG");
	}
	#[test]
	fn ansi_radios_are_keyboard_and_mouse_accessible() {
		use crossterm::event::MouseEvent;
		let mut picker = Picker::new(Action::ExportRaw, "", None);
		key(&mut picker, KeyCode::Tab, KeyModifiers::NONE);
		assert!(picker.focus == Focus::Format);
		key(&mut picker, KeyCode::Right, KeyModifiers::NONE);
		assert_eq!(picker.action(), Action::ExportEscaped);
		let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
		terminal
			.draw(|frame| picker.draw(frame, "Export ANSI", None))
			.unwrap();
		let rect = picker.format_buttons[0].0;
		picker.event(&Event::Mouse(MouseEvent {
			kind: MouseEventKind::Down(MouseButton::Left),
			column: rect.x,
			row: rect.y,
			modifiers: KeyModifiers::NONE,
		}));
		assert_eq!(picker.action(), Action::ExportRaw);
	}

	#[test]
	fn browse_filter_and_save_in_selected_directory() {
		let folder = tempfile::tempdir().unwrap();
		std::fs::create_dir(folder.path().join("nested")).unwrap();
		for name in ["doc.tuxd.json", "other.txt", ".hidden.tuxd.json"] {
			std::fs::write(folder.path().join(name), "").unwrap();
		}
		let mut picker = Picker::new(
			Action::SaveAs,
			"",
			Some(&folder.path().join("doc.tuxd.json")),
		);
		assert_eq!(picker.explorer.as_ref().unwrap().files().len(), 3); // parent, nested, document
		key(&mut picker, KeyCode::Char('f'), KeyModifiers::CONTROL);
		assert_eq!(picker.explorer.as_ref().unwrap().files().len(), 4);
		key(&mut picker, KeyCode::Char('h'), KeyModifiers::CONTROL);
		assert_eq!(picker.explorer.as_ref().unwrap().files().len(), 5);
		selected(&mut picker, "nested/");
		key(&mut picker, KeyCode::Enter, KeyModifiers::NONE);
		assert_eq!(picker.cwd, folder.path().join("nested"));
		picker.focus = Focus::Name;
		picker.event(&Event::Paste("new 界 document".into()));
		match key(&mut picker, KeyCode::Enter, KeyModifiers::NONE) {
			Outcome::Submit(path) => assert_eq!(path, folder.path().join("nested/new 界 document")),
			_ => panic!("expected a destination"),
		}
	}
	#[test]
	fn location_paste_failure_and_unicode_filename_editing() {
		let folder = tempfile::tempdir().unwrap();
		let mut picker = Picker::new(Action::Open, "", None);
		key(&mut picker, KeyCode::Char('l'), KeyModifiers::CONTROL);
		picker.event(&Event::Paste(folder.path().to_string_lossy().into_owned()));
		key(&mut picker, KeyCode::Enter, KeyModifiers::NONE);
		assert_eq!(picker.cwd, folder.path());
		picker.navigate(folder.path().join("missing"));
		assert_eq!(picker.cwd, folder.path());
		assert!(picker.error.is_some());
		picker.focus = Focus::Name;
		picker.event(&Event::Paste("café界\n".into()));
		key(&mut picker, KeyCode::Backspace, KeyModifiers::NONE);
		assert_eq!(picker.name.text, "café");
		key(&mut picker, KeyCode::Char('a'), KeyModifiers::CONTROL);
		picker.event(&Event::Paste("replacement.tuxd.json".into()));
		assert_eq!(picker.name.text, "replacement.tuxd.json");
		assert_eq!(
			picker.destination(),
			folder.path().join("replacement.tuxd.json")
		);
	}
	#[test]
	fn clipboard_delivery_is_scoped_to_the_picker_and_target_field() {
		let mut picker = Picker::new(Action::Open, "", None);
		let (sender, receiver) = std::sync::mpsc::channel();
		picker.paste = Some((Focus::Name, receiver));
		assert!(!picker.poll_paste());
		sender
			.send(Ok("/tmp/pasted path.tuxd.json".into()))
			.unwrap();
		assert!(picker.poll_paste());
		assert_eq!(
			picker.destination(),
			PathBuf::from("/tmp/pasted path.tuxd.json")
		);
		let (sender, receiver) = std::sync::mpsc::channel();
		picker.paste = Some((Focus::Name, receiver));
		drop(picker);
		assert!(sender.send(Ok("stale".into())).is_err());
	}
	#[test]
	fn mouse_scrollbar_and_small_terminal_remain_usable() {
		use crossterm::event::MouseEvent;
		let folder = tempfile::tempdir().unwrap();
		for i in 0..60 {
			std::fs::write(folder.path().join(format!("{i:02}.tuxd.json")), "").unwrap();
		}
		let mut picker = Picker::new(Action::Open, "", Some(&folder.path().join("doc.tuxd.json")));
		let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
		terminal
			.draw(|frame| picker.draw(frame, "Open", None))
			.unwrap();
		let bar = picker.bar.unwrap();
		picker.event(&Event::Mouse(MouseEvent {
			kind: MouseEventKind::Down(MouseButton::Left),
			column: bar.area.x,
			row: bar.area.bottom() - 1,
			modifiers: KeyModifiers::NONE,
		}));
		terminal
			.draw(|frame| picker.draw(frame, "Open", None))
			.unwrap();
		assert!(picker.top > 0);
		let index = picker.top;
		let path = picker.explorer.as_ref().unwrap().files()[index]
			.path
			.clone();
		let click = Event::Mouse(MouseEvent {
			kind: MouseEventKind::Down(MouseButton::Left),
			column: picker.list_area.x,
			row: picker.list_area.y,
			modifiers: KeyModifiers::NONE,
		});
		picker.event(&click);
		assert_eq!(picker.destination(), path);
		assert!(matches!(picker.event(&click), Outcome::Submit(p) if p == path));
		for (width, height) in [(1, 1), (12, 5), (30, 12), (80, 24)] {
			let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
			terminal
				.draw(|frame| picker.draw(frame, "Open", None))
				.unwrap();
		}
		assert!(matches!(
			key(&mut picker, KeyCode::Esc, KeyModifiers::NONE),
			Outcome::Cancel
		));
	}
}
