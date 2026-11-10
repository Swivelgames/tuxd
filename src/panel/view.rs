use crate::theme;
use crate::{
	libraries::ansi::{Action, Ink},
	libraries::editing::Editing,
	panel::Control,
};
use ratatui::{
	Frame,
	buffer::Buffer,
	layout::Rect,
	style::{Modifier, Style},
	text::{Line, Span},
	widgets::{Borders, Paragraph, Tabs, Widget},
};

impl super::Panel {
	pub fn draw(&mut self, frame: &mut Frame, area: Rect, editing: &Editing, has_focus: bool) {
		self.area = area;
		let shared_frame = self.active_section.is_none();
		let sidebar_track = Rect::new(
			area.right()
				.saturating_sub(if shared_frame { 2 } else { 1 }),
			area.y,
			1,
			area.height,
		);
		let area = if shared_frame {
			area
		} else {
			Rect::new(area.x, area.y, area.width.saturating_sub(1), area.height)
		};
		// Content leaves room for the nested track, sidebar track, and shared border.
		let nested_right_inset = if shared_frame { 3 } else { 2 };
		let padding = nested_right_inset + 2;
		self.color.indexed_area = Rect::default();
		let character_start = self.layout().1[2].0;
		let focused = self.items().get(self.selected).map(|item| item.control);
		let character_resized = self.characters.resize(
			(area.height as usize).saturating_sub(character_start),
			area.width.saturating_sub(padding) as usize,
		);
		if character_resized
			&& let Some(control @ (Control::Glyph(_) | Control::CharacterType(_, _))) = focused
		{
			self.characters.reveal_selection();
			self.focus_character_control(control);
		}
		let resized = self.height != area.height as usize;
		self.height = area.height as usize;
		if (resized || character_resized) && has_focus {
			self.reveal();
		}
		let (items, sections) = self.layout();
		let count = sections.last().map_or(0, |(start, height)| start + height);
		self.top = self.top.min(count.saturating_sub(self.height));
		let top = self.top;
		let mut buffer = Buffer::empty(Rect::new(0, 0, area.width, count as u16));
		if shared_frame {
			draw_section_frame(&mut buffer, &sections);
		}
		let width = area.width.saturating_sub(padding) as usize;
		for (index, item) in items.iter().enumerate() {
			let Some(display_row) = self.display_row(item) else {
				continue;
			};
			let focused = has_focus && self.selected == index;
			let state = match item.control {
				Control::Style(Action::Clear) => Some(false),
				Control::Style(action) => editing.tool_state(action),
				Control::Target(bg) => Some(self.color.background == bg),
				Control::Palette(tab) => Some(self.color.palette == tab),
				Control::CharacterScope => Some(self.characters.local_search),
				_ => Some(false),
			};
			let selected = state == Some(true);
			let x = 2 + width * item.column / item.columns;
			let end = 2 + width * (item.column + 1) / item.columns;
			let mut rect = Rect::new(
				x as u16,
				display_row as u16,
				(end - x).saturating_sub(usize::from(item.columns > 1)) as u16,
				1,
			);
			let mut style = Style::default().fg(theme::muted_color()).bg(if focused {
				theme::focus_color()
			} else if selected {
				theme::selected_color()
			} else {
				theme::surface_color()
			});
			if focused {
				style = style.add_modifier(Modifier::UNDERLINED);
			}
			if selected {
				style = style.fg(theme::selected_text_color());
			}
			let mut line = Line::from(item.label.clone());
			match item.control {
				Control::Section(section) => {
					if self.active_section.is_some() {
						continue;
					}
					line = Line::from(format!(
						" {} {} ",
						if self.collapsed(section) {
							"▸"
						} else {
							"▾"
						},
						item.label
					));
					style = style.fg(theme::accent_color()).add_modifier(Modifier::BOLD);
					rect.width = (item.label.len() as u16 + 4).min(area.width.saturating_sub(4));
				}
				Control::Palette(_) => {
					let (labels, active) = (["Basic", "256", "RGB"], self.color.palette);
					let first_tab = index - item.column;
					let focused_tab = has_focus
						.then(|| self.selected.checked_sub(first_tab))
						.flatten();
					rect = draw_tabs(
						&mut buffer,
						area.width,
						rect.y,
						labels,
						active,
						focused_tab,
						item.column,
					);
				}

				Control::CharacterSearch => {
					rect.width = (width as u16).saturating_sub(8);
					let label_width = if rect.width >= 10 { 6 } else { 0 };
					if label_width > 0 {
						Paragraph::new("Find:")
							.style(style)
							.render(Rect::new(rect.x, rect.y, 5, 1), &mut buffer);
					}
					crate::components::field::draw_buffer(
						&self.characters.query,
						&mut buffer,
						Rect::new(
							rect.x + label_width,
							rect.y,
							rect.width.saturating_sub(label_width),
							1,
						),
						focused,
					);
				}
				Control::CharacterGroup => {
					style = style.bg(if focused {
						theme::focus_color()
					} else {
						theme::field_color()
					});
				}
				Control::CharacterScope => {
					rect.x = 2 + (width as u16).saturating_sub(7);
					rect.width = 5.min(width as u16);
				}
				Control::CharacterClear => {
					rect.x = 2 + (width as u16).saturating_sub(1);
					rect.width = 1.min(width as u16);
				}
				Control::Field(field) => {
					if let Some(input) =
						self.color
							.draw_field(field, rect, style, focused, &mut buffer)
						&& display_row >= top
						&& display_row < top + self.height
					{
						self.color.gauge_areas.push((
							Rect::new(
								area.x + input.x,
								area.y + (display_row - top) as u16,
								input.width,
								1,
							),
							field,
						));
					}
				}
				Control::Apply => {
					rect.width = 5;
					style = style
						.bg(theme::surface_color())
						.add_modifier(Modifier::BOLD);
				}

				Control::Target(_) => {
					line = Line::from(vec![
						Span::styled(
							"● ",
							Style::default().fg(if selected {
								theme::accent_color()
							} else {
								theme::inactive_indicator_color()
							}),
						),
						Span::raw(&item.label),
					]);
				}
				Control::Style(
					action @ (Action::Bold
					| Action::Italic
					| Action::Underline
					| Action::Dim
					| Action::Strike
					| Action::Clear),
				) => {
					let step = width.saturating_sub(5) / 5;
					rect.x = (2 + item.column * step) as u16;
					rect.width = if action == Action::Clear {
						rect.x = (2 + width.saturating_sub(5)) as u16;
						5.min(width) as u16
					} else {
						if step >= 4 { 3 } else { 1 }
					};
					style = style.bg(if focused {
						theme::focus_color()
					} else if selected {
						theme::selected_color()
					} else {
						theme::field_color()
					});
					let modifier = match action {
						Action::Bold => Modifier::BOLD,
						Action::Italic => Modifier::ITALIC,
						Action::Underline => Modifier::UNDERLINED,
						Action::Dim => Modifier::DIM,
						Action::Strike => Modifier::CROSSED_OUT,
						_ => Modifier::empty(),
					};
					style = style.add_modifier(modifier);
					line = Line::from(format!(
						"{:^width$}",
						item.label,
						width = rect.width as usize
					));
				}

				Control::Style(
					Action::Foreground(Ink::Default) | Action::Background(Ink::Default),
				) => {
					style = style
						.bg(theme::surface_color())
						.add_modifier(Modifier::BOLD);
					if self.color.palette == 2 {
						rect.x = 9;
					}
					rect.width = item.label.len() as u16;
				}
				Control::CharacterType(_, _) => {
					rect.x = 2;
					rect.width = width as u16;
					style = style.fg(theme::accent_color()).add_modifier(Modifier::BOLD);
				}
				Control::Glyph(_) => {
					rect.x = (2 + item.column) as u16;
					rect.width = unicode_width::UnicodeWidthStr::width(item.label.as_str()) as u16;
				}
				_ => (),
			}
			if let Some(ink) = item.ink {
				if matches!(item.control, Control::Style(_)) {
					rect = self.color.swatch_bounds(item, area.width, rect.y);
				}
				style = super::color::swatch_style(ink, style, focused, selected);
				line = Line::from(format!(
					"{:>width$}",
					item.label,
					width = rect.width as usize
				));
			}

			if !matches!(
				item.control,
				Control::Palette(_) | Control::CharacterSearch | Control::Field(_)
			) {
				Paragraph::new(line).style(style).render(rect, &mut buffer);
			}
			if display_row >= top && display_row < top + self.height {
				self.tool_areas.push((
					Rect::new(
						area.x + rect.x,
						area.y + (display_row - top) as u16,
						rect.width,
						1,
					),
					index,
				));
			}
		}
		if sections[1].1 > 2 && self.color.palette == 1 {
			let first = (sections[1].0 + 5).max(top);
			let end = (sections[1].0 + 14).min(top + self.height);
			if end > first {
				self.color.indexed_area = Rect::new(
					area.x + 1,
					area.y + (first - top) as u16,
					area.width.saturating_sub(nested_right_inset),
					(end - first) as u16,
				);
			}
		}

		if sections[1].1 > 2
			&& self.color.palette == 2
			&& let Some(error) = &self.color.rgb.error
		{
			let row = sections[1].0 + 10;
			Paragraph::new(error.as_str())
				.style(
					Style::default()
						.fg(theme::error_color())
						.bg(theme::surface_color()),
				)
				.render(Rect::new(2, row as u16, width as u16, 1), &mut buffer);
		}

		if sections[2].1 > 2 {
			let (start, _) = sections[2];
			let characters = &self.characters;
			let detail = characters
				.selected_glyph()
				.and_then(|glyph| crate::libraries::characters::CATALOG.get(glyph));
			let detail_row = start + characters.height - 2;
			let detail = detail
				.map(|entry| format!("U+{:04X} · {}", entry.glyph as u32, entry.name))
				.unwrap_or_else(|| {
					if characters.results.is_empty() {
						"No matching characters".into()
					} else {
						String::new()
					}
				});
			Paragraph::new(detail).style(theme::muted()).render(
				Rect::new(2, detail_row as u16, width as u16, 1),
				&mut buffer,
			);
			let first = (start + 5).max(top);
			let end = (start + 5 + characters.rows).min(top + self.height);
			self.characters.area = if end > first {
				Rect::new(
					area.x + 1,
					area.y + (first - top) as u16,
					area.width.saturating_sub(nested_right_inset),
					(end - first) as u16,
				)
			} else {
				Rect::default()
			};
		}
		for row in 0..area.height.min(count.saturating_sub(top) as u16) {
			for column in 0..area.width {
				frame.buffer_mut()[(area.x + column, area.y + row)] =
					buffer[(column, top as u16 + row)].clone();
			}
		}
		use super::scroll::Target;
		use crate::components::scrollbar::{self, Bar};
		// Reserve separate tracks for the sidebar and its nested scrolling controls.
		scrollbar::draw(
			frame,
			&mut self.scrollbars,
			Bar {
				target: Target::Tools,
				area: sidebar_track,
				content: count,
				viewport: self.height,
				offset: top,
			},
		);
		let characters = &self.characters;
		if characters.area.height > 0 {
			scrollbar::draw(
				frame,
				&mut self.scrollbars,
				Bar {
					target: Target::Characters,
					area: Rect::new(
						area.right().saturating_sub(nested_right_inset),
						characters.area.y,
						1,
						characters.area.height,
					),
					content: characters.content_rows,
					viewport: characters.rows,
					offset: characters.top,
				},
			);
		}
		let grid = self.color.indexed_area;
		if grid.height > 0 {
			scrollbar::draw(
				frame,
				&mut self.scrollbars,
				Bar {
					target: Target::Indexed,
					area: Rect::new(
						area.right().saturating_sub(nested_right_inset),
						grid.y,
						1,
						grid.height,
					),
					content: 37,
					viewport: 5,
					offset: self.color.indexed_top,
				},
			);
		}
		self.draw_character_dropdown(frame, area);
	}
}

/// Paint the sidebar rails and connect each visible section header to them.
fn draw_section_frame(buffer: &mut Buffer, sections: &[(usize, usize)]) {
	let area = buffer.area;
	let style = Style::default()
		.fg(theme::border_color())
		.bg(theme::surface_color());
	for row in 0..area.height {
		for x in [0, area.width - 1] {
			buffer[(x, row)].set_symbol("│").set_style(style);
		}
	}

	for &(start, height) in sections {
		if height == 0 {
			continue;
		}
		theme::block()
			.borders(Borders::TOP)
			.border_style(Style::default().fg(theme::border_color()))
			.style(Style::default().bg(theme::surface_color()))
			.render(
				Rect::new(0, start as u16, area.width, height as u16),
				buffer,
			);
		buffer[(0, start as u16)].set_symbol("├");
		buffer[(area.width - 1, start as u16)].set_symbol("┤");
	}
}

/// Paint a tool's inset tab group and return this tab's local hit bounds.
fn draw_tabs(
	buffer: &mut Buffer,
	area_width: u16,
	row: u16,
	labels: [&str; 3],
	active: usize,
	focused_tab: Option<usize>,
	column: usize,
) -> Rect {
	let width = area_width.saturating_sub(4) as usize;
	let mut rect = Rect::new(0, row, 0, 1);
	let padded = width >= labels.iter().map(|label| label.len()).sum::<usize>() + 8;
	let padding = usize::from(padded);
	let offset: usize = labels[..column]
		.iter()
		.map(|label| label.len() + padding * 2 + 1)
		.sum();
	rect.x = (2 + offset) as u16;
	rect.width = (labels[column].len() + padding * 2) as u16;
	if column == 0 {
		Paragraph::new(format!(
			"├{}┤",
			"─".repeat(area_width.saturating_sub(2) as usize)
		))
		.style(Style::default().fg(theme::border_color()))
		.render(Rect::new(0, rect.y, area_width, 1), buffer);
		let titles: Vec<Line> = labels
			.iter()
			.enumerate()
			.map(|(tab, label)| {
				let focus = focused_tab == Some(tab);
				Line::styled(
					*label,
					if focus {
						Style::default()
							.bg(theme::focus_color())
							.add_modifier(Modifier::UNDERLINED)
					} else {
						Style::default()
					},
				)
			})
			.collect();
		let selected_focus = focused_tab == Some(active);
		let tabs_width = labels
			.iter()
			.map(|label| label.len() + padding * 2)
			.sum::<usize>()
			+ 2;
		Tabs::new(titles)
			.select(active)
			.style(
				Style::default()
					.fg(theme::muted_color())
					.bg(theme::surface_color()),
			)
			.highlight_style(
				Style::default()
					.fg(theme::accent_color())
					.bg(if selected_focus {
						theme::focus_color()
					} else {
						theme::selected_color()
					})
					.add_modifier(Modifier::BOLD),
			)
			.divider("•")
			.padding(if padded { " " } else { "" }, if padded { " " } else { "" })
			.render(Rect::new(2, rect.y, tabs_width as u16, 1), buffer);
	}
	rect
}
