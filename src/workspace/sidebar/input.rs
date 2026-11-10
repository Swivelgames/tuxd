use super::{Request, Row, Sidebar};
use crate::shortcuts::ActionId as A;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

impl Sidebar {
	fn reveal_selected(&mut self) {
		let page = self.area.height.saturating_sub(2).max(1) as usize;
		self.top = self.top.min(self.selected);
		if self.selected >= self.top + page {
			self.top = self.selected + 1 - page;
		}
	}

	fn activate(&mut self) -> Request {
		match self.rows.get(self.selected).cloned() {
			Some(Row::OpenFiles) => self.open_files = !self.open_files,
			Some(Row::Root) => self.root_open = !self.root_open,
			Some(Row::Buffer { id, .. }) => return Request::Select(id),
			Some(Row::Entry(entry)) if entry.directory => {
				if let Some(tree) = &mut self.tree
					&& let Err(error) = tree.toggle(&entry.path)
				{
					return Request::Notice(format!("Directory: {error}"));
				}
			}
			Some(Row::Entry(entry)) => return Request::Open(entry.path),
			Some(Row::Spacer) | None => (),
		}
		Request::None
	}

	pub fn action(&mut self, action: A) -> Request {
		let last = self.rows.len().saturating_sub(1);
		let page = self.area.height.saturating_sub(2).max(1) as usize;
		match action {
			A::Activate => return self.activate(),
			A::NextField | A::PreviousField => {
				let root = self.rows.iter().position(|row| matches!(row, Row::Root));
				self.selected = root.filter(|root| self.selected < *root).unwrap_or(0);
			}
			A::CloseBuffer => {
				if let Some(Row::Buffer { id, .. }) = self.rows.get(self.selected) {
					return Request::Close(*id);
				}
			}
			A::ExplorerUp => self.selected = self.selected.saturating_sub(1),
			A::ExplorerDown => self.selected = (self.selected + 1).min(last),
			A::ExplorerFirst => self.selected = 0,
			A::ExplorerLast => self.selected = last,
			A::ExplorerPageUp => self.selected = self.selected.saturating_sub(page),
			A::ExplorerPageDown => self.selected = (self.selected + page).min(last),
			A::ExplorerMiddle => self.selected = (self.top + page / 2).min(last),
			A::ExplorerEnter => match self.rows.get(self.selected) {
				Some(Row::Entry(entry)) if entry.directory && entry.expanded => {
					if matches!(self.rows.get(self.selected + 1), Some(Row::Entry(child)) if child.depth > entry.depth)
					{
						self.selected += 1;
					}
				}
				Some(Row::Entry(entry)) if entry.directory => return self.activate(),
				Some(Row::OpenFiles) if !self.open_files => return self.activate(),
				Some(Row::Root) if !self.root_open => return self.activate(),
				_ => (),
			},
			A::ExplorerParent => match self.rows.get(self.selected).cloned() {
				Some(Row::Entry(entry)) if entry.expanded => return self.activate(),
				Some(Row::Entry(entry)) => {
					self.selected = self.rows[..self.selected]
						.iter()
						.rposition(|row| match row {
							Row::Entry(parent) => parent.depth < entry.depth,
							Row::Root => true,
							_ => false,
						})
						.unwrap_or(0);
				}
				Some(Row::Buffer { .. }) => self.selected = 0,
				Some(Row::OpenFiles) if self.open_files => return self.activate(),
				Some(Row::Root) if self.root_open => return self.activate(),
				_ => (),
			},
			A::PickerHidden | A::PickerFilter | A::RefreshDirectory => {
				return self.refresh_tree(action);
			}
			_ => (),
		}
		self.skip_spacer(matches!(action, A::ExplorerUp | A::ExplorerPageUp));
		self.reveal_selected();
		Request::None
	}

	fn refresh_tree(&mut self, action: A) -> Request {
		let Some(tree) = &mut self.tree else {
			return Request::None;
		};
		let mut filters = tree.filters;
		if action == A::PickerHidden {
			filters.hidden = !filters.hidden;
		}
		if action == A::PickerFilter {
			filters.native_only = !filters.native_only;
		}
		if let Err(error) = tree.refresh_with(filters) {
			return Request::Notice(format!("Directory: {error}"));
		}

		self.selected = 0;
		self.top = 0;
		Request::Notice(format!(
			"Hidden: {} · Files: {}",
			if filters.hidden { "on" } else { "off" },
			if filters.native_only {
				"*.tuxd.json"
			} else {
				"all"
			},
		))
	}

	pub fn mouse(&mut self, mouse: MouseEvent) -> Option<Request> {
		let point = (mouse.column, mouse.row).into();
		if mouse.kind == MouseEventKind::Up(MouseButton::Left) {
			self.drag = None;
		}
		if mouse.kind == MouseEventKind::Drag(MouseButton::Left)
			&& let (Some(bar), Some(grab)) = (self.bars.first(), self.drag)
		{
			self.top = bar.offset_at(bar.coordinate(mouse.column, mouse.row), grab);
			return Some(Request::None);
		}
		if !self.area.contains(point) {
			return None;
		}
		match mouse.kind {
			MouseEventKind::ScrollUp => self.top = self.top.saturating_sub(3),
			MouseEventKind::ScrollDown => self.top = self.top.saturating_add(3),
			MouseEventKind::Down(MouseButton::Left) => {
				if let Some(bar) = self.bars.iter().find(|bar| bar.area.contains(point)) {
					let at = bar.coordinate(mouse.column, mouse.row);
					let grab = bar.grab_at(at);
					self.top = bar.offset_at(at, grab);
					self.drag = Some(grab);
				} else if mouse.row > self.area.y && mouse.row < self.area.bottom() - 1 {
					let index = self.top + (mouse.row - self.area.y - 1) as usize;
					if index < self.rows.len() && !matches!(self.rows[index], Row::Spacer) {
						self.selected = index;
						if mouse.column == self.area.right() - 3
							&& let Row::Buffer { id, .. } = self.rows[index]
						{
							return Some(Request::Close(id));
						}
						return Some(self.activate());
					}
				}
			}
			_ => (),
		}
		Some(Request::None)
	}
}

impl super::super::Workspace {
	pub(in crate::workspace) fn sidebar_mouse(
		&mut self,
		mouse: MouseEvent,
	) -> Option<super::super::Effect> {
		self.refresh_sidebar_rows();
		let request = self.sidebar.mouse(mouse)?;
		if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
			self.cancel_pointer_interactions();
			self.focus = super::super::Pane::Sidebar;
			self.active.session.editing.break_group();
		}
		Some(self.sidebar_request(request))
	}
}
