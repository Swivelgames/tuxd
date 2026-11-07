//! Inert source preview and independently scrollable import diagnostics.
use crate::{
	ansi_import::Import,
	document::sanitize,
	layout::Layout,
	scrollbar::{self, Bar, Target},
};
use crossterm::event::{Event, MouseButton, MouseEventKind};
use ratatui::{
	Frame,
	layout::{Constraint, Rect},
	style::{Color, Modifier, Style},
	text::{Line, Span},
	widgets::{Block, BorderType, Clear, Paragraph, Row, Table},
};
use std::path::PathBuf;
use unicode_segmentation::UnicodeSegmentation;

pub struct Review {
	clipboard: bool,
	path: PathBuf,
	top: [usize; 2],
	areas: [Rect; 2],
	bars: [Option<Bar>; 2],
	buttons: Vec<(Rect, usize)>,
	focus: usize,
	drag: Option<(usize, usize)>,
}
impl Default for Review {
	fn default() -> Self {
		Self::new(PathBuf::from("Source"))
	}
}
impl Review {
	pub fn clipboard() -> Self {
		let mut review = Self::new(PathBuf::from("Clipboard"));
		review.clipboard = true;
		review
	}
	pub fn source_path(&self) -> &std::path::Path {
		&self.path
	}
	pub fn new(path: PathBuf) -> Self {
		Self {
			clipboard: false,
			path,
			top: [0; 2],
			areas: [Rect::default(); 2],
			bars: [None; 2],
			buttons: Vec::new(),
			focus: 3,
			drag: None,
		}
	}
	#[cfg(test)]
	pub fn has_scrollbar(&self) -> bool {
		self.bars.iter().any(Option::is_some)
	}
	#[cfg(test)]
	pub fn scrolled(&self) -> bool {
		self.top.iter().any(|x| *x > 0)
	}
	pub fn input_scope(&self) -> crate::shortcuts::InputScope {
		if self.focus >= 2 {
			crate::shortcuts::InputScope::ReviewButtons
		} else {
			crate::shortcuts::InputScope::ReviewContent
		}
	}
	pub fn shortcut(&mut self, action: crate::shortcuts::ActionId) -> Option<usize> {
		use crate::shortcuts::ActionId as A;
		let at = self.focus.min(1);
		match action {
			A::Close | A::Abort | A::Negative => return Some(1),
			A::Positive => return Some(0),
			A::NextField => self.focus = (self.focus + 1) % 4,
			A::PreviousField => self.focus = (self.focus + 3) % 4,
			A::FieldLeft | A::FieldRight | A::FieldUp | A::FieldDown if self.focus >= 2 => {
				self.focus = 5 - self.focus
			}
			A::Activate if self.focus >= 2 => return Some(self.focus - 2),
			A::FieldUp => self.top[at] = self.top[at].saturating_sub(1),
			A::FieldDown => self.top[at] = self.top[at].saturating_add(1),
			A::FieldPageUp => self.top[at] = self.top[at].saturating_sub(5),
			A::FieldPageDown => self.top[at] = self.top[at].saturating_add(5),
			A::FieldHome => self.top[at] = 0,
			A::FieldEnd => self.top[at] = usize::MAX,
			_ => (),
		}
		None
	}
	pub fn event(&mut self, event: &Event) -> Option<usize> {
		if let Event::Mouse(mouse) = event {
			let point = (mouse.column, mouse.row).into();
			match mouse.kind {
				MouseEventKind::Up(MouseButton::Left) => self.drag = None,
				MouseEventKind::Drag(MouseButton::Left) => {
					if let Some((i, grab)) = self.drag
						&& let Some(bar) = self.bars[i]
					{
						self.top[i] = bar.offset_at(bar.coordinate(mouse.column, mouse.row), grab);
					}
				}
				MouseEventKind::Down(MouseButton::Left) => {
					for (rect, choice) in &self.buttons {
						if rect.contains(point) {
							return Some(*choice);
						}
					}
					for i in 0..2 {
						if self.areas[i].contains(point) {
							self.focus = i;
						}
						if let Some(bar) = self.bars[i].filter(|bar| bar.area.contains(point)) {
							let at = bar.coordinate(mouse.column, mouse.row);
							let (start, size) = bar.thumb();
							let grab = if (start..start + size).contains(&at) {
								at - start
							} else {
								size / 2
							};
							self.top[i] = bar.offset_at(at, grab);
							self.drag = Some((i, grab));
						}
					}
				}
				MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
					for i in 0..2 {
						if self.areas[i].contains(point) {
							self.focus = i;
							self.top[i] = if mouse.kind == MouseEventKind::ScrollUp {
								self.top[i].saturating_sub(3)
							} else {
								self.top[i].saturating_add(3)
							};
						}
					}
				}
				_ => (),
			}
		}
		None
	}
	pub fn draw(&mut self, frame: &mut Frame, import: &Import) {
		let screen = frame.area();
		let width = screen
			.width
			.saturating_sub(if screen.width > 40 { 4 } else { 0 })
			.min(100);
		let content_width = width.saturating_sub(12) as usize;
		let mut lines = vec![String::new()];
		let mut marks = vec![Vec::new()];
		let mut issue_index = 0;
		for (offset, ch) in import.source.char_indices() {
			if ch == '\n' {
				lines.push(String::new());
				marks.push(Vec::new());
				continue;
			}
			let value = if ch == '\x1b' {
				"\\x1b".to_owned()
			} else if ch.is_control() {
				ch.escape_default().to_string()
			} else {
				ch.to_string()
			};
			while issue_index < import.issues.len() && import.issues[issue_index].end <= offset {
				issue_index += 1;
			}
			let error = import
				.issues
				.get(issue_index)
				.is_some_and(|issue| offset >= issue.start && offset < issue.end);
			lines.last_mut().unwrap().push_str(&value);
			marks
				.last_mut()
				.unwrap()
				.extend(std::iter::repeat_n(error, value.len()));
		}
		let layout = Layout::new(&lines, content_width, true, false);
		let preview_height = layout.rows.len().clamp(1, 5) as u16;
		let error_height = import.issues.len().clamp(1, 5) as u16;
		let height = (14 + preview_height + error_height).min(screen.height);
		let popup = Rect::new(
			(screen.width - width) / 2,
			(screen.height - height) / 2,
			width,
			height,
		);
		frame.render_widget(Clear, popup);
		let block = Block::bordered()
			.border_type(BorderType::Rounded)
			.title(" Unsupported ANSI Sequences ");
		let inner = block.inner(popup);
		frame.render_widget(block, popup);
		self.bars = [None; 2];
		self.buttons.clear();
		self.areas = [Rect::default(); 2];
		if inner.width < 2 || inner.height < 3 {
			return;
		}
		let row = |y: u16| {
			Rect::new(
				inner.x + 1,
				inner.y + y.min(inner.height),
				inner.width.saturating_sub(2),
				u16::from(y < inner.height),
			)
		};
		frame.render_widget(
			Paragraph::new("Any unsupported ANSI sequences will be stripped."),
			row(if screen.height < 18 { 0 } else { 1 }),
		);
		frame.render_widget(
			Paragraph::new(if self.clipboard {
				"The clipboard will be unaffected."
			} else {
				"The original file will be unaffected."
			}),
			row(3),
		);
		let preview = Rect::new(
			inner.x + 1,
			inner.y + if screen.height < 18 { 1 } else { 5 },
			inner.width.saturating_sub(2),
			(preview_height + 2).min(inner.height.saturating_sub(if screen.height < 18 {
				6
			} else {
				10
			})),
		);
		let cwd = std::env::current_dir().ok();
		let display_path = cwd
			.as_deref()
			.and_then(|cwd| self.path.strip_prefix(cwd).ok())
			.unwrap_or(&self.path);
		let block = Block::bordered()
			.border_type(BorderType::Rounded)
			.title(format!(" {} ", sanitize(&display_path.to_string_lossy())));
		let inside = block.inner(preview);
		frame.render_widget(block, preview);
		self.areas[0] = inside;
		self.top[0] = self.top[0].min(layout.rows.len().saturating_sub(inside.height as usize));
		let red = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
		for (visual, row) in layout
			.rows
			.iter()
			.enumerate()
			.skip(self.top[0])
			.take(inside.height as usize)
		{
			let y = inside.y + (visual - self.top[0]) as u16;
			let gutter = inside.width.min(6);
			frame.render_widget(
				Paragraph::new(if row.start == 0 {
					format!("{:>4}  ", row.line + 1)
				} else {
					"      ".into()
				})
				.style(Style::default().bg(Color::Rgb(23, 28, 44))),
				Rect::new(inside.x, y, gutter, 1),
			);
			let spans: Vec<_> = lines[row.line][row.start..row.end]
				.grapheme_indices(true)
				.map(|(at, ch)| {
					Span::styled(
						crate::document::display_grapheme(ch).into_owned(),
						if marks[row.line][row.start + at] {
							red
						} else {
							Style::default()
						},
					)
				})
				.collect();
			frame.render_widget(
				Paragraph::new(Line::from(spans)),
				Rect::new(inside.x + gutter, y, inside.width.saturating_sub(gutter), 1),
			);
		}
		let table_y = preview
			.bottom()
			.saturating_add(u16::from(screen.height >= 18));
		let table_height = (error_height + 2).min(inner.bottom().saturating_sub(table_y + 2));
		let table_area = Rect::new(
			inner.x + 2,
			table_y,
			inner.width.saturating_sub(4),
			table_height,
		);
		self.areas[1] = table_area;
		let visible = table_height.saturating_sub(2) as usize;
		self.top[1] = self.top[1].min(import.issues.len().saturating_sub(visible));
		let rows = import
			.issues
			.iter()
			.skip(self.top[1])
			.take(visible)
			.map(|issue| {
				Row::new(vec![
					ratatui::widgets::Cell::from(
						import.source[issue.start..issue.end]
							.chars()
							.map(|ch| {
								if ch == '\x1b' {
									"\\x1b".to_owned()
								} else if ch.is_control() {
									ch.escape_default().to_string()
								} else {
									ch.to_string()
								}
							})
							.collect::<String>(),
					)
					.style(red),
					ratatui::widgets::Cell::from(issue.reason.clone()),
				])
			});
		let seq_width = import
			.issues
			.iter()
			.map(|i| i.notation.len())
			.max()
			.unwrap_or(4)
			.clamp(6, 24) as u16;
		frame.render_widget(
			Table::new(rows, [Constraint::Length(seq_width), Constraint::Min(1)])
				.column_spacing(3)
				.header(
					Row::new(["Seq.", "Error"])
						.style(Style::default().add_modifier(Modifier::BOLD))
						.bottom_margin(1),
				)
				.style(Style::default().bg(Color::Rgb(23, 28, 44))),
			table_area,
		);
		for (i, area, count, viewport) in [
			(
				0,
				Rect::new(
					preview.right().saturating_sub(1),
					inside.y,
					1,
					inside.height,
				),
				layout.rows.len(),
				inside.height as usize,
			),
			(
				1,
				Rect::new(
					table_area.right(),
					table_area.y + 2,
					1,
					table_height.saturating_sub(2),
				),
				import.issues.len(),
				visible,
			),
		] {
			let mut bars = Vec::new();
			scrollbar::draw(
				frame,
				&mut bars,
				Bar {
					target: Target::Tools,
					area,
					content: count,
					viewport,
					offset: self.top[i],
				},
			);
			self.bars[i] = bars.first().copied();
		}
		let button_y = inner.bottom() - 1;
		let mut x = inner.right().saturating_sub(26).max(inner.x);
		for (i, label) in [
			if self.clipboard {
				"Strip and paste"
			} else {
				"Import anyway"
			},
			"Abort",
		]
		.iter()
		.enumerate()
		{
			let rect = Rect::new(
				x,
				button_y,
				(label.len() as u16 + 2).min(inner.right().saturating_sub(x)),
				1,
			);
			frame.render_widget(
				Paragraph::new(format!(" {label} ")).style(Style::default().fg(Color::Cyan).bg(
					if self.focus == i + 2 {
						Color::Rgb(41, 54, 83)
					} else {
						Color::Reset
					},
				)),
				rect,
			);
			self.buttons.push((rect, i));
			x += label.len() as u16 + 4;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::ansi_import::{self, Mode};
	use ratatui::{Terminal, backend::TestBackend};
	#[test]
	fn independent_five_row_views_and_error_highlighting() {
		let import =
			ansi_import::parse(&"before\\x1b[2J after\n".repeat(12), Mode::Escaped).unwrap();
		let mut review = Review::new(PathBuf::from("sample.txt"));
		let mut terminal = Terminal::new(TestBackend::new(100, 32)).unwrap();
		terminal.draw(|frame| review.draw(frame, &import)).unwrap();
		assert_eq!(review.areas[0].height, 5);
		assert_eq!(review.areas[1].height, 7);
		assert!(review.bars.iter().all(Option::is_some));
		assert!(
			terminal
				.backend()
				.buffer()
				.content
				.iter()
				.any(|cell| cell.fg == Color::Red && cell.modifier.contains(Modifier::BOLD))
		);
		review.focus = 0;
		review.shortcut(crate::shortcuts::ActionId::FieldEnd);
		terminal.draw(|frame| review.draw(frame, &import)).unwrap();
		assert!(review.top[0] > 0);
		assert_eq!(review.top[1], 0);
	}
}
