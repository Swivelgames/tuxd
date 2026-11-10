use crate::theme;
use crate::workspace::Pane;
use ratatui::{
	Frame,
	layout::Rect,
	style::{Modifier, Style},
	widgets::{Block, Clear, Paragraph},
};

use crate::libraries::text::display::column_at;

pub struct Chrome<'a> {
	pub status: Option<&'a str>,
	pub focused: bool,
}

fn wrap_button(frame: &mut Frame, body: Rect, enabled: bool) -> Rect {
	if body.width < 16 || body.height == 0 {
		return Rect::default();
	}
	let area = Rect::new(body.right() - 12, body.y, 11, 1);
	frame.render_widget(
		Paragraph::new(if enabled { "[Wrap: on]" } else { "[Wrap: off]" })
			.style(Style::default().fg(theme::accent_color())),
		area,
	);
	area
}

fn draw_sidebar_divider(frame: &mut Frame, area: Rect, divider_x: u16) {
	let style = Style::default().fg(theme::border_color());
	for y in area.y + 1..area.bottom() - 1 {
		if frame.buffer_mut()[(divider_x, y)].symbol() == "┤" {
			continue;
		}
		frame.render_widget(
			Paragraph::new("│").style(style),
			Rect::new(divider_x, y, 1, 1),
		);
	}
	frame.render_widget(
		Paragraph::new("┬").style(style),
		Rect::new(divider_x, area.y, 1, 1),
	);
	frame.render_widget(
		Paragraph::new("┴").style(style),
		Rect::new(divider_x, area.bottom() - 1, 1, 1),
	);
}

impl super::Workspace {
	pub fn draw(&mut self, frame: &mut Frame, chrome: Chrome<'_>) {
		let area = frame.area();
		frame.render_widget(
			Block::default().style(
				Style::default()
					.fg(theme::text_color())
					.bg(theme::surface_color()),
			),
			area,
		);
		frame.render_widget(Clear, area);

		self.panel.hide();
		self.pane_toggles = [Rect::default(); 2];
		self.slim_tabs.clear();
		self.tab_hits.clear();
		self.tab_close_hits.clear();
		self.sidebar_overlay = area.width <= 64;
		if self.sidebar_overlay && self.focus != Pane::Sidebar {
			self.sidebar.visible = false;
		}
		let sidebar_width = if self.sidebar.visible {
			26.min(area.width.saturating_sub(4))
		} else {
			0
		};
		let docked = sidebar_width > 0 && area.width > 64;
		let content = if docked {
			Rect::new(
				area.x + sidebar_width - 1,
				area.y,
				area.width - sidebar_width + 1,
				area.height,
			)
		} else {
			area
		};
		self.update_arrangement(content.width <= 64);
		let slim = self.slim;
		self.active.code.surface.hide();
		self.active.editor.surface.hide();
		self.code_wrap_toggle = Rect::default();
		self.sidebar_toggle = Rect::default();
		if !self.sidebar.visible {
			self.sidebar.hide_geometry();
		}

		if area.width < 20 || area.height < 5 {
			self.sidebar.hide_geometry();
			self.active
				.editor
				.resize(&self.active.session.editing, 0, 0);
			frame.render_widget(Paragraph::new("Resize · F1 settings"), area);
			return;
		}

		let layout = super::layout::Layout::new(
			content,
			self.editor_open,
			self.code_open,
			slim && self.slim_tab > 0,
		);
		frame.render_widget(
			theme::block().border_type(ratatui::widgets::BorderType::Rounded),
			area,
		);
		if let Some(tools) = layout.tools {
			draw_sidebar_divider(frame, area, tools.x);
			self.panel.draw(
				frame,
				tools,
				&self.active.session.editing,
				self.focus == Pane::Tools,
			);
		}
		self.draw_editor(frame, layout.editor, chrome.focused);
		self.draw_code(frame, layout.code, slim);

		if slim && self.code_open && self.slim_tab > 0 {
			let tool_rect = Rect::new(
				layout.code.x + 1,
				layout.code.y + 1,
				layout.code.width - 2,
				layout.code.height.saturating_sub(2),
			);
			self.panel.draw(
				frame,
				tool_rect,
				&self.active.session.editing,
				self.focus == Pane::Tools,
			);
			self.active.code.surface.hide();
			self.code_wrap_toggle = Rect::default();
		}

		self.sidebar_toggle = Rect::new(area.x + 2, area.y, 7, 1);
		if !self.sidebar.visible {
			frame.render_widget(
				Paragraph::new(" tuxd ›").style(Style::default().fg(theme::accent_color())),
				self.sidebar_toggle,
			);
			self.draw_tabs(
				frame,
				Rect::new(
					area.x + 9,
					area.y,
					layout.editor.width.saturating_sub(10),
					1,
				),
			);
		}
		self.draw_status(frame, layout.status, chrome.status);
		if self.sidebar.visible {
			self.draw_directory_sidebar(
				frame,
				Rect::new(area.x, area.y, sidebar_width, area.height),
			);
			if docked {
				draw_sidebar_divider(frame, area, content.x);
				let symbol = if frame.buffer_mut()[(content.x, layout.code.y)].symbol() == "┤" {
					"┼"
				} else {
					"├"
				};
				frame.render_widget(
					Paragraph::new(symbol).style(Style::default().fg(theme::border_color())),
					Rect::new(content.x, layout.code.y, 1, 1),
				);
			}
		}
	}

	fn draw_status(&mut self, frame: &mut Frame, area: Rect, notice: Option<&str>) {
		let editor = &self.active.session.editing;
		let status = notice.map(str::to_owned).unwrap_or_else(|| {
			format!(
				"Ln {}, Col {} · {} lines",
				editor.cursor.row + 1,
				column_at(
					&editor.document.lines()[editor.cursor.row],
					editor.cursor.byte
				) + 1,
				editor.document.lines().len(),
			)
		});
		let status = if self.editor_open {
			status
		} else {
			format!("▸ Editor · {status}")
		};
		frame.render_widget(Paragraph::new(status).style(theme::muted()), area);
		if !self.editor_open {
			self.pane_toggles[0] = area;
		}
	}

	fn draw_code(&mut self, frame: &mut Frame, code_rect: Rect, slim: bool) {
		let block = theme::block()
			.borders(ratatui::widgets::Borders::TOP)
			.title(if slim && self.code_open {
				"  ▾ "
			} else if slim {
				"  ▸ "
			} else if self.code_open {
				"  ▾ Code "
			} else {
				"  ▸ Code "
			});
		let (surface, mut inner) =
			super::layout::surface_rects(code_rect, self.active.code.surface.viewport.wrap);
		let junction = if frame.buffer_mut()[(code_rect.right() - 1, code_rect.y)].symbol() == "├"
		{
			"┼"
		} else {
			"┤"
		};
		frame.render_widget(block, code_rect);
		let style = Style::default().fg(theme::border_color());
		frame.render_widget(
			Paragraph::new("├").style(style),
			Rect::new(code_rect.x, code_rect.y, 1, 1),
		);
		frame.render_widget(
			Paragraph::new(junction).style(style),
			Rect::new(code_rect.right() - 1, code_rect.y, 1, 1),
		);
		self.pane_toggles[1] = Rect::new(
			code_rect.x + 1,
			code_rect.y,
			(if slim { 3 } else { 8 }).min(code_rect.width.saturating_sub(1)),
			1,
		);
		if slim {
			self.draw_slim_tabs(frame, code_rect);
		}

		if slim && self.slim_tab > 0 {
			inner = Rect::default();
		}

		self.code_wrap_toggle =
			if self.code_open && (!slim || (self.slim_tab == 0 && code_rect.width >= 53)) {
				wrap_button(frame, code_rect, self.active.code.surface.viewport.wrap)
			} else {
				Rect::default()
			};
		self.active.code.draw(
			frame,
			surface,
			inner,
			&self.active.session.editing.document,
			self.active.session.editing.cursor.row,
		);
	}

	fn draw_slim_tabs(&mut self, frame: &mut Frame, code_rect: Rect) {
		let labels = if code_rect.width < 40 {
			["Code", "Style", "Color", "Chars"]
		} else {
			["Code", "Style", "Color", "Characters"]
		};
		let end = if self.code_open && self.slim_tab == 0 && code_rect.width >= 53 {
			code_rect.right() - 13
		} else {
			code_rect.right().saturating_sub(1)
		};
		let start = if code_rect.width < 35 {
			self.slim_tab.saturating_sub(1)
		} else {
			0
		};
		let mut x = code_rect.x + 4;
		for (i, label) in labels.iter().enumerate().skip(start) {
			let width = label.len() as u16;
			if x + width > end {
				break;
			}
			let rect = Rect::new(x, code_rect.y, width, 1);
			frame.render_widget(
				Paragraph::new(*label).style(
					Style::default()
						.fg(theme::accent_color())
						.bg(if self.slim_tab == i {
							theme::selected_color()
						} else {
							theme::surface_color()
						})
						.add_modifier(if self.slim_tab == i {
							Modifier::BOLD
						} else {
							Modifier::empty()
						}),
				),
				rect,
			);
			self.slim_tabs.push((rect, i));
			x += width;
			if x < end {
				frame.render_widget(Paragraph::new(" "), Rect::new(x, code_rect.y, 1, 1));
			}
			if i < 3 && x + 3 + labels[i + 1].len() as u16 <= end {
				frame.render_widget(Paragraph::new(" · "), Rect::new(x, code_rect.y, 3, 1));
				x += 3;
			} else {
				break;
			}
		}
	}

	fn draw_editor(&mut self, frame: &mut Frame, body: Rect, focused: bool) {
		let (surface, inner) =
			super::layout::surface_rects(body, self.active.editor.surface.viewport.wrap);
		self.active.editor.draw(
			frame,
			surface,
			inner,
			&self.active.session.editing,
			focused && self.focus == Pane::Editor,
		);
	}
}
