//! Local file picker: directory discovery, selection and destination policy.
//! Input and painting share the owner's row and hit geometry.
mod input;
pub mod shortcuts;
mod view;

use crate::components::field::Field;
use crate::shortcuts::InputScope;
use crate::{components::scrollbar::Bar, files::Action, libraries::text::display::sanitize};
use crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui_explorer::FileExplorer;
use std::{
	path::{Path, PathBuf},
	time::Instant,
};

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
		items[crate::libraries::controls::cycle(at, items.len(), back)]
	}
}

pub struct Selection {
	pub path: PathBuf,
	pub action: Action,
}

pub enum Outcome {
	None,
	Submit(Selection),
	Cancel,
}

pub struct Picker {
	#[cfg(test)]
	test_shortcuts: crate::shortcuts::Runtime,
	explorer: Option<FileExplorer>,
	cwd: PathBuf,
	action: Action,
	location: Field,
	name: Field,
	focus: Focus,
	projects_only: bool,
	import_prompt: bool,
	format_buttons: Vec<(Rect, bool)>,
	error: Option<String>,
	top: usize,
	list_area: Rect,
	location_area: Rect,
	name_area: Rect,
	buttons: Vec<(Rect, Focus)>,
	bar: Option<Bar>,
	drag: Option<usize>,
	last_click: Option<(usize, Instant)>,
	clipboard: crate::libraries::clipboard::Service,
	paste: Option<(Focus, crate::libraries::clipboard::ReadRequest)>,
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
			#[cfg(test)]
			test_shortcuts: Default::default(),
			explorer: None,
			cwd,
			action,
			location: Field::default(),
			name: Field::default(),
			focus: Focus::List,
			projects_only: matches!(action, Action::Open | Action::Save | Action::SaveAs),
			import_prompt: false,
			format_buttons: Vec::new(),
			error: None,
			top: 0,
			list_area: Rect::default(),
			location_area: Rect::default(),
			name_area: Rect::default(),
			buttons: Vec::new(),
			bar: None,
			drag: None,
			last_click: None,
			clipboard: Default::default(),
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
			picker.name.select_all();
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
		if self.name.text().is_empty() {
			PathBuf::new()
		} else {
			self.resolve(self.name.text())
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
		let Some((path, directory)) = self.selected() else {
			return Outcome::None;
		};
		if directory {
			self.navigate(path);
			return Outcome::None;
		}

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
			return Outcome::Submit(Selection {
				path,
				action: self.action,
			});
		}

		self.focus = Focus::Name;
		self.name.select_all();
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
			Outcome::Submit(Selection {
				path,
				action: self.action,
			})
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
		let clean: String = text.chars().filter(|ch| !ch.is_control()).collect();
		if focus == Focus::Location {
			self.location.insert(&clean, usize::MAX);
		} else {
			self.name.insert(&clean, usize::MAX);
			self.focus = Focus::Name;
		}
	}

	pub fn poll_paste(&mut self) -> bool {
		let Some((focus, request)) = &mut self.paste else {
			return false;
		};
		let Some(completion) = request.poll() else {
			return false;
		};
		let focus = *focus;
		self.paste = None;
		match completion.result {
			Ok(text) => {
				self.insert(focus, &text);
				self.error = None;
			}
			Err(crate::libraries::clipboard::Error::WorkerDisconnected) => {
				self.error = Some("Clipboard helper failed".into())
			}
			Err(error) => self.error = Some(crate::features::clipboard_notice::paste_error(error)),
		}
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

	pub fn selected_text(&self) -> Option<String> {
		let field = match self.focus {
			Focus::Location => &self.location,
			Focus::Name => &self.name,
			_ => return None,
		};
		field.selected().then(|| field.text().to_owned())
	}

	pub fn input_scope(&self) -> InputScope {
		match self.focus {
			Focus::Location => InputScope::Location,
			Focus::Name => InputScope::Filename,
			Focus::List => InputScope::Explorer,
			Focus::Format => InputScope::Format,
			_ => InputScope::FileButtons,
		}
	}
}

#[cfg(test)]
mod tests;
