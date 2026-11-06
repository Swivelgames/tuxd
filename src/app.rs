use crate::panel::{Control, Panel};
use crate::styling::Action;
use std::time::Instant;

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

use crate::{
	document::sanitize,
	editor::{EditKind, Editor, Movement},
};

#[derive(Default, PartialEq, Eq)]
pub enum Overlay {
	#[default]
	None,
	Help,
}

pub struct App {
	pub editor_open: bool,
	pub slim: bool,
	pub slim_tab: usize,
	pub pane_toggles: [Rect; 2],
	pub slim_tabs: Vec<(Rect, usize)>,
	pub files: crate::files::Files,
	pub document_path: Option<std::path::PathBuf>,
	pub scrollbars: Vec<crate::scrollbar::Bar>,
	pub scrollbar_drag: Option<(crate::scrollbar::Target, usize)>,
	pub panel: Panel,
	pub code_wrap: bool,
	pub editor_wrap_toggle: Rect,
	pub code_wrap_toggle: Rect,
	pub notice: Option<String>,
	clipboard_result: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
	pub focus: usize,
	pub code_open: bool,
	pub code_top: usize,
	pub code_left: usize,
	pub code_area: Rect,
	pub code_pane: Rect,
	pub editor_pane: Rect,
	pub code_toggle: Rect,
	pub tool_areas: Vec<(Rect, usize)>,
	pub tool_index: usize,
	pub dragging: bool,

	pub editor: Editor,
	pub overlay: Overlay,
	pub editor_area: Rect,
}

impl Default for App {
	fn default() -> Self {
		let mut editor = Editor::default();
		editor.wrap = true;
		Self {
			editor_open: true,
			slim: false,
			slim_tab: 0,
			pane_toggles: [Rect::default(); 2],
			slim_tabs: Vec::new(),
			files: Default::default(),
			document_path: None,
			scrollbars: Vec::new(),
			scrollbar_drag: None,
			panel: Default::default(),
			code_wrap: true,
			editor_wrap_toggle: Default::default(),
			code_wrap_toggle: Default::default(),
			notice: Default::default(),
			clipboard_result: Default::default(),
			focus: Default::default(),
			code_open: true,
			code_top: Default::default(),
			code_left: Default::default(),
			code_area: Default::default(),
			code_pane: Default::default(),
			editor_pane: Default::default(),
			code_toggle: Default::default(),
			tool_areas: Default::default(),
			tool_index: Default::default(),
			dragging: Default::default(),
			editor,
			overlay: Default::default(),
			editor_area: Default::default(),
		}
	}
}

impl App {
	pub fn poll_clipboard(&mut self) -> bool {
		let path_updated =
			if let Some(crate::files::Dialog::Path { picker, .. }) = &mut self.files.dialog {
				picker.poll_paste()
			} else {
				false
			};
		let was_pending = self.clipboard_result.is_some();
		if let Some(receiver) = &self.clipboard_result {
			match receiver.try_recv() {
				Ok(result) => {
					self.notice = Some(match result {
						Ok(()) => "Copied escaped ANSI to clipboard".into(),
						Err(message) => message,
					});
					self.clipboard_result = None;
				}
				Err(std::sync::mpsc::TryRecvError::Disconnected) => {
					self.notice = Some("Clipboard copy failed".into());
					self.clipboard_result = None;
				}
				Err(std::sync::mpsc::TryRecvError::Empty) => (),
			}
		}
		path_updated || (was_pending && self.clipboard_result.is_none())
	}

	fn copy_code(&mut self) {
		if self.clipboard_result.is_some() {
			return;
		}
		let text = crate::styling::clipboard_text(&self.editor.document);
		let (sender, receiver) = std::sync::mpsc::channel();
		self.clipboard_result = Some(receiver);
		self.notice = Some("Copying escaped ANSI…".into());
		std::thread::spawn(move || {
			let _ = sender.send(crate::clipboard::copy(&text));
		});
	}

	fn toggle_wrap(&mut self, code: bool) {
		if code {
			self.code_wrap = !self.code_wrap;
			self.code_left = 0;
			self.code_top = 0;
		} else {
			self.editor.toggle_wrap();
		}
	}

	pub fn toggle_pane(&mut self, code: bool) {
		if code {
			self.code_open = !self.code_open;
			if !self.code_open {
				self.editor_open = true;
				if self.focus == 2 || self.slim {
					self.focus = 0;
				}
			}
		} else {
			self.editor_open = !self.editor_open;
			if !self.editor_open {
				self.code_open = true;
				if self.focus == 0 {
					self.focus = if self.slim && self.slim_tab > 0 { 1 } else { 2 };
				}
			}
		}
		self.dragging = false;
		self.scrollbar_drag = None;
		self.editor.break_group();
	}
	pub fn select_slim_tab(&mut self, tab: usize) {
		self.slim_tab = tab;
		self.code_open = true;
		self.focus = if tab == 0 { 2 } else { 1 };
		self.panel.active_section = tab.checked_sub(1);
		self.panel.top = 0;
		self.tool_index = 0;
		self.panel.gauge_drag = None;
		self.scrollbar_drag = None;
		self.editor.break_group();
	}
	fn workspace_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
		let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
		match key.code {
			KeyCode::F(3) => {
				if self.focus != 1 {
					self.toggle_wrap(self.focus == 2);
				}
				return true;
			}
			KeyCode::Char('c' | 'C') if ctrl && self.focus == 2 => {
				self.copy_code();
				return true;
			}
			KeyCode::F(2) => {
				self.toggle_pane(true);
				return true;
			}
			KeyCode::F(5) => {
				self.toggle_pane(false);
				return true;
			}
			KeyCode::Left | KeyCode::Right
				if self.slim && self.focus != 0 && key.modifiers.contains(KeyModifiers::ALT) =>
			{
				self.select_slim_tab(
					(self.slim_tab + if key.code == KeyCode::Left { 3 } else { 1 }) % 4,
				);
				return true;
			}
			KeyCode::F(6) => {
				if self.slim {
					if self.focus == 0 {
						self.select_slim_tab(self.slim_tab);
					} else {
						self.editor_open = true;
						self.focus = 0;
					}
				} else {
					self.focus = (self.focus + 1) % 3;
					if self.focus == 0 {
						self.editor_open = true;
					}
					if self.focus == 2 {
						self.code_open = true;
					}
					if self.focus == 1 {
						self.reveal_tool();
					}
				}
				self.editor.break_group();
				return true;
			}
			KeyCode::Esc => {
				self.editor.anchor = None;
				self.focus = 0;
				self.editor_open = true;
				self.dragging = false;
				self.editor.break_group();
				return true;
			}
			KeyCode::Char(ch) if ctrl && "bBiIuU".contains(ch) => {
				self.editor.apply_tool(match ch.to_ascii_lowercase() {
					'b' => Action::Bold,
					'i' => Action::Italic,
					_ => Action::Underline,
				});
				return true;
			}
			KeyCode::Char('z' | 'Z' | 'y' | 'Y') if ctrl => return false,
			_ => (),
		}
		if self.focus == 1 {
			self.panel_key(key);
			return true;
		}
		if self.focus == 2 {
			match key.code {
				KeyCode::Up => self.code_top = self.code_top.saturating_sub(1),
				KeyCode::Down => self.code_top = self.code_top.saturating_add(1),
				KeyCode::Left => self.code_left = self.code_left.saturating_sub(4),
				KeyCode::Right => self.code_left = self.code_left.saturating_add(4),
				KeyCode::PageUp => {
					self.code_top = self.code_top.saturating_sub(self.code_area.height as usize)
				}
				KeyCode::PageDown => {
					self.code_top = self.code_top.saturating_add(self.code_area.height as usize)
				}
				KeyCode::Home => {
					self.code_left = 0;
					if ctrl {
						self.code_top = 0;
					}
				}
				KeyCode::End => {
					self.code_left = usize::MAX;
					if ctrl {
						self.code_top = usize::MAX;
					}
				}
				_ => (),
			}
			return true;
		}
		false
	}

	pub fn reveal_tool(&mut self) {
		if let Some(item) = self.panel.items().get(self.tool_index) {
			if let Some(crate::styling::Ink::Indexed(n)) = item.ink {
				let row = n as usize / 7;
				if row < self.panel.indexed_top {
					self.panel.indexed_top = row;
				}
				if row >= self.panel.indexed_top + 5 {
					self.panel.indexed_top = row - 4;
				}
			}
			if let Some(row) = self.panel.display_row(item) {
				if row < self.panel.top {
					self.panel.top = row;
				}
				if row >= self.panel.top + self.panel.height.max(1) {
					self.panel.top = row + 1 - self.panel.height.max(1);
				}
			}
		}
	}

	fn activate_tool(&mut self) {
		let items = self.panel.items();
		let Some(item) = items.get(self.tool_index) else {
			return;
		};
		match item.control {
			Control::Section(section) => {
				self.panel.collapsed[section] = !self.panel.collapsed[section]
			}
			Control::Style(action) => self.editor.apply_tool(action),
			Control::Target(background) => self.panel.background = background,
			Control::Palette(tab) => self.panel.palette = tab,
			Control::GlyphTab(tab) => self.panel.glyph_tab = tab,
			Control::Glyph(ch) => {
				self.focus = 1;
				self.editor
					.insert(&ch.to_string(), EditKind::Atomic, Instant::now());
			}
			Control::Field(_) => {
				self.focus = 1;
				self.panel.replace_field = true;
			}
			Control::Apply => self.apply_rgb(),
		}
		self.reveal_tool();
	}

	fn apply_rgb(&mut self) {
		match self.panel.rgb() {
			Ok(ink) => {
				self.editor.apply_tool(self.panel.color_action(ink));
				self.panel.error = None;
			}
			Err(error) => {
				self.notice = Some(error.clone());
				self.panel.error = Some(error);
			}
		}
	}

	fn panel_key(&mut self, key: crossterm::event::KeyEvent) {
		let mut items = self.panel.items();
		for item in &mut items {
			item.row = self.panel.display_row(item).unwrap_or(usize::MAX);
		}
		self.tool_index = self.tool_index.min(items.len() - 1);
		let current = &items[self.tool_index];
		if let Some(crate::styling::Ink::Indexed(n)) = current.ink
			&& matches!(
				key.code,
				KeyCode::Up | KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown
			) {
			let down = matches!(key.code, KeyCode::Down | KeyCode::PageDown);
			let distance = if matches!(key.code, KeyCode::PageUp | KeyCode::PageDown) {
				35
			} else {
				7
			};
			let next = if down {
				n as i32 + distance
			} else {
				n as i32 - distance
			};
			let control = if next < 0 {
				Control::Palette(1)
			} else if next > 255 {
				Control::Style(self.panel.color_action(crate::styling::Ink::Default))
			} else {
				Control::Style(
					self.panel
						.color_action(crate::styling::Ink::Indexed(next as u8)),
				)
			};
			if let Some(index) = items.iter().position(|item| item.control == control) {
				self.tool_index = index;
			}
			self.reveal_tool();
			return;
		}

		if let Control::Field(field) = current.control {
			if field > 0
				&& matches!(
					key.code,
					KeyCode::Left | KeyCode::Right | KeyCode::Home | KeyCode::End
				) {
				let current = self.panel.fields[field].parse::<u8>().unwrap_or(0);
				let step = if key.modifiers.contains(KeyModifiers::SHIFT) {
					10
				} else {
					1
				};
				let value = match key.code {
					KeyCode::Left => current.saturating_sub(step),
					KeyCode::Right => current.saturating_add(step),
					KeyCode::Home => 0,
					_ => 255,
				};
				self.panel.set_component(field, value);
				return;
			}

			match key.code {
				KeyCode::Enter => {
					self.apply_rgb();
					return;
				}
				KeyCode::Backspace => {
					self.panel.edit(field, "", true);
					return;
				}
				KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
					self.panel.replace_field = true;
					return;
				}
				KeyCode::Char(ch)
					if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
				{
					self.panel.edit(field, &ch.to_string(), false);
					return;
				}
				_ => (),
			}
		}
		match key.code {
			KeyCode::BackTab | KeyCode::Left => {
				self.tool_index = self.tool_index.checked_sub(1).unwrap_or(items.len() - 1)
			}
			KeyCode::Tab | KeyCode::Right => self.tool_index = (self.tool_index + 1) % items.len(),
			KeyCode::Up | KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown => {
				let down = matches!(key.code, KeyCode::Down | KeyCode::PageDown);
				let distance = if matches!(key.code, KeyCode::PageUp | KeyCode::PageDown) {
					self.panel.height.max(1)
				} else {
					1
				};
				let row = if down {
					current.row.saturating_add(distance)
				} else {
					current.row.saturating_sub(distance)
				};
				if let Some((index, _)) = items
					.iter()
					.enumerate()
					.filter(|(_, item)| item.row != usize::MAX)
					.filter(|(_, item)| {
						if down {
							item.row > current.row
						} else {
							item.row < current.row
						}
					})
					.min_by_key(|(_, item)| {
						(item.row.abs_diff(row), item.column.abs_diff(current.column))
					}) {
					self.tool_index = index;
				}
			}
			KeyCode::Home => self.tool_index = 0,
			KeyCode::End => self.tool_index = items.len() - 1,
			KeyCode::Enter | KeyCode::Char(' ') => self.activate_tool(),
			_ => (),
		}
		self.panel.replace_field = true;
		self.reveal_tool();
	}

	fn set_scroll(&mut self, target: crate::scrollbar::Target, offset: usize) {
		use crate::scrollbar::Target;
		match target {
			Target::EditorVertical => self.editor.top = offset,
			Target::EditorHorizontal => self.editor.left = offset,
			Target::CodeVertical => self.code_top = offset,
			Target::CodeHorizontal => self.code_left = offset,
			Target::Tools => self.panel.top = offset,
			Target::Indexed => self.panel.indexed_top = offset,
		}
	}

	fn mouse(&mut self, mouse: crossterm::event::MouseEvent) {
		let point = (mouse.column, mouse.row).into();
		if mouse.kind == MouseEventKind::Up(MouseButton::Left) {
			self.dragging = false;
			self.scrollbar_drag = None;
			self.panel.gauge_drag = None;
			return;
		}
		let gauge = if mouse.kind == MouseEventKind::Drag(MouseButton::Left) {
			self.panel.gauge_drag.and_then(|field| {
				self.panel
					.gauge_areas
					.iter()
					.find(|(_, f)| *f == field)
					.copied()
			})
		} else {
			self.panel
				.gauge_areas
				.iter()
				.find(|(area, _)| area.contains(point))
				.copied()
		};
		if let Some((area, field)) = gauge {
			let value = match mouse.kind {
				MouseEventKind::Down(MouseButton::Left)
				| MouseEventKind::Drag(MouseButton::Left) => {
					self.dragging = false;
					self.panel.gauge_drag = Some(field);
					Some(
						(mouse
							.column
							.saturating_sub(area.x)
							.min(area.width.saturating_sub(1)) as usize
							* 255 / area.width.saturating_sub(1).max(1) as usize) as u8,
					)
				}
				MouseEventKind::ScrollUp => Some(
					self.panel.fields[field]
						.parse::<u8>()
						.unwrap_or(0)
						.saturating_add(1),
				),
				MouseEventKind::ScrollDown => Some(
					self.panel.fields[field]
						.parse::<u8>()
						.unwrap_or(0)
						.saturating_sub(1),
				),
				_ => None,
			};
			if let Some(value) = value {
				self.focus = 1;
				if let Some(index) = self
					.panel
					.items()
					.iter()
					.position(|item| item.control == Control::Field(field))
				{
					self.tool_index = index;
				}
				self.panel.set_component(field, value);
				return;
			}
		}

		if mouse.kind == MouseEventKind::Drag(MouseButton::Left)
			&& let Some((target, grab)) = self.scrollbar_drag
		{
			if let Some(bar) = self
				.scrollbars
				.iter()
				.find(|bar| bar.target == target)
				.copied()
			{
				self.set_scroll(
					target,
					bar.offset_at(bar.coordinate(mouse.column, mouse.row), grab),
				);
			}
			return;
		}
		if let Some(bar) = self
			.scrollbars
			.iter()
			.find(|bar| bar.area.contains(point))
			.copied()
		{
			match mouse.kind {
				MouseEventKind::Down(MouseButton::Left) => {
					self.dragging = false;
					let at = bar.coordinate(mouse.column, mouse.row);
					let (start, size) = bar.thumb();
					let grab = if (start..start + size).contains(&at) {
						at - start
					} else {
						size / 2
					};
					self.scrollbar_drag = Some((bar.target, grab));
					if !(start..start + size).contains(&at) {
						self.set_scroll(bar.target, bar.offset_at(at, grab));
					}
					return;
				}
				MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
					let offset = if mouse.kind == MouseEventKind::ScrollDown {
						bar.offset
							.saturating_add(3)
							.min(bar.content.saturating_sub(bar.viewport))
					} else {
						bar.offset.saturating_sub(3)
					};
					self.set_scroll(bar.target, offset);
					return;
				}
				_ => (),
			}
		}

		if self.dragging
			&& mouse.kind == MouseEventKind::Drag(MouseButton::Left)
			&& self.editor_area.width > 0
			&& self.editor_area.height > 0
		{
			let area = self.editor_area;
			self.editor.click(
				mouse
					.column
					.clamp(area.x, area.right() - 1)
					.saturating_sub(area.x) as usize,
				mouse
					.row
					.clamp(area.y, area.bottom() - 1)
					.saturating_sub(area.y) as usize,
			);
			return;
		}
		if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
			self.dragging = false;
			if self.editor_wrap_toggle.contains(point) {
				self.toggle_wrap(false);
				return;
			}
			if self.code_wrap_toggle.contains(point) {
				self.toggle_wrap(true);
				return;
			}
			if self.pane_toggles[0].contains(point) {
				self.toggle_pane(false);
				return;
			}
			if self.code_toggle.contains(point) || self.pane_toggles[1].contains(point) {
				self.toggle_pane(true);
				return;
			}
			if let Some((_, tab)) = self
				.slim_tabs
				.iter()
				.find(|(rect, _)| rect.contains(point))
				.copied()
			{
				self.select_slim_tab(tab);
				return;
			}

			if let Some((_, tool)) = self
				.tool_areas
				.iter()
				.find(|(area, _)| area.contains(point))
			{
				self.tool_index = *tool;
				self.activate_tool();
				return;
			}
			if self.code_pane.contains(point) {
				self.focus = 2;
				self.editor.break_group();
				return;
			}
			if self.editor_pane.contains(point) {
				self.focus = 0;
				self.editor.click(
					mouse.column.saturating_sub(self.editor_area.x) as usize,
					(mouse.row - self.editor_area.y) as usize,
				);
				self.editor.anchor = Some(self.editor.cursor);
				self.dragging = true;
			}
		}
		if matches!(
			mouse.kind,
			MouseEventKind::ScrollDown | MouseEventKind::ScrollUp
		) {
			let down = mouse.kind == MouseEventKind::ScrollDown;
			if self.panel.indexed_area.contains(point) {
				self.panel.indexed_top = if down {
					(self.panel.indexed_top + 1).min(32)
				} else {
					self.panel.indexed_top.saturating_sub(1)
				};
			} else if self.panel.area.contains(point) {
				self.panel.top = if down {
					self.panel.top.saturating_add(3)
				} else {
					self.panel.top.saturating_sub(3)
				};
			} else if self.editor_pane.contains(point) {
				self.editor.scroll(down);
			} else if self.code_pane.contains(point) {
				self.code_top = if down {
					self.code_top.saturating_add(3)
				} else {
					self.code_top.saturating_sub(3)
				};
			}
		}
	}

	/// Returns true only when the user has chosen to exit.
	pub fn handle(&mut self, event: Event) -> bool {
		if let Some(exit) = self.file_event(&event) {
			return exit;
		}
		match event {
			Event::Key(key) if key.kind != KeyEventKind::Release => {
				self.notice = None;
				let control = key.modifiers.contains(KeyModifiers::CONTROL);
				if self.overlay == Overlay::Help {
					if matches!(key.code, KeyCode::Esc | KeyCode::F(1)) {
						self.overlay = Overlay::None;
					}
					return false;
				}
				if key.code == KeyCode::F(1) {
					self.editor.break_group();
					self.overlay = Overlay::Help;
					return false;
				}
				if self.workspace_key(key) {
					return false;
				}
				let selecting = key.modifiers.contains(KeyModifiers::SHIFT);
				let now = Instant::now();
				let word = key
					.modifiers
					.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
				if word {
					match key.code {
						KeyCode::Left => {
							self.editor.navigate(Movement::WordLeft, selecting);
							return false;
						}
						KeyCode::Right => {
							self.editor.navigate(Movement::WordRight, selecting);
							return false;
						}
						KeyCode::Backspace => {
							self.editor.delete_word(now);
							return false;
						}
						_ => (),
					}
				}
				match (key.code, control) {
					(KeyCode::Char('z' | 'Z'), true) => {
						self.editor
							.undo(key.modifiers.contains(KeyModifiers::SHIFT));
					}
					(KeyCode::Char('y' | 'Y'), true) => self.editor.undo(true),
					(KeyCode::Home, true) => self.editor.navigate(Movement::Start, selecting),
					(KeyCode::End, true) => self.editor.navigate(Movement::Finish, selecting),
					(KeyCode::Left, false) => self.editor.navigate(Movement::Left, selecting),
					(KeyCode::Right, false) => self.editor.navigate(Movement::Right, selecting),
					(KeyCode::Up, false) => self.editor.navigate(Movement::Up, selecting),
					(KeyCode::Down, false) => self.editor.navigate(Movement::Down, selecting),
					(KeyCode::Home, false) => self.editor.navigate(Movement::Home, selecting),
					(KeyCode::End, false) => self.editor.navigate(Movement::End, selecting),
					(KeyCode::PageUp, false) => self.editor.navigate(Movement::PageUp, selecting),
					(KeyCode::PageDown, false) => {
						self.editor.navigate(Movement::PageDown, selecting)
					}
					(KeyCode::Backspace, false) => self.editor.delete(true, now),
					(KeyCode::Delete, false) => self.editor.delete(false, now),
					(KeyCode::Enter, false) => self.editor.insert("\n", EditKind::Atomic, now),
					(KeyCode::Tab, false) => self.editor.insert("\t", EditKind::Atomic, now),
					(KeyCode::Char(ch), false)
						if !key.modifiers.contains(KeyModifiers::ALT) && !ch.is_control() =>
					{
						self.editor.insert(&ch.to_string(), EditKind::Typing, now)
					}
					_ => self.editor.break_group(),
				}
			}
			Event::Paste(text) if self.overlay == Overlay::None && self.focus == 1 => {
				if let Some(item) = self.panel.items().get(self.tool_index)
					&& let Control::Field(field) = item.control
				{
					self.panel.edit(field, text.trim(), false);
				}
			}
			Event::Paste(text) if self.overlay == Overlay::None && self.focus == 0 => self
				.editor
				.insert(&sanitize(&text), EditKind::Atomic, Instant::now()),
			Event::Mouse(mouse) if self.overlay == Overlay::None => self.mouse(mouse),
			Event::Resize(_, _) | Event::FocusLost => self.editor.break_group(),
			_ => (),
		}
		false
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crossterm::event::KeyEvent;

	fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
		Event::Key(KeyEvent::new(code, modifiers))
	}

	fn choose(app: &mut App, control: Control) {
		app.tool_index = app
			.panel
			.items()
			.iter()
			.position(|item| item.control == control)
			.unwrap();
		app.activate_tool();
	}

	#[test]
	fn collapsing_sections_preserves_selection_and_document_history() {
		let mut app = App::default();
		app.handle(Event::Paste("selected".into()));
		app.editor.navigate(Movement::Home, true);
		let selection = app.editor.selection();
		let original = app.editor.document.clone();
		for section in 0..3 {
			choose(&mut app, Control::Section(section));
			assert!(app.panel.collapsed[section]);
		}
		assert_eq!(app.panel.items().len(), 3);
		assert_eq!(app.editor.selection(), selection);
		assert_eq!(app.editor.document, original);
		for section in 0..3 {
			choose(&mut app, Control::Section(section));
		}
		assert!(
			app.panel
				.items()
				.iter()
				.any(|item| item.control == Control::Glyph('┌'))
		);
		app.editor.undo(false);
		assert!(app.editor.document.is_empty());
	}

	#[test]
	fn palettes_preserve_selection_and_rgb_is_staged_until_valid_apply() {
		use crate::styling::Ink;
		let mut app = App::default();
		assert!(app.code_open && app.code_wrap && app.editor.wrap);
		app.handle(Event::Paste("hello".into()));
		app.handle(key(KeyCode::Home, KeyModifiers::SHIFT));
		let selection = app.editor.selection();
		let original = app.editor.document.clone();
		choose(&mut app, Control::Target(true));
		choose(&mut app, Control::Palette(2));
		choose(&mut app, Control::Field(0));
		app.handle(Event::Paste("#00ff80".into()));
		assert_eq!(app.editor.document, original);
		app.handle(key(KeyCode::Enter, KeyModifiers::NONE));
		assert_eq!(
			app.editor.document.styles.at(0).background,
			Ink::Rgb(0, 255, 128)
		);
		assert_eq!(app.editor.selection(), selection);
		let colored = app.editor.document.clone();
		choose(&mut app, Control::Field(2));
		app.handle(Event::Paste("256".into()));
		app.handle(key(KeyCode::Enter, KeyModifiers::NONE));
		assert!(app.panel.error.is_some());
		assert_eq!(app.editor.document, colored);
		// Invalid apply did not add an undo transaction.
		app.handle(key(KeyCode::Char('z'), KeyModifiers::CONTROL));
		assert_eq!(app.editor.document, original);
		app.handle(key(KeyCode::Char('y'), KeyModifiers::CONTROL));
		assert_eq!(app.editor.document, colored);
		choose(&mut app, Control::Style(Action::Clear));
		assert_eq!(app.editor.document, original);
		app.editor.undo(false);
		assert_eq!(app.editor.document, colored);
	}

	#[test]
	fn glyph_replacement_and_repeated_insertion_are_independent_undo_steps() {
		let mut app = App::default();
		app.handle(Event::Paste("界x".into()));
		app.editor.navigate(Movement::Start, false);
		app.editor.navigate(Movement::Right, true);
		app.editor.apply_tool(Action::Dim);
		let before = app.editor.document.clone();
		choose(&mut app, Control::Glyph('┌'));
		assert_eq!(app.focus, 1);
		assert_eq!(app.editor.document.lines, ["┌x"]);
		assert!(app.editor.document.styles.at(0).dim);
		choose(&mut app, Control::Glyph('─'));
		assert_eq!(app.editor.document.lines, ["┌─x"]);
		app.editor.undo(false);
		assert_eq!(app.editor.document.lines, ["┌x"]);
		app.editor.undo(false);
		assert_eq!(app.editor.document, before);
		app.editor.undo(true);
		choose(&mut app, Control::GlyphTab(2));
		choose(&mut app, Control::Glyph('\u{f015}'));
		assert_eq!(app.editor.document.lines, ["┌\u{f015}x"]);
		app.handle(key(KeyCode::Esc, KeyModifiers::NONE));
		assert_eq!(app.focus, 0);
	}

	#[test]
	fn control_and_alt_word_bindings_and_focus_specific_wrap() {
		for modifier in [KeyModifiers::CONTROL, KeyModifiers::ALT] {
			let mut app = App::default();
			app.handle(Event::Paste("one two".into()));
			app.handle(key(KeyCode::Left, modifier));
			assert_eq!(app.editor.cursor.byte, 4);
			app.handle(key(KeyCode::Right, modifier | KeyModifiers::SHIFT));
			assert!(app.editor.selection().is_some());
			app.handle(key(KeyCode::Backspace, modifier));
			assert_eq!(app.editor.document.lines, ["one "]);
			app.handle(key(KeyCode::F(3), KeyModifiers::NONE));
			assert!(!app.editor.wrap);
			assert!(app.code_wrap);
			app.code_open = true;
			app.focus = 2;
			app.handle(key(KeyCode::F(3), KeyModifiers::NONE));
			assert!(!app.editor.wrap && !app.code_wrap);
		}
	}

	#[test]
	fn keyboard_selection_applies_styles_to_text() {
		let mut app = App::default();
		app.handle(Event::Paste("Hello\nworld".into()));
		app.handle(key(KeyCode::Home, KeyModifiers::CONTROL));
		for _ in 0..5 {
			app.handle(key(KeyCode::Right, KeyModifiers::SHIFT));
		}
		assert!(app.editor.selection().is_some());
		app.handle(key(KeyCode::Char('b'), KeyModifiers::CONTROL));
		assert!(app.editor.document.styles.at(0).bold);
	}

	#[test]
	fn quit_is_cancelable_and_paste_cannot_answer_a_dialog() {
		let mut app = App::default();
		app.handle(Event::Paste("hello".into()));
		assert!(!app.handle(key(KeyCode::Char('q'), KeyModifiers::CONTROL)));
		assert!(matches!(
			app.files.dialog,
			Some(crate::files::Dialog::Dirty)
		));
		assert!(!app.handle(Event::Paste("y".into())));
		assert_eq!(app.editor.document.lines, ["hello"]);
		app.handle(key(KeyCode::Esc, KeyModifiers::NONE));
		assert!(app.overlay == Overlay::None);
		app.handle(key(KeyCode::Char('q'), KeyModifiers::CONTROL));
		assert!(app.handle(key(KeyCode::Char('y'), KeyModifiers::NONE)));
	}

	#[test]
	fn help_blocks_edits_and_empty_editor_exits_directly() {
		let mut app = App::default();
		app.handle(key(KeyCode::F(1), KeyModifiers::NONE));
		app.handle(Event::Paste("ignored".into()));
		app.handle(key(KeyCode::Char('x'), KeyModifiers::NONE));
		assert!(app.editor.document.is_empty());
		app.handle(key(KeyCode::Esc, KeyModifiers::NONE));
		assert!(app.handle(key(KeyCode::Char('q'), KeyModifiers::CONTROL)));
	}
}
