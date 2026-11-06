use crate::panel::{Control, Panel};
use crate::styling::Action;
use std::time::Instant;

use crossterm::event::{Event, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

use crate::{
	document::sanitize,
	editor::{EditKind, Editor, Movement},
};

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
	clipboard_result: Option<std::sync::mpsc::Receiver<Result<crate::clipboard::Copied, String>>>,
	copy_ansi: bool,
	rgb_paste: Option<(usize, std::sync::mpsc::Receiver<Result<String, String>>)>,
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
	pub shortcuts: crate::shortcuts::Runtime,
	pub settings: Option<crate::shortcuts::settings::Settings>,
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
			copy_ansi: false,
			rgb_paste: None,
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
			shortcuts: Default::default(),
			settings: None,
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
					self.notice = Some(
						result
							.and_then(|copied| crate::clipboard::finish(copied, self.copy_ansi))
							.unwrap_or_else(|error| error),
					);
					self.clipboard_result = None;
				}
				Err(std::sync::mpsc::TryRecvError::Disconnected) => {
					self.notice = Some("Clipboard copy failed".into());
					self.clipboard_result = None;
				}
				Err(std::sync::mpsc::TryRecvError::Empty) => (),
			}
		}
		let mut pasted = false;
		if let Some((field, receiver)) = &self.rgb_paste {
			let field = *field;
			match receiver.try_recv() {
				Ok(Ok(text)) => {
					self.panel.edit(field, text.trim(), false);
					pasted = true;
				}
				Ok(Err(error)) => {
					self.notice = Some(error);
					pasted = true;
				}
				Err(std::sync::mpsc::TryRecvError::Disconnected) => {
					self.notice = Some("Clipboard paste failed".into());
					pasted = true;
				}
				Err(std::sync::mpsc::TryRecvError::Empty) => (),
			}
		}
		if pasted {
			self.rgb_paste = None;
		}
		pasted || path_updated || (was_pending && self.clipboard_result.is_none())
	}

	fn copy_code(&mut self) {
		self.copy_text(crate::styling::clipboard_text(&self.editor.document), true);
	}
	fn copy_text(&mut self, text: String, ansi: bool) {
		if self.clipboard_result.is_some() {
			return;
		}
		let (sender, receiver) = std::sync::mpsc::channel();
		self.clipboard_result = Some(receiver);
		self.copy_ansi = ansi;
		self.notice = Some(
			if ansi {
				"Copying escaped ANSI…"
			} else {
				"Copying selection…"
			}
			.into(),
		);
		std::thread::spawn(move || {
			let _ = sender.send(crate::clipboard::copy(&text));
		});
	}
	fn paste_rgb(&mut self, field: usize) {
		if self.rgb_paste.is_some() {
			return;
		}
		let (sender, receiver) = std::sync::mpsc::channel();
		self.rgb_paste = Some((field, receiver));
		std::thread::spawn(move || {
			let _ = sender.send(crate::clipboard::paste());
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
	pub fn input_scope(&self) -> crate::shortcuts::InputScope {
		use crate::{files::Dialog, shortcuts::InputScope as S};
		match &self.files.dialog {
			Some(Dialog::Path { picker, .. }) => picker.input_scope(),
			Some(Dialog::Menu { .. }) => S::Menu,
			Some(Dialog::Review(_)) => self.files.review.input_scope(),
			Some(_) => S::Prompt,
			None if self.focus == 0 => S::Editor,
			None if self.focus == 2 => S::Code,
			None => self
				.panel
				.items()
				.get(self.tool_index)
				.map_or(S::Tools, |item| match item.control {
					Control::Field(0) => S::Hex,
					Control::Field(_) => S::Channel,
					Control::Target(_) => S::Radio,
					Control::Palette(_) | Control::GlyphTab(_) => S::ToolTabs,
					Control::Glyph(_) => S::Grid,
					_ if item.ink.is_some() => S::Grid,
					_ => S::Tools,
				}),
		}
	}
	fn execute_action(&mut self, action: crate::shortcuts::ActionId) {
		use crate::files::Action as F;
		use crate::shortcuts::ActionId as A;
		let now = Instant::now();
		match action {
			A::New => self.file_action(F::New),
			A::Open => self.file_action(F::Open),
			A::Save => self.file_action(F::Save),
			A::SaveAs => self.file_action(F::SaveAs),
			A::Import => self.file_action(F::ImportEscaped),
			A::Export => self.file_action(F::ExportEscaped),
			A::Quit => self.file_action(F::Quit),
			A::FileMenu => self.open_file_menu(),
			A::ToggleCode => self.toggle_pane(true),
			A::ToggleEditor => self.toggle_pane(false),
			A::EditorWrap => self.toggle_wrap(false),
			A::CodeWrap => self.toggle_wrap(true),
			A::Bold => self.editor.apply_tool(Action::Bold),
			A::Italic => self.editor.apply_tool(Action::Italic),
			A::Underline => self.editor.apply_tool(Action::Underline),
			A::Undo => self.editor.undo(false),
			A::Redo => self.editor.undo(true),
			A::DeleteBefore => self.editor.delete(true, now),
			A::DeleteAfter => self.editor.delete(false, now),
			A::DeleteWord => self.editor.delete_word(now),
			A::InsertNewline => self.editor.insert("\n", EditKind::Atomic, now),
			A::InsertTab => self.editor.insert("\t", EditKind::Atomic, now),
			A::ClearSelection => {
				self.editor.anchor = None;
				self.focus = 0;
				self.editor_open = true;
				self.dragging = false;
				self.editor.break_group();
			}
			A::NextTab | A::PreviousTab => {
				if self.slim {
					self.select_slim_tab(
						(self.slim_tab + if action == A::PreviousTab { 3 } else { 1 }) % 4,
					);
				}
			}
			A::CyclePanes => {
				if self.slim {
					if self.focus == 0 {
						self.select_slim_tab(self.slim_tab);
					} else {
						self.focus = 0;
						self.editor_open = true;
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
			}
			A::FocusEditor => {
				self.focus = 0;
				self.editor_open = true;
				self.editor.break_group();
			}
			A::FocusCode => {
				if self.slim {
					self.select_slim_tab(0);
				} else {
					self.focus = 2;
					self.code_open = true;
				}
				self.editor.break_group();
			}
			A::FocusTools => {
				if self.slim {
					self.select_slim_tab(self.slim_tab.max(1));
				} else {
					self.focus = 1;
					self.reveal_tool();
				}
				self.editor.break_group();
			}
			A::Copy => {
				if self.focus == 2 && self.files.dialog.is_none() {
					self.copy_code();
				} else if let Some(crate::files::Dialog::Path { picker, .. }) = &self.files.dialog {
					if let Some(text) = picker.selected_text() {
						self.copy_text(text, false);
					}
				} else if let Some(item) = self.panel.items().get(self.tool_index)
					&& let Control::Field(field) = item.control
					&& self.panel.replace_field
				{
					self.copy_text(self.panel.fields[field].clone(), false);
				}
			}
			A::CodeUp => self.code_top = self.code_top.saturating_sub(1),
			A::CodeDown => self.code_top = self.code_top.saturating_add(1),
			A::CodeLeft => self.code_left = self.code_left.saturating_sub(4),
			A::CodeRight => self.code_left = self.code_left.saturating_add(4),
			A::CodePageUp => {
				self.code_top = self.code_top.saturating_sub(self.code_area.height as usize)
			}
			A::CodePageDown => {
				self.code_top = self.code_top.saturating_add(self.code_area.height as usize)
			}
			A::CodeHome => self.code_left = 0,
			A::CodeEnd => self.code_left = usize::MAX,
			A::CodeStart => {
				self.code_top = 0;
				self.code_left = 0;
			}
			A::CodeFinish => {
				self.code_top = usize::MAX;
				self.code_left = usize::MAX;
			}
			A::MoveLeft => self.editor.navigate(Movement::Left, false),
			A::SelectLeft => self.editor.navigate(Movement::Left, true),
			A::MoveRight => self.editor.navigate(Movement::Right, false),
			A::SelectRight => self.editor.navigate(Movement::Right, true),
			A::MoveUp => self.editor.navigate(Movement::Up, false),
			A::SelectUp => self.editor.navigate(Movement::Up, true),
			A::MoveDown => self.editor.navigate(Movement::Down, false),
			A::SelectDown => self.editor.navigate(Movement::Down, true),
			A::MoveHome => self.editor.navigate(Movement::Home, false),
			A::SelectHome => self.editor.navigate(Movement::Home, true),
			A::MoveEnd => self.editor.navigate(Movement::End, false),
			A::SelectEnd => self.editor.navigate(Movement::End, true),
			A::MovePageUp => self.editor.navigate(Movement::PageUp, false),
			A::SelectPageUp => self.editor.navigate(Movement::PageUp, true),
			A::MovePageDown => self.editor.navigate(Movement::PageDown, false),
			A::SelectPageDown => self.editor.navigate(Movement::PageDown, true),
			A::MoveStart => self.editor.navigate(Movement::Start, false),
			A::SelectStart => self.editor.navigate(Movement::Start, true),
			A::MoveFinish => self.editor.navigate(Movement::Finish, false),
			A::SelectFinish => self.editor.navigate(Movement::Finish, true),
			A::MoveWordLeft => self.editor.navigate(Movement::WordLeft, false),
			A::SelectWordLeft => self.editor.navigate(Movement::WordLeft, true),
			A::MoveWordRight => self.editor.navigate(Movement::WordRight, false),
			A::SelectWordRight => self.editor.navigate(Movement::WordRight, true),
			_ if self.files.dialog.is_some() => self.file_shortcut(action),
			_ => self.panel_action(action),
		}
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

	fn panel_action(&mut self, action: crate::shortcuts::ActionId) {
		use crate::shortcuts::ActionId as A;
		let items = self.panel.items();
		if items.is_empty() {
			return;
		}
		self.tool_index = self.tool_index.min(items.len() - 1);
		let current = &items[self.tool_index];
		if matches!(action, A::NextField | A::PreviousField) {
			self.tool_index = self
				.panel
				.next_field(self.tool_index, action == A::PreviousField);
		} else if let Control::Field(field) = current.control {
			let value = self.panel.fields[field].parse::<u8>().unwrap_or(0);
			let adjusted = match action {
				A::RangeDecrease => Some(value.saturating_sub(1)),
				A::RangeIncrease => Some(value.saturating_add(1)),
				A::RangeDecrease10 => Some(value.saturating_sub(10)),
				A::RangeIncrease10 => Some(value.saturating_add(10)),
				A::RangeMinimum => Some(0),
				A::RangeMaximum => Some(255),
				_ => None,
			};
			if let Some(value) = adjusted {
				self.panel.set_component(field, value);
				return;
			}
			match action {
				A::RgbApply | A::Submit => self.apply_rgb(),
				A::SelectAll => self.panel.replace_field = true,
				A::FieldDeleteBefore => self.panel.edit(field, "", true),
				A::FieldLeft | A::FieldRight | A::FieldHome | A::FieldEnd => {
					self.panel.move_caret(field, action)
				}
				A::FieldUp | A::FieldDown => {
					self.tool_index = self.panel.next_field(self.tool_index, action == A::FieldUp)
				}
				A::Paste => self.paste_rgb(field),
				_ => (),
			}
			self.reveal_tool();
			return;
		} else if action == A::Activate {
			self.activate_tool();
			return;
		} else {
			let group = current.group();
			let members: Vec<_> = items
				.iter()
				.enumerate()
				.filter(|(_, item)| item.group() == group)
				.collect();
			let at = members
				.iter()
				.position(|(i, _)| *i == self.tool_index)
				.unwrap_or(0);
			let columns = current.columns.max(1);
			let grid = matches!(
				group,
				crate::panel::Group::Colors(_) | crate::panel::Group::Glyphs(_)
			);
			let rows = if matches!(current.ink, Some(crate::styling::Ink::Indexed(_))) {
				5
			} else {
				(self.panel.height / 2).max(1)
			};
			let next = match action {
				A::FieldHome => 0,
				A::FieldEnd => members.len() - 1,
				A::FieldLeft => at.saturating_sub(1),
				A::FieldRight => (at + 1).min(members.len() - 1),
				A::FieldUp if grid => {
					if at >= columns {
						at - columns
					} else {
						at
					}
				}
				A::FieldDown if grid => {
					if at / columns < (members.len() - 1) / columns {
						(at + columns).min(members.len() - 1)
					} else {
						at
					}
				}
				A::FieldPageUp if grid => at.saturating_sub(columns * rows),
				A::FieldPageDown if grid => (at + columns * rows).min(members.len() - 1),
				A::FieldUp | A::FieldDown | A::FieldPageUp | A::FieldPageDown => {
					self.tool_index = self.panel.next_field(
						self.tool_index,
						matches!(action, A::FieldUp | A::FieldPageUp),
					);
					self.reveal_tool();
					return;
				}
				_ => at,
			};
			self.tool_index = members[next].0;
			if next != at
				&& matches!(
					current.control,
					Control::Target(_) | Control::Palette(_) | Control::GlyphTab(_)
				) {
				self.activate_tool();
			}
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
		use crate::shortcuts::{Dispatch, ShortcutKey};
		if self.settings.is_some() {
			self.settings_event(event);
			return false;
		}
		let scope = self.input_scope();
		if let Event::Key(key) = event {
			if key.kind == KeyEventKind::Release {
				return false;
			}
			let decoded = ShortcutKey::from(key);
			if decoded.opener() {
				self.shortcuts.cancel();
				self.editor.break_group();
				self.settings = Some(crate::shortcuts::settings::Settings::for_scope(
					&self.shortcuts.config,
					scope,
				));
				return false;
			}
			self.notice = None;
			match self
				.shortcuts
				.dispatch(decoded.clone(), scope, Instant::now())
			{
				Dispatch::Action(action) => self.execute_action(action),
				Dispatch::Unbound => {
					if let Some(ch) = decoded.printable() {
						if self.files.dialog.is_some() {
							self.file_event(&Event::Paste(ch.to_string()));
						} else if self.focus == 0 {
							self.editor
								.insert(&ch.to_string(), EditKind::Typing, Instant::now());
						} else if self.focus == 1
							&& let Some(item) = self.panel.items().get(self.tool_index)
							&& let Control::Field(field) = item.control
						{
							self.panel.edit(field, &ch.to_string(), false);
						}
					} else {
						self.editor.break_group();
					}
				}
				Dispatch::Pending | Dispatch::Consumed => (),
			}
		} else {
			// A mouse/paste/resize/focus event ends any pending keyboard sequence.
			self.shortcuts.cancel();
			if self.file_event(&event).is_none() {
				match event {
					Event::Paste(text) if self.focus == 1 => {
						if let Some(item) = self.panel.items().get(self.tool_index)
							&& let Control::Field(field) = item.control
						{
							self.panel.edit(field, text.trim(), false);
						}
					}
					Event::Paste(text) if self.focus == 0 => {
						self.editor
							.insert(&sanitize(&text), EditKind::Atomic, Instant::now())
					}
					Event::Mouse(mouse) => self.mouse(mouse),
					Event::Resize(_, _) | Event::FocusLost => self.editor.break_group(),
					_ => (),
				}
			}
		}
		if self.input_scope() != scope {
			self.shortcuts.cancel();
		}
		self.files.exiting()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
		assert!(app.settings.is_none());
		app.handle(key(KeyCode::Char('q'), KeyModifiers::CONTROL));
		assert!(app.handle(key(KeyCode::Char('d'), KeyModifiers::NONE)));
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
