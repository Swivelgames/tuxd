use super::{Chrome, Workspace};
use crate::workspace::{Pane, TextPane};
use crate::{
	libraries::ansi::{Document, Position},
	shortcuts::{ActionId, InputScope},
};
use ratatui::{Terminal, backend::TestBackend};

#[test]
fn replacement_installs_a_new_session_and_resets_views_without_resetting_preferences() {
	let mut workspace = Workspace::default();
	let previous = workspace.active.session.id;
	workspace.editor_open = false;
	workspace.code_open = false;
	workspace.focus = Pane::Tools;
	workspace.dragging = true;
	workspace.active.editor.surface.viewport.wrap = false;
	workspace.active.editor.surface.viewport.top = 42;
	workspace.active.code.surface.viewport.top = 10;
	workspace.active.code.surface.viewport.left = 20;
	let mut document = Document::default();
	document.insert(Position::default(), "replacement").unwrap();
	workspace.replace(document.clone(), Some("/tmp/source.ansi".into()), true);
	assert_ne!(workspace.active.session.id, previous);
	assert_eq!(workspace.active.session.editing.document, document);
	assert!(workspace.active.session.path.is_none());
	assert_eq!(
		workspace.active.session.origin.as_deref(),
		Some(std::path::Path::new("/tmp/source.ansi"))
	);
	assert!(workspace.active.session.dirty());
	assert!(workspace.editor_open);
	assert!(!workspace.code_open);
	assert_eq!(workspace.input_scope(), InputScope::Editor);
	assert!(!workspace.dragging);
	assert!(!workspace.active.editor.surface.viewport.wrap);
	assert_eq!(workspace.active.editor.surface.viewport.top, 0);
	assert_eq!(workspace.active.code.surface.viewport.top, 0);
	assert_eq!(workspace.active.code.surface.viewport.left, 0);
}

#[test]
fn workspace_composes_slim_tools_and_collapsed_panes_without_application_services() {
	let mut workspace = Workspace::default();
	let mut terminal = Terminal::new(TestBackend::new(64, 24)).unwrap();
	let draw = |workspace: &mut Workspace, terminal: &mut Terminal<TestBackend>| {
		terminal
			.draw(|frame| {
				workspace.draw(
					frame,
					Chrome {
						status: None,
						focused: true,
					},
				)
			})
			.unwrap();
	};
	draw(&mut workspace, &mut terminal);
	workspace.select_slim_tab(2);
	workspace.panel.color.palette = 2;
	draw(&mut workspace, &mut terminal);
	assert_eq!(workspace.panel.active_section, Some(1));
	assert_eq!(workspace.active.code.surface.text.width, 0);
	workspace.toggle_pane(TextPane::Code);
	assert!(workspace.editor_open);
	assert_eq!(workspace.input_scope(), InputScope::Editor);
	workspace.pane_action(ActionId::FocusCode);
	draw(&mut workspace, &mut terminal);
	assert!(workspace.code_open);
	assert_eq!(workspace.slim_tab, 0);
	assert_eq!(workspace.input_scope(), InputScope::Code);
	assert_eq!(workspace.panel.color.palette, 2);
	workspace.toggle_pane(TextPane::Editor);
	draw(&mut workspace, &mut terminal);
	assert!(workspace.active.code.surface.inner.height > 15);
	assert!(!workspace.editor_open);
}

#[test]
fn shared_frame_keeps_status_and_scrollbars_inside_pane_boundaries() {
	for width in [40, 64, 65, 100] {
		let mut workspace = Workspace::default();
		let mut document = Document::default();
		document
			.insert(
				Position::default(),
				&vec!["long line ".repeat(30); 50].join("\n"),
			)
			.unwrap();
		workspace.replace(document, None, false);
		workspace.active.editor.surface.viewport.wrap = false;
		workspace.active.code.surface.viewport.wrap = false;
		let mut terminal = Terminal::new(TestBackend::new(width, 30)).unwrap();
		terminal
			.draw(|frame| {
				workspace.draw(
					frame,
					Chrome {
						status: None,
						focused: true,
					},
				)
			})
			.unwrap();
		let buffer = terminal.backend().buffer();
		assert_eq!(buffer[(0, 0)].symbol(), "╭");
		assert_eq!(buffer[(width - 1, 0)].symbol(), "╮");
		assert_eq!(buffer[(0, 29)].symbol(), "╰");
		assert_eq!(buffer[(width - 1, 29)].symbol(), "╯");
		let divider_y = workspace.pane_toggles[1].y;
		let status: String = (1..width.min(30))
			.map(|x| buffer[(x, divider_y - 1)].symbol())
			.collect();
		assert!(status.starts_with("Ln 1, Col 1"), "{status}");
		assert_eq!(buffer[(0, divider_y)].symbol(), "├");
		for bar in workspace
			.active
			.editor
			.surface
			.bars
			.iter()
			.chain(&workspace.active.code.surface.bars)
		{
			assert!(bar.area.right() < width);
			assert!(bar.area.bottom() < 30);
			assert!(!(bar.area.y..bar.area.bottom()).contains(&(divider_y - 1)));
		}
	}
}
