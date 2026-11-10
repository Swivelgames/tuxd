use super::*;
use crate::{
	libraries::ansi::Document,
	shortcuts::ActionId as A,
	workspace::{Chrome, Pane, Workspace},
};
use ratatui::{Terminal, backend::TestBackend};
use std::fs;

fn draw(workspace: &mut Workspace, width: u16, height: u16) {
	let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
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
}

#[test]
fn discovery_is_lazy_filtered_sorted_and_does_not_follow_directory_symlinks() {
	let folder = tempfile::tempdir().unwrap();
	let root = folder.path();
	fs::create_dir(root.join("nested")).unwrap();
	fs::write(root.join("nested/child.tuxd.json"), "").unwrap();
	for name in [
		"z.tuxd.json",
		"a.tuxd.json",
		".hidden.tuxd.json",
		"notes.txt",
	] {
		fs::write(root.join(name), "").unwrap();
	}
	std::os::unix::fs::symlink(root, root.join("cycle")).unwrap();
	let mut tree = Tree::open(root).unwrap();
	let names: Vec<_> = tree
		.entries
		.iter()
		.map(|entry| entry.path.file_name().unwrap().to_str().unwrap())
		.collect();
	assert_eq!(names, ["nested", "a.tuxd.json", "z.tuxd.json"]);
	tree.toggle(&root.join("nested")).unwrap();
	assert_eq!(tree.entries[1].depth, 1);
	assert!(tree.entries[1].path.ends_with("child.tuxd.json"));
	tree.filters.hidden = true;
	tree.filters.native_only = false;
	tree.refresh().unwrap();
	assert!(
		tree.entries
			.iter()
			.any(|entry| entry.path.ends_with(".hidden.tuxd.json"))
	);
	assert!(
		tree.entries
			.iter()
			.any(|entry| entry.path.ends_with("notes.txt"))
	);
	assert!(
		!tree
			.entries
			.iter()
			.any(|entry| entry.path.ends_with("cycle"))
	);
	assert!(tree.entries[0].expanded);
	tree.toggle(&root.join("nested")).unwrap();
	assert!(!tree.entries.iter().any(|entry| entry.depth > 0));
}

#[test]
fn failed_filter_refresh_preserves_entries_and_filters() {
	let folder = tempfile::tempdir().unwrap();
	let root = folder.path().join("root");
	fs::create_dir(&root).unwrap();
	fs::write(root.join("keep.tuxd.json"), "").unwrap();
	let mut tree = Tree::open(&root).unwrap();
	fs::rename(&root, folder.path().join("moved")).unwrap();

	assert!(
		tree.refresh_with(tree::Filters {
			hidden: true,
			native_only: false
		})
		.is_err()
	);
	assert!(!tree.filters.hidden);
	assert!(tree.filters.native_only);
	assert_eq!(tree.entries.len(), 1);
	assert!(tree.entries[0].path.ends_with("keep.tuxd.json"));
}

#[test]
fn opening_a_directory_is_transactional_and_keeps_buffers_and_history() {
	let first = tempfile::tempdir().unwrap();
	let second = tempfile::tempdir().unwrap();
	let mut workspace = Workspace::default();
	workspace.add_buffer(
		Document::from_text("keep", Default::default()).unwrap(),
		None,
		true,
	);
	let id = workspace.active.session.id;
	workspace.open_directory(first.path()).unwrap();
	workspace.open_directory(second.path()).unwrap();
	assert_eq!(workspace.active.session.id, id);
	assert_eq!(workspace.buffers().count(), 2);
	assert!(workspace.active.session.dirty());
	assert!(
		workspace
			.open_directory(&second.path().join("missing"))
			.is_err()
	);
	assert_eq!(workspace.directory(), Some(second.path()));
}

#[test]
fn sidebar_docks_or_overlays_without_leaking_mouse_hits_and_tabs_return_when_hidden() {
	let folder = tempfile::tempdir().unwrap();
	let mut workspace = Workspace::default();
	workspace.open_directory(folder.path()).unwrap();
	for width in [20, 40, 64, 65, 80, 100, 140] {
		workspace.focus = Pane::Sidebar;
		workspace.sidebar.visible = true;
		draw(&mut workspace, width, 24);
		assert!(workspace.tab_hits.is_empty());
		assert_eq!(workspace.sidebar.area.width, 26.min(width - 4));
		assert_eq!(workspace.sidebar_overlay, width <= 64);
		if width > 64 {
			assert!(workspace.active.editor.surface.text.x >= workspace.sidebar.area.right());
		}
		workspace.pane_action(A::ToggleSidebar);
		draw(&mut workspace, width, 24);
		assert_eq!(workspace.sidebar.area, Rect::default());
		assert!(!workspace.tab_hits.is_empty());
	}
}

#[test]
fn keyboard_tree_actions_and_open_file_close_target_the_selected_buffer() {
	let folder = tempfile::tempdir().unwrap();
	fs::create_dir(folder.path().join("nested")).unwrap();
	fs::write(folder.path().join("nested/child.tuxd.json"), "").unwrap();
	let mut workspace = Workspace::default();
	let first = workspace.active.session.id;
	workspace.add_buffer(Document::default(), None, false);
	workspace.open_directory(folder.path()).unwrap();
	draw(&mut workspace, 100, 24);
	workspace.sidebar_action(A::ExplorerDown);
	assert!(
		matches!(workspace.sidebar_action(A::CloseBuffer), crate::workspace::Effect::CloseBuffer(id) if id == first)
	);
	workspace.sidebar_action(A::ExplorerLast);
	workspace.sidebar_action(A::ExplorerEnter);
	workspace.sidebar_action(A::ExplorerDown);
	assert!(
		matches!(workspace.sidebar_action(A::Activate), crate::workspace::Effect::OpenFile(path) if path.ends_with("child.tuxd.json"))
	);
	workspace.sidebar_action(A::ExplorerParent);
	workspace.sidebar_action(A::ExplorerParent);
	workspace.refresh_sidebar_rows();
	assert!(
		!workspace
			.sidebar
			.rows
			.iter()
			.any(|row| matches!(row, Row::Entry(entry) if entry.depth > 0))
	);
}

#[test]
fn scrollbar_reaches_the_end_and_refresh_discovers_new_files() {
	let folder = tempfile::tempdir().unwrap();
	for index in 0..40 {
		fs::write(folder.path().join(format!("{index:02}.tuxd.json")), "").unwrap();
	}
	let mut workspace = Workspace::default();
	workspace.open_directory(folder.path()).unwrap();
	draw(&mut workspace, 100, 12);
	let bar = workspace.sidebar.bars[0];
	workspace.sidebar.top = bar.offset_at(bar.length(), 0);
	draw(&mut workspace, 100, 12);
	assert_eq!(workspace.sidebar.top, workspace.sidebar.rows.len() - 10);
	fs::write(folder.path().join("new.tuxd.json"), "").unwrap();
	workspace.sidebar_action(A::RefreshDirectory);
	workspace.refresh_sidebar_rows();
	assert!(
		workspace
			.sidebar
			.rows
			.iter()
			.any(|row| matches!(row, Row::Entry(entry) if entry.path.ends_with("new.tuxd.json")))
	);
}

#[test]
fn sidebar_keeps_relative_labels_dirty_markers_and_a_visible_editor_on_dismissal() {
	let folder = tempfile::tempdir().unwrap();
	fs::create_dir(folder.path().join("nested")).unwrap();
	let path = folder.path().join("nested/document.tuxd.json");
	fs::write(&path, "").unwrap();
	let mut workspace = Workspace::default();
	workspace.open_document(Document::default(), Some(path), false);
	workspace.open_directory(folder.path()).unwrap();
	workspace
		.active
		.session
		.editing
		.document
		.insert(Default::default(), "dirty")
		.unwrap();
	draw(&mut workspace, 100, 24);
	assert!(
		matches!(&workspace.sidebar.rows[1], Row::Buffer { label, dirty: true, .. } if label == "nested/document.tuxd.json")
	);
	workspace.editor_open = false;
	workspace.pane_action(A::ToggleSidebar);
	assert_eq!(workspace.focus, Pane::Editor);
	assert!(workspace.editor_open);
}
