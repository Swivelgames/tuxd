use super::{Chrome, Workspace};
use crate::workspace::{Pane, TextPane};
use crate::{
	libraries::ansi::{Document, Position},
	shortcuts::{ActionId, Config, InputScope},
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
	let config = Config::default();
	let mut terminal = Terminal::new(TestBackend::new(64, 24)).unwrap();
	let draw = |workspace: &mut Workspace, terminal: &mut Terminal<TestBackend>| {
		terminal
			.draw(|frame| {
				workspace.draw(
					frame,
					Chrome {
						config: &config,
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
