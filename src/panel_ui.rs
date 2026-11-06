use crate::{
	app::App,
	panel::Control,
	styling::{Action, Ink},
};
use ratatui::{
	Frame,
	buffer::Buffer,
	layout::Rect,
	style::{Color, Modifier, Style},
	text::{Line, Span},
	widgets::{Block, BorderType, Gauge, Paragraph, Tabs, Widget},
};

use unicode_width::UnicodeWidthStr;

const BORDER: Color = Color::Rgb(62, 78, 106);
const SURFACE: Color = Color::Reset;
const MUTED: Color = Color::Rgb(136, 150, 179);
const SELECTED: Color = Color::Rgb(41, 54, 83);
const FOCUS: Color = Color::Rgb(53, 70, 96);

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
	app.panel.area = area;
	let sidebar_track = Rect::new(area.right().saturating_sub(1), area.y, 1, area.height);
	let area = Rect::new(area.x, area.y, area.width.saturating_sub(1), area.height);
	app.panel.indexed_area = Rect::default();
	let resized = app.panel.height != area.height as usize;
	app.panel.height = area.height as usize;
	if resized && app.focus == 1 {
		app.reveal_tool();
	}
	let (items, sections) = app.panel.layout();
	let count = sections.last().map_or(0, |(start, height)| start + height);
	app.panel.top = app.panel.top.min(count.saturating_sub(app.panel.height));
	let top = app.panel.top;
	let mut buffer = Buffer::empty(Rect::new(0, 0, area.width, count as u16));
	for &(start, height) in &sections {
		if height == 0 || app.panel.active_section.is_some() {
			continue;
		}
		Block::bordered()
			.border_type(BorderType::Rounded)
			.border_style(Style::default().fg(BORDER))
			.style(Style::default().bg(SURFACE))
			.render(
				Rect::new(0, start as u16, area.width, height as u16),
				&mut buffer,
			);
	}
	let width = area.width.saturating_sub(4) as usize;
	for (index, item) in items.iter().enumerate() {
		let Some(display_row) = app.panel.display_row(item) else {
			continue;
		};
		let focused = app.focus == 1 && app.tool_index == index;
		let state = match item.control {
			Control::Style(Action::Clear) => Some(false),
			Control::Style(action) => app.editor.tool_state(action),
			Control::Target(bg) => Some(app.panel.background == bg),
			Control::Palette(tab) => Some(app.panel.palette == tab),
			Control::GlyphTab(tab) => Some(app.panel.glyph_tab == tab),
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
		let mut style = Style::default().fg(MUTED).bg(if focused {
			FOCUS
		} else if selected {
			SELECTED
		} else {
			SURFACE
		});
		if focused {
			style = style.add_modifier(Modifier::UNDERLINED);
		}
		if selected {
			style = style.fg(Color::Rgb(220, 232, 255));
		}
		let mut line = Line::from(item.label.clone());
		match item.control {
			Control::Section(section) => {
				if app.panel.active_section.is_some() {
					continue;
				}
				line = Line::from(format!(
					" {} {} ",
					if app.panel.collapsed[section] {
						"▸"
					} else {
						"▾"
					},
					item.label
				));
				style = style.fg(Color::Cyan).add_modifier(Modifier::BOLD);
				rect.width = (item.label.len() as u16 + 4).min(area.width.saturating_sub(4));
			}
			Control::Palette(_) | Control::GlyphTab(_) => {
				let labels = if matches!(item.control, Control::Palette(_)) {
					["Basic", "256", "RGB"]
				} else {
					["Box", "Blocks", "Nerd"]
				};
				let active = if matches!(item.control, Control::Palette(_)) {
					app.panel.palette
				} else {
					app.panel.glyph_tab
				};
				let padded = width >= labels.iter().map(|label| label.len()).sum::<usize>() + 8;
				let padding = usize::from(padded);
				let offset: usize = labels[..item.column]
					.iter()
					.map(|label| label.len() + padding * 2 + 1)
					.sum();
				rect.x = (2 + offset) as u16;
				rect.width = (item.label.len() + padding * 2) as u16;
				if item.column == 0 {
					Paragraph::new(format!(
						"├{}┤",
						"─".repeat(area.width.saturating_sub(2) as usize)
					))
					.style(Style::default().fg(BORDER))
					.render(Rect::new(0, rect.y, area.width, 1), &mut buffer);
					let titles: Vec<Line> = labels
						.iter()
						.enumerate()
						.map(|(tab, label)| {
							let focus = app.focus == 1 && app.tool_index == index + tab;
							Line::styled(
								*label,
								if focus {
									Style::default()
										.bg(FOCUS)
										.add_modifier(Modifier::UNDERLINED)
								} else {
									Style::default()
								},
							)
						})
						.collect();
					let selected_focus = app.focus == 1 && app.tool_index == index + active;
					let tabs_width = labels
						.iter()
						.map(|label| label.len() + padding * 2)
						.sum::<usize>() + 2;
					Tabs::new(titles)
						.select(active)
						.style(Style::default().fg(MUTED).bg(SURFACE))
						.highlight_style(
							Style::default()
								.fg(Color::Cyan)
								.bg(if selected_focus { FOCUS } else { SELECTED })
								.add_modifier(Modifier::BOLD),
						)
						.divider("•")
						.padding(if padded { " " } else { "" }, if padded { " " } else { "" })
						.render(Rect::new(2, rect.y, tabs_width as u16, 1), &mut buffer);
				}
			}

			Control::Field(field) => {
				let label = ["Hex: ", "R: ", "G: ", "B: "][field];
				Paragraph::new(label)
					.style(Style::default().fg(MUTED))
					.render(
						Rect::new(rect.x, rect.y, label.len() as u16, 1),
						&mut buffer,
					);
				let input = Rect::new(
					rect.x + label.len() as u16,
					rect.y,
					if field == 0 {
						9.min(rect.width.saturating_sub(11))
					} else {
						rect.width.saturating_sub(label.len() as u16)
					},
					1,
				);
				let background = if focused {
					FOCUS
				} else {
					Color::Rgb(23, 28, 44)
				};
				if field == 0 {
					let value = &app.panel.fields[0];
					let cursor = value[..app.panel.field_cursors[0].min(value.len())].width();
					let scroll = if focused && !app.panel.replace_field {
						(cursor + 1).saturating_sub(input.width as usize)
					} else {
						0
					};
					Paragraph::new(value.clone())
						.scroll((0, scroll as u16))
						.style(style.bg(background))
						.render(input, &mut buffer);
					if focused && !app.panel.replace_field && input.width > 0 {
						buffer[(input.x + (cursor - scroll) as u16, input.y)]
							.set_style(Style::default().add_modifier(Modifier::REVERSED));
					}

					let preview = Rect::new(rect.right().saturating_sub(5), rect.y, 5, 1);
					Paragraph::new(if app.panel.rgb().is_ok() { "" } else { "  ?" })
						.style(
							Style::default()
								.fg(MUTED)
								.bg(app.panel.rgb().map_or(background, |ink| ink.render())),
						)
						.render(preview, &mut buffer);
				} else {
					let value = app.panel.fields[field].parse::<u8>().unwrap_or(0);
					let fill = [
						Color::Reset,
						Color::Rgb(115, 45, 55),
						Color::Rgb(35, 95, 65),
						Color::Rgb(40, 70, 130),
					][field];
					Gauge::default()
						.ratio(value as f64 / 255.0)
						.label(Span::styled(
							format!(
								"{:<width$}",
								app.panel.fields[field],
								width = input.width as usize
							),
							Style::default().fg(Color::White).add_modifier(if focused {
								Modifier::UNDERLINED
							} else {
								Modifier::empty()
							}),
						))
						.gauge_style(Style::default().fg(fill).bg(background))
						.render(input, &mut buffer);
					if display_row >= top && display_row < top + app.panel.height {
						app.panel.gauge_areas.push((
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
			}
			Control::Apply => {
				rect.width = 5;
				style = style.bg(SURFACE).add_modifier(Modifier::BOLD);
			}

			Control::Target(_) => {
				line = Line::from(vec![
					Span::styled(
						"● ",
						Style::default().fg(if selected {
							Color::Cyan
						} else {
							Color::Rgb(54, 65, 86)
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
					FOCUS
				} else if selected {
					SELECTED
				} else {
					Color::Rgb(23, 28, 44)
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

			Control::Style(Action::Foreground(Ink::Default) | Action::Background(Ink::Default)) => {
				style = style.bg(SURFACE).add_modifier(Modifier::BOLD);
				if app.panel.palette == 2 {
					rect.x = 9;
				}
				rect.width = item.label.len() as u16;
			}
			Control::Glyph(_) => {
				rect.x += 1;
				rect.width =
					unicode_width::UnicodeWidthStr::width(item.label.as_str()).max(1) as u16;
			}
			_ => (),
		}
		if let Some(ink) = item.ink {
			if matches!(item.control, Control::Style(_)) {
				let swatch_width = area.width.saturating_sub(4) as usize;
				let cell_width = swatch_width / item.columns;
				let button_width = (if app.panel.palette == 0 { 2 } else { 3 })
					.min(cell_width)
					.max(1);
				let preferred_gap = if app.panel.palette == 0 { 2 } else { 1 };
				let gap = preferred_gap.min(
					swatch_width.saturating_sub(button_width * item.columns) / (item.columns - 1),
				);
				rect.x = (2 + item.column * (button_width + gap)) as u16;
				rect.width = button_width as u16;
			}
			let light = match ink {
				Ink::Rgb(r, g, b) => r as u32 * 299 + g as u32 * 587 + b as u32 * 114 > 140000,
				Ink::Ansi(n) | Ink::Indexed(n) => {
					matches!(n, 3 | 6 | 7 | 10..=15)
						|| (16..232).contains(&n) && (n - 16) / 6 % 6 >= 3
						|| n >= 246
				}
				_ => false,
			};
			style = style
				.bg(ink.render())
				.fg(if light { Color::Black } else { Color::White });
			line = Line::from(format!(
				"{:>width$}",
				item.label,
				width = rect.width as usize
			));
			if selected {
				style = style.add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
			}
			if focused {
				style = style.add_modifier(Modifier::REVERSED);
			}
		}
		if !matches!(
			item.control,
			Control::Palette(_) | Control::GlyphTab(_) | Control::Field(_)
		) {
			Paragraph::new(line).style(style).render(rect, &mut buffer);
		}
		if display_row >= top && display_row < top + app.panel.height {
			app.tool_areas.push((
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
	if sections[1].1 > 2 && app.panel.palette == 1 {
		let first = (sections[1].0 + 5).max(top);
		let end = (sections[1].0 + 14).min(top + app.panel.height);
		if end > first {
			app.panel.indexed_area = Rect::new(
				area.x + 1,
				area.y + (first - top) as u16,
				area.width.saturating_sub(2),
				(end - first) as u16,
			);
		}
	}

	if sections[1].1 > 2 && app.panel.palette == 2 && app.panel.error.is_some() {
		let row = sections[1].0 + 10;
		Paragraph::new(app.panel.error.as_deref().unwrap_or("Enter to apply"))
			.style(
				Style::default()
					.fg(if app.panel.error.is_some() {
						Color::LightRed
					} else {
						MUTED
					})
					.bg(SURFACE),
			)
			.render(Rect::new(2, row as u16, width as u16, 1), &mut buffer);
	}
	if sections[2].1 > 2 {
		let detail = items.get(app.tool_index).and_then(|item| {
			if let Control::Glyph(ch) = item.control {
				app.panel
					.glyphs()
					.iter()
					.find(|(glyph, _)| *glyph == ch)
					.map(|(_, name)| format!("U+{:04X} · {name}", ch as u32))
			} else {
				None
			}
		});
		let (start, height) = sections[2];
		Paragraph::new(detail.as_deref().unwrap_or("Choose a character"))
			.style(Style::default().fg(MUTED).bg(SURFACE))
			.render(
				Rect::new(2, (start + height - 3) as u16, width as u16, 1),
				&mut buffer,
			);
	}
	for row in 0..area.height.min(count.saturating_sub(top) as u16) {
		for column in 0..area.width {
			frame.buffer_mut()[(area.x + column, area.y + row)] =
				buffer[(column, top as u16 + row)].clone();
		}
	}
	use crate::scrollbar::{self, Bar, Target};
	// Keep the sidebar track outside the cards; the indexed viewport has its own inset track.
	scrollbar::draw(
		frame,
		&mut app.scrollbars,
		Bar {
			target: Target::Tools,
			area: sidebar_track,
			content: count,
			viewport: app.panel.height,
			offset: top,
		},
	);
	let grid = app.panel.indexed_area;
	if grid.height > 0 {
		scrollbar::draw(
			frame,
			&mut app.scrollbars,
			Bar {
				target: Target::Indexed,
				area: Rect::new(area.right().saturating_sub(2), grid.y, 1, grid.height),
				content: 37,
				viewport: 5,
				offset: app.panel.indexed_top,
			},
		);
	}
}
