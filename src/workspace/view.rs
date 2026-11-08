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
	pub config: &'a crate::shortcuts::Config,
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

fn shortcut_hint(chrome: &Chrome<'_>, action: crate::shortcuts::ActionId, label: &str) -> String {
	let key = chrome.config.hint(action);
	if key.is_empty() {
		String::new()
	} else {
		format!(" · {key} {label}")
	}
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
		let slim = area.width <= 64;
		if self.slim != slim {
			self.slim = slim;
			self.panel.top = 0;
			self.panel.selected = 0;
			if slim && self.focus == Pane::Tools {
				self.select_slim_tab(1);
			} else if slim && self.focus == Pane::Code {
				self.select_slim_tab(0);
			}
		}
		self.panel.active_section = if slim {
			self.slim_tab.checked_sub(1)
		} else {
			None
		};
		self.code.surface.hide();
		self.editor.surface.hide();
		self.code_toggle = Rect::default();
		self.editor_wrap_toggle = Rect::default();
		self.code_wrap_toggle = Rect::default();
		if area.width < 20 || area.height < 5 {
			self.editor.surface.hide();
			self.editor.resize(&self.session.editing, 0, 0);
			frame.render_widget(Paragraph::new("Resize · F1 settings"), area);
			return;
		}
		frame.render_widget(
			Paragraph::new(format!(
				" Terminal UX Designer{}{}{}{}",
				shortcut_hint(&chrome, crate::shortcuts::ActionId::ToggleCode, "code"),
				shortcut_hint(
					&chrome,
					if self.focus == Pane::Code {
						crate::shortcuts::ActionId::CodeWrap
					} else {
						crate::shortcuts::ActionId::EditorWrap
					},
					"wrap"
				),
				shortcut_hint(&chrome, crate::shortcuts::ActionId::FileMenu, "files"),
				shortcut_hint(&chrome, crate::shortcuts::ActionId::CyclePanes, "focus")
			))
			.style(Style::default().fg(theme::accent_color())),
			Rect::new(area.x, area.y, area.width, 1),
		);
		self.code_toggle = Rect::new(area.right() - 11, area.y, 11, 1);
		frame.render_widget(
			Paragraph::new(if self.code_open {
				"[Code: on]"
			} else {
				"[Code: off]"
			}),
			self.code_toggle,
		);
		let tools_width = if area.width >= 65 { 35 } else { 0 };
		let available = area.height - 2;
		let code_height = if !self.code_open {
			2.min(available)
		} else if !self.editor_open {
			available.saturating_sub(2)
		} else if slim && self.slim_tab > 0 {
			(area.height / 2).max(8).min(available.saturating_sub(3))
		} else {
			(area.height / 3).max(4).min(available.saturating_sub(2))
		};
		let body = Rect::new(
			area.x,
			area.y + 1,
			area.width - tools_width,
			area.height - 2 - code_height,
		);
		if code_height > 0 {
			self.draw_code(
				frame,
				Rect::new(body.x, body.bottom(), body.width, code_height),
				slim,
			);
		}
		self.draw_editor(frame, body, chrome.focused);
		let editor = &self.session.editing;
		let status = chrome.status.map(str::to_owned).unwrap_or_else(|| {
			format!(
				" Ln {}, Col {} · {} lines{}{} · F1 settings{}",
				editor.cursor.row + 1,
				column_at(
					&editor.document.lines()[editor.cursor.row],
					editor.cursor.byte
				) + 1,
				editor.document.lines().len(),
				shortcut_hint(&chrome, crate::shortcuts::ActionId::Undo, "undo"),
				shortcut_hint(&chrome, crate::shortcuts::ActionId::Redo, "redo"),
				shortcut_hint(&chrome, crate::shortcuts::ActionId::Quit, "quit"),
			)
		});
		frame.render_widget(
			Paragraph::new(status).style(theme::muted()),
			Rect::new(area.x, area.bottom() - 1, area.width, 1),
		);
		if tools_width > 0 {
			let tool_rect = Rect::new(body.right(), body.y, tools_width, area.height - 2);
			frame.render_widget(Clear, tool_rect);
			self.panel.draw(
				frame,
				tool_rect,
				&self.session.editing,
				self.focus == Pane::Tools,
			);
		} else if self.code_open && self.slim_tab > 0 {
			let tool_rect = Rect::new(
				body.x + 1,
				body.bottom() + 1,
				body.width.saturating_sub(2),
				code_height.saturating_sub(2),
			);
			frame.render_widget(Clear, tool_rect);
			self.panel.draw(
				frame,
				tool_rect,
				&self.session.editing,
				self.focus == Pane::Tools,
			);
			self.code.surface.hide();
			self.code_wrap_toggle = Rect::default();
		}
	}

	fn draw_code(&mut self, frame: &mut Frame, code_rect: Rect, slim: bool) {
		let block = theme::block()
			.border_type(ratatui::widgets::BorderType::Rounded)
			.title(if slim && self.code_open {
				" ▾ "
			} else if slim {
				" ▸ "
			} else if self.code_open {
				" ▾ Code "
			} else {
				" ▸ Code "
			});
		let mut inner = block.inner(code_rect);
		frame.render_widget(block, code_rect);
		self.pane_toggles[1] = Rect::new(
			code_rect.x + 1,
			code_rect.y,
			(if slim { 3 } else { 8 }).min(code_rect.width.saturating_sub(1)),
			1,
		);
		if slim {
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

		if slim && self.slim_tab > 0 {
			inner = Rect::default();
		}

		self.code_wrap_toggle =
			if self.code_open && (!slim || (self.slim_tab == 0 && code_rect.width >= 53)) {
				wrap_button(frame, code_rect, self.code.surface.viewport.wrap)
			} else {
				Rect::default()
			};
		self.code.draw(
			frame,
			code_rect,
			inner,
			&self.session.editing.document,
			self.session.editing.cursor.row,
		);
	}

	fn draw_editor(&mut self, frame: &mut Frame, body: Rect, focused: bool) {
		let name = self
			.session
			.path
			.as_ref()
			.and_then(|p| p.file_name())
			.map(|s| crate::libraries::text::display::sanitize(&s.to_string_lossy()))
			.unwrap_or_else(|| "untitled".into());
		let border = theme::block()
			.border_type(ratatui::widgets::BorderType::Rounded)
			.title(format!(
				" {} Editor · {name}{} ",
				if self.editor_open { "▾" } else { "▸" },
				if self.session.dirty() { " *" } else { "" }
			));
		let inner = border.inner(body);
		frame.render_widget(border, body);
		self.pane_toggles[0] =
			Rect::new(body.x + 1, body.y, 10.min(body.width.saturating_sub(1)), 1);
		self.editor_wrap_toggle = if self.editor_open {
			wrap_button(frame, body, self.editor.surface.viewport.wrap)
		} else {
			Rect::default()
		};
		self.editor.draw(
			frame,
			body,
			inner,
			&self.session.editing,
			focused && self.focus == Pane::Editor,
		);
	}
}
