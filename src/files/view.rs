use super::{Action, Dialog, MENU};
use crate::{
	components::scrollbar::{self, Axis, Bar},
	libraries::text::{Layout, display::sanitize},
	theme,
};
use ratatui::{
	Frame,
	layout::Rect,
	style::{Modifier, Style},
	widgets::{BorderType, Clear, Paragraph},
};

fn title(action: Action) -> &'static str {
	match action {
		Action::Open => "Open document",
		Action::Save | Action::SaveAs => "Save document as",
		Action::ImportRaw | Action::ImportEscaped => "Import ANSI",
		Action::ExportRaw | Action::ExportEscaped => "Export ANSI",
		_ => "File",
	}
}

fn draw_dirty(frame: &mut Frame, files: &mut super::Files) {
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
	let block = theme::block().border_type(BorderType::Rounded);
	let inner = block.inner(popup);
	frame.render_widget(block, popup);
	files.buttons.clear();
	files.bar = None;
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
					.fg(theme::accent_color())
					.add_modifier(Modifier::BOLD)
					.bg(if files.choice == i {
						theme::selected_color()
					} else {
						theme::field_color()
					}),
			),
			rect,
		);
		files.buttons.push((rect, i));
		x += label.len() as u16 + 4;
	}
}

fn draw_menu(
	frame: &mut Frame,
	files: &mut super::Files,
	shortcuts: &crate::shortcuts::Config,
	index: usize,
) {
	let area = frame.area();
	let width = area.width.min(
		MENU.iter()
			.map(|(action, label)| {
				label.len() + shortcuts.hint(action.shortcut()).chars().count() + 5
			})
			.max()
			.unwrap_or(33)
			.max(33)
			.min(u16::MAX as usize) as u16,
	);
	let height = area.height.min(12);
	let popup = Rect::new(
		(area.width - width) / 2,
		(area.height - height) / 2,
		width,
		height,
	);
	frame.render_widget(Clear, popup);
	let block = theme::block()
		.border_type(BorderType::Rounded)
		.title(" File ");
	let inner = block.inner(popup);
	frame.render_widget(block, popup);
	files.buttons.clear();
	files.bar = None;
	let rows: Vec<usize> = (0..MENU.len())
		.map(|i| i + usize::from(i >= 2) + usize::from(i >= 4) + usize::from(i >= 6))
		.collect();
	let top = rows[index].saturating_sub(inner.height.saturating_sub(1) as usize);
	for (i, (action, label)) in MENU.iter().enumerate() {
		let hint = shortcuts.hint(action.shortcut());
		let padding = inner
			.width
			.saturating_sub(3 + label.len() as u16 + hint.chars().count() as u16)
			as usize;
		let label = format!("{label}{}{hint}", " ".repeat(padding));
		let y = rows[i];
		if y < top || y - top >= inner.height as usize {
			continue;
		}
		let rect = Rect::new(inner.x, inner.y + (y - top) as u16, inner.width, 1);
		frame.render_widget(
			Paragraph::new(format!("{} {label}", if i == index { ">" } else { " " })).style(
				Style::default().bg(if i == index {
					theme::selected_color()
				} else {
					theme::surface_color()
				}),
			),
			rect,
		);
		files.buttons.push((rect, i));
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

pub fn draw(frame: &mut Frame, files: &mut super::Files, shortcuts: &crate::shortcuts::Config) {
	if let Some(Dialog::Menu { index }) = files.dialog {
		draw_menu(frame, files, shortcuts, index);
		return;
	}
	if matches!(files.dialog, Some(Dialog::Dirty)) {
		draw_dirty(frame, files);
		return;
	}
	if let Some(Dialog::Path { picker }) = &mut files.dialog {
		picker.draw(frame, title(picker.action()), files.error.as_deref());
		return;
	}

	let Some(Dialog::Overwrite { path, .. }) = &files.dialog else {
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
	let block = theme::block()
		.border_type(BorderType::Rounded)
		.title(" Overwrite existing file? ");
	let inner = block.inner(popup);
	frame.render_widget(block, popup);
	files.buttons.clear();
	files.bar = None;
	if inner.width == 0 || inner.height == 0 {
		return;
	}
	let mut lines = Vec::new();
	let mut actions: Vec<(usize, usize)> = Vec::new();
	lines.push(sanitize(&path.to_string_lossy()));
	for (i, label) in ["Overwrite", "Cancel"].iter().enumerate() {
		actions.push((lines.len(), i));
		lines.push(format!(
			"{} {label}",
			if files.choice == i { ">" } else { " " }
		));
	}
	if let Some(error) = &files.error {
		lines.push(format!("Error: {}", sanitize(error)));
	}
	let layout = Layout::new(&lines, inner.width as usize, true, false);
	let count = layout.rows.len();
	files.viewport = inner.height as usize;
	files.scroll = files.scroll.min(count.saturating_sub(files.viewport));
	for (visual, row) in layout
		.rows
		.iter()
		.enumerate()
		.skip(files.scroll)
		.take(files.viewport)
	{
		let rect = Rect::new(
			inner.x,
			inner.y + (visual - files.scroll) as u16,
			inner.width,
			1,
		);
		let text = &lines[row.line][row.start..row.end];
		let style = if actions.iter().any(|(line, _)| *line == row.line) {
			Style::default()
				.fg(theme::accent_color())
				.add_modifier(Modifier::BOLD)
		} else {
			Style::default()
		};
		frame.render_widget(Paragraph::new(text).style(style), rect);
		if let Some((_, choice)) = actions.iter().find(|(line, _)| *line == row.line) {
			files.buttons.push((rect, *choice));
		}
	}
	let mut bars = Vec::new();
	scrollbar::draw(
		frame,
		&mut bars,
		Bar {
			target: Axis::Vertical,
			area: Rect::new(popup.right() - 1, inner.y, 1, inner.height),
			content: count,
			viewport: files.viewport,
			offset: files.scroll,
		},
	);
	files.bar = bars.first().copied();
}
