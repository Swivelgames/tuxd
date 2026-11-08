//! Adapt resolved actions and pointer events to picker operations.
use super::{Focus, Outcome, Picker};
use crate::{components::field, shortcuts::ActionId as A};
use crossterm::event::{Event, KeyCode, MouseButton, MouseEventKind};
use std::time::{Duration, Instant};

impl Picker {
	pub fn shortcut(&mut self, action: A) -> Outcome {
		match action {
			A::Close | A::Abort | A::Negative => return Outcome::Cancel,
			A::Positive => return self.submit(),
			A::NextField | A::PreviousField => {
				self.focus = self
					.focus
					.next(matches!(action, A::PreviousField), self.has_format())
			}
			A::PickerLocation => {
				self.focus = Focus::Location;
				self.location.select_all();
			}
			A::PickerCwd => match std::env::current_dir() {
				Ok(path) => {
					self.navigate(path);
					self.focus = Focus::List;
				}
				Err(error) => self.error = Some(error.to_string()),
			},
			A::Paste if self.paste.is_none() => {
				self.paste = Some((self.focus, self.clipboard.read()));
			}
			A::PickerHidden => {
				if let Some(explorer) = &mut self.explorer {
					if let Err(error) = explorer.set_show_hidden(!explorer.show_hidden()) {
						self.error = Some(error.to_string());
					}
					self.top = 0;
				}
			}
			A::PickerFilter => {
				self.projects_only = !self.projects_only;
				self.filter();
			}
			A::Submit | A::Activate => match self.focus {
				Focus::Location => {
					self.navigate(self.resolve(self.location.text()));
					self.focus = Focus::List;
				}
				Focus::List => return self.choose(),
				Focus::Format => return self.submit(),
				Focus::Cancel => return Outcome::Cancel,
				_ => return self.submit(),
			},
			A::ExplorerParent => {
				if let Some(parent) = self.cwd.parent() {
					self.navigate(parent.to_path_buf());
				}
			}
			A::ExplorerEnter => {
				if let Some((path, true)) = self.selected() {
					self.navigate(path);
				}
			}
			A::ExplorerUp => self.move_selection(KeyCode::Up),
			A::ExplorerDown => self.move_selection(KeyCode::Down),
			A::ExplorerFirst => self.move_selection(KeyCode::Home),
			A::ExplorerLast => self.move_selection(KeyCode::End),
			A::ExplorerPageUp => self.move_selection(KeyCode::PageUp),
			A::ExplorerPageDown => self.move_selection(KeyCode::PageDown),
			A::ExplorerMiddle => self.move_selection(KeyCode::Char('M')),
			A::FieldLeft if self.focus == Focus::Format => self.set_format(false),
			A::FieldRight if self.focus == Focus::Format => self.set_format(true),
			A::FieldLeft | A::FieldUp | A::FieldRight | A::FieldDown
				if matches!(self.focus, Focus::Submit | Focus::Cancel) =>
			{
				self.focus = if self.focus == Focus::Submit {
					Focus::Cancel
				} else {
					Focus::Submit
				};
			}
			_ if self.focus == Focus::Location => {
				if let Some(action) = field::action(action) {
					self.location.apply(action);
				}
			}
			_ if self.focus == Focus::Name => {
				if let Some(action) = field::action(action) {
					self.name.apply(action);
				}
			}
			_ => (),
		}
		Outcome::None
	}

	pub fn event(&mut self, event: &Event) -> Outcome {
		match event {
			Event::Paste(text) => self.insert(self.focus, text),
			Event::Mouse(mouse) => {
				let point = (mouse.column, mouse.row).into();
				match mouse.kind {
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
							let grab = bar.grab_at(at);
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
							self.location.select_all();
						} else if self.name_area.contains(point) {
							self.focus = Focus::Name;
							self.name.select_all();
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
}
