use super::*;
use crate::shortcuts::registry::category_actions;
use crate::shortcuts::{ActionId, Dispatch, InputScope};
use crate::shortcuts::{default_slots, sequence_label};
use crate::workspace::Pane;
use crossterm::event::KeyEvent;
use crossterm::event::{Event, KeyEventKind, KeyModifiers as M, MouseButton, MouseEventKind};
use ratatui::{Terminal, backend::TestBackend};
use std::time::Instant;
fn event(text: &str) -> Event {
	let key = crate::shortcuts::key::defaults(text).remove(0);
	Event::Key(KeyEvent::new(key.code, key.modifiers))
}

fn select(settings: &mut Settings, action: ActionId, slot: usize) {
	settings.select_action(action);
	settings.shortcuts.slot = slot;
	settings.focus = Focus::Binding(slot);
}

#[test]
fn preset_chooser_previews_without_changing_draft_until_confirmed() {
	let mut settings = Settings::new(&Config::default(), Section::Appearance);
	settings.focus = Focus::Preset;
	settings.event(event("Enter"));
	settings.event(event("G"));
	assert_eq!(settings.appearance.draft.preset, "midnight");
	assert_eq!(
		settings.preview(),
		crate::theme::Theme::preset("catppuccin-mocha").unwrap()
	);
	settings.event(event("Esc"));
	assert_eq!(settings.preview(), crate::theme::Theme::default());
	settings.event(event("Enter"));
	settings.event(event("G"));
	settings.event(event("Enter"));
	assert_eq!(settings.appearance.draft.preset, "catppuccin-mocha");
	assert!(settings.dirty());
	settings.event(event("Enter"));
	for (width, height) in [(140, 40), (80, 24), (40, 12), (20, 5), (1, 1)] {
		let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
		terminal.draw(|frame| settings.draw(frame)).unwrap();
		for (rect, _) in &settings.hits {
			assert!(rect.right() <= width && rect.bottom() <= height);
		}
	}
}

#[test]
fn presets_preserve_overrides_and_theme_reset_leaves_shortcuts_alone() {
	let mut settings = Settings::new(&Config::default(), Section::Appearance);
	settings
		.appearance
		.draft
		.colors
		.insert("accent".into(), "#123456".into());
	let shortcuts = settings.shortcuts.draft.clone();
	settings.focus = Focus::Preset;
	settings.event(event("Right"));
	assert_eq!(settings.appearance.draft.preset, "light");
	assert_eq!(
		settings.preview().accent,
		crate::libraries::themes::Color::Rgb(18, 52, 86)
	);
	settings.focus = Focus::ResetTheme;
	settings.event(event("Enter"));
	assert_eq!(settings.appearance.draft.preset, "light");
	assert!(settings.appearance.draft.colors.is_empty());
	assert_eq!(settings.shortcuts.draft, shortcuts);
	assert_eq!(
		settings.preview(),
		crate::theme::Theme::preset("light").unwrap()
	);
	assert!(settings.dirty());
}

#[test]
fn appearance_sections_fields_reset_and_resizing() {
	let mut settings = Settings::new(&Config::default(), Section::Appearance);
	settings.focus = Focus::List;
	assert!(settings.appearance.collapsed.is_empty());
	settings
		.appearance
		.draft
		.colors
		.insert("accent".into(), "#123456".into());
	settings.event(event("Tab"));
	assert_eq!(settings.focus, Focus::Reset);
	settings.event(event("Enter"));
	assert!(!settings.appearance.draft.colors.contains_key("accent"));
	settings.focus = Focus::List;
	settings.event(event("Home"));
	settings.event(event("Enter"));
	assert!(
		settings
			.appearance
			.collapsed
			.contains(&crate::theme::Group::Interface)
	);
	assert!(!matches!(settings.mode, Mode::Color(_)));
	settings.event(event("Enter"));
	assert!(
		!settings
			.appearance
			.collapsed
			.contains(&crate::theme::Group::Interface)
	);
	for (width, height) in [(140, 40), (80, 24), (40, 16), (24, 8), (1, 1)] {
		let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
		settings.event(event("G"));
		terminal.draw(|frame| settings.draw(frame)).unwrap();
		for (rect, _) in &settings.hits {
			assert!(rect.right() <= width && rect.bottom() <= height, "{rect:?}");
		}
	}
}

#[test]
fn theme_preview_groups_and_failed_save_preserve_active_configuration() {
	let mut settings = Settings::new(&Config::default(), Section::Appearance);
	settings.focus = Focus::List;
	settings.event(event("Enter"));
	settings.event(Event::Paste("#123456".into()));
	assert_eq!(
		settings.preview().accent,
		crate::libraries::themes::Color::Rgb(18, 52, 86)
	);
	assert_eq!(settings.appearance.draft, crate::theme::Config::default());
	settings.event(event("Enter"));
	assert!(settings.dirty());
	assert!(matches!(settings.close(), Outcome::Keep));
	settings.mode = Mode::Browse;
	settings.switch_category(Section::ShortcutsGroup);
	settings.toggle_group();
	assert!(!settings.categories().contains(&Section::Editor));
	settings.switch_category(Section::Editor);
	assert!(settings.categories().contains(&Section::Editor));
	let dir = tempfile::tempdir().unwrap();
	let shortcuts_path = dir.path().join("shortcuts.json");
	std::fs::create_dir(&shortcuts_path).unwrap();
	let mut runtime = crate::shortcuts::Runtime::default();
	runtime.path = Ok(shortcuts_path);
	let original = runtime.config.clone();
	assert!(
		crate::configuration::commit::save(
			settings.drafts(),
			|config| runtime.save(config.clone()),
			&Ok(dir.path().join("theme.json"))
		)
		.is_err()
	);
	assert_eq!(runtime.config, original);
	assert!(!dir.path().join("theme.json").exists());
	let theme_path = dir.path().join("theme.json");
	crate::theme::Config::default().write(&theme_path).unwrap();
	let original_bytes = std::fs::read(&theme_path).unwrap();
	assert!(
		crate::configuration::commit::save(
			settings.drafts(),
			|config| runtime.save(config.clone()),
			&Ok(dir.path().join("theme.json"))
		)
		.is_err()
	);
	assert_eq!(std::fs::read(&theme_path).unwrap(), original_bytes);
}

#[test]
fn general_navigation_and_unaffected_leader_need_no_confirmation() {
	let mut settings = Settings::new(&Config::default(), Section::Timeout);
	settings.focus = Focus::Categories;
	settings.event(event("Right"));
	assert_eq!(settings.focus, Focus::Leader);
	settings.event(event("Down"));
	assert_eq!(settings.focus, Focus::Timeout);
	settings.event(event("Up"));
	assert_eq!(settings.focus, Focus::Leader);
	settings.event(event("Enter"));
	settings.event(event("F12"));
	assert!(!matches!(settings.mode, Mode::LeaderChange(_)));
	assert_eq!(
		settings.shortcuts.draft.leader,
		Some(crate::shortcuts::key::defaults("F12")[0].clone())
	);
	settings.event(event("Delete"));
	assert!(!matches!(settings.mode, Mode::LeaderChange(_)));
	assert!(settings.shortcuts.draft.leader.is_none());
	settings.event(event("Enter"));
	settings.event(event("Ctrl+y"));
	assert!(matches!(settings.mode, Mode::LeaderChange(_)));
	settings.event(event("Enter")); // Cancel is the default.
	assert!(settings.shortcuts.draft.leader.is_none());
}

#[test]
fn leader_capture_extends_count_and_changes_are_staged() {
	let mut settings = Settings::new(&Config::default(), Section::Timeout);
	settings.focus = Focus::Leader;
	settings.event(event("Enter"));
	settings.event(event("Ctrl+y"));
	assert!(settings.shortcuts.draft.leader.is_none());
	settings.event(event("y"));
	assert!(settings.shortcuts.draft.leader.is_some());
	assert!(settings.shortcuts.original.leader.is_none());
	select(&mut settings, ActionId::FocusTools, 0);
	settings.event(event("2"));
	settings.event(event("Ctrl+y"));
	settings.event(event("g"));
	assert!(matches!(settings.mode, Mode::Capture(_)));
	settings.event(event("g"));
	assert!(!matches!(settings.mode, Mode::Capture(_)));
	assert_eq!(
		sequence_label(
			settings.shortcuts.draft.bindings[&ActionId::FocusTools][0]
				.as_ref()
				.unwrap()
		),
		"<leader> g g"
	);
	settings.focus = Focus::Leader;
	settings.event(event("Delete"));
	settings.event(event("p"));
	assert!(settings.shortcuts.draft.leader.is_none());
	assert_eq!(
		sequence_label(
			settings.shortcuts.draft.bindings[&ActionId::FocusTools][0]
				.as_ref()
				.unwrap()
		),
		"Ctrl+Y g g"
	);
	settings.shortcuts.draft.build().unwrap();
}

#[test]
fn capture_is_counted_ignores_repeat_release_and_can_cancel() {
	let mut settings = Settings::new(&Config::default(), Section::Editor);
	select(&mut settings, ActionId::FocusTools, 2);
	settings.event(event("2"));
	let Event::Key(mut repeat) = event("Ctrl+w") else {
		unreachable!()
	};
	repeat.kind = KeyEventKind::Repeat;
	settings.event(Event::Key(repeat));
	repeat.kind = KeyEventKind::Release;
	settings.event(Event::Key(repeat));
	assert!(settings.mode.capture().unwrap().keys.is_empty());
	settings.event(event("Ctrl+w"));
	assert_eq!(settings.mode.capture().unwrap().keys.len(), 1);
	settings.event(event("Right"));
	assert!(!matches!(settings.mode, Mode::Capture(_)));
	assert_eq!(
		settings.shortcuts.draft.bindings[&ActionId::FocusTools][2],
		Some(crate::shortcuts::key::defaults("Ctrl+w Right"))
	);
	assert!(settings.error.is_none());
	settings.event(event("Enter"));
	settings.event(event("Esc"));
	assert!(!matches!(settings.mode, Mode::Capture(_)));
	assert!(settings.shortcuts.draft.bindings[&ActionId::FocusTools][2].is_some());
}

#[test]
fn conflicts_keep_the_draft_editable_and_reset_unset_and_discard_are_fixed() {
	let mut settings = Settings::new(&Config::default(), Section::Editor);
	select(&mut settings, ActionId::FocusTools, 0);
	settings.event(event("Enter"));
	settings.event(event("F1"));
	assert!(settings.error.as_ref().unwrap().contains("reserved"));
	assert!(matches!(settings.event(event("Ctrl+s")), Outcome::Keep));
	settings.focus = Focus::Binding(0);
	settings.event(event("Delete"));
	assert!(settings.error.is_none());
	select(&mut settings, ActionId::Bold, 0);
	settings.event(event("Delete"));
	assert!(settings.shortcuts.draft.bindings[&ActionId::Bold][0].is_none());
	settings.focus = Focus::Reset;
	settings.event(event("Enter"));
	assert_eq!(
		settings.shortcuts.draft.bindings[&ActionId::Bold],
		default_slots(ActionId::Bold)
	);
	settings.focus = Focus::Binding(0);
	settings.event(event("Delete"));
	settings.event(event("Esc"));
	assert!(matches!(settings.mode, Mode::Discard(false)));
	settings.event(event("Enter"));
	assert!(!matches!(settings.mode, Mode::Discard(_)));
	settings.event(event("Esc"));
	settings.event(event("Left"));
	assert!(matches!(settings.event(event("Enter")), Outcome::Close));
}

#[test]
fn save_activates_and_persists_only_after_explicit_save_and_restart() {
	let folder = tempfile::tempdir().unwrap();
	let path = folder.path().join("shortcuts.json");
	let mut app = crate::app::App::default();
	app.shortcuts.path = Ok(path.clone());
	app.handle(event("F1"));
	let settings = app.settings.as_mut().unwrap();
	select(settings, ActionId::ToggleCode, 0);
	app.handle(event("Enter"));
	app.handle(event("F8"));
	assert_eq!(app.shortcuts.config.hint(ActionId::ToggleCode), "F2");
	assert!(!path.exists());
	app.handle(event("Ctrl+s"));
	assert!(app.settings.is_none());
	assert_eq!(app.shortcuts.config.hint(ActionId::ToggleCode), "F8");
	app.handle(event("F2"));
	assert!(app.workspace.code_open);
	app.handle(event("F8"));
	assert!(!app.workspace.code_open);
	let mut restarted = crate::app::App::default();
	restarted.shortcuts.path = Ok(path);
	restarted.shortcuts.load().unwrap();
	restarted.handle(event("F8"));
	assert!(!restarted.workspace.code_open);
}

#[test]
fn save_failure_retains_draft_and_capture_reports_actual_terminal_key() {
	let mut app = crate::app::App::default();
	app.shortcuts.path = Err("test unavailable config directory".into());
	app.handle(event("F1"));
	let settings = app.settings.as_mut().unwrap();
	select(settings, ActionId::FocusTools, 0);
	app.handle(event("Enter"));
	app.handle(event("Ctrl+7"));
	assert!(app.settings.as_ref().unwrap().error.is_some());
	app.handle(event("Enter"));
	app.handle(event("F8"));
	app.handle(event("Ctrl+s"));
	let settings = app.settings.as_ref().unwrap();
	assert!(
		settings
			.error
			.as_ref()
			.unwrap()
			.contains("test unavailable")
	);
	assert_eq!(settings.shortcuts.draft.hint(ActionId::FocusTools), "F8");
	assert!(app.shortcuts.config.hint(ActionId::FocusTools).is_empty());
}

#[test]
fn timeout_edit_reset_all_and_fixed_navigation_are_independent_of_bindings() {
	let mut config = Config::default();
	for slots in config.bindings.values_mut() {
		*slots = [None, None, None];
	}
	let mut settings = Settings::new(&config, Section::Editor);
	settings.event(event("Ctrl+t"));
	for ch in ["2", "5", "0"] {
		settings.event(event(ch));
	}
	settings.event(event("Enter"));
	assert_eq!(settings.shortcuts.draft.timeout_ms, 250);
	settings.focus = Focus::Categories;
	settings.event(event("g"));
	settings.event(event("g"));
	for _ in 0..3 {
		settings.event(event("j"));
	}
	assert_eq!(settings.category, Section::Timeout);
	settings.event(event("j"));
	assert_eq!(settings.category, Section::Workspace);
	settings.event(event("G"));
	assert_eq!(settings.category, Section::Explorer);
	settings.event(event("Ctrl+r"));
	assert_eq!(settings.shortcuts.draft, Config::default());
	assert!(matches!(settings.event(event("Ctrl+s")), Outcome::Save));
}

#[test]
fn resize_scroll_mouse_and_context_jump_keep_controls_reachable() {
	let mut settings = Settings::new(&Config::default(), Section::Explorer);
	for (width, height) in [
		(140, 40),
		(80, 24),
		(64, 20),
		(40, 12),
		(20, 5),
		(1, 1),
		(140, 40),
	] {
		let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
		terminal.draw(|frame| settings.draw(frame)).unwrap();
		if width >= 40 && height >= 20 {
			assert!(settings.hits.iter().any(|(_, hit)| *hit == Hit::Footer(2)));
			assert!(!settings.hits.is_empty());
			let rect = settings
				.hits
				.iter()
				.find(|(_, c)| matches!(c, Hit::Binding(_)))
				.unwrap()
				.0;
			settings.event(Event::Mouse(crossterm::event::MouseEvent {
				kind: MouseEventKind::Down(MouseButton::Left),
				column: rect.x,
				row: rect.y,
				modifiers: M::NONE,
			}));
			assert!(matches!(settings.mode, Mode::Capture(_)));
			settings.event(event("Esc"));
		}
	}
	let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
	settings.event(event("G"));
	terminal.draw(|frame| settings.draw(frame)).unwrap();
	assert!(settings.hits.iter().any(|(_, c)| *c == Hit::Footer(2)));
	settings.focus = Focus::Footer(2);
	assert!(matches!(settings.event(event("Enter")), Outcome::Close));
}

#[test]
fn draft_sequence_uses_library_after_save_and_does_not_change_dialog_scope() {
	let folder = tempfile::tempdir().unwrap();
	let mut app = crate::app::App::default();
	app.shortcuts.path = Ok(folder.path().join("shortcuts.json"));
	app.handle(event("F1"));
	select(app.settings.as_mut().unwrap(), ActionId::FocusCode, 0);
	app.handle(event("2"));
	app.handle(event("Ctrl+w"));
	app.handle(event("Right"));
	app.handle(event("Ctrl+s"));
	app.handle(event("Ctrl+w"));
	assert!(app.shortcuts.pending_label().is_some());
	app.handle(event("Right"));
	assert_eq!(app.workspace.focus, Pane::Code);
	assert!(matches!(
		app.shortcuts.dispatch(
			crate::shortcuts::key::defaults("Ctrl+w").remove(0),
			InputScope::Prompt,
			Instant::now()
		),
		Dispatch::Unbound
	));
}

#[test]
fn categories_subsections_and_context_jump_keep_movement_and_selection_separate() {
	assert_eq!(
		Section::ALL.map(Section::label),
		[
			"Theme",
			"Appearance",
			"Shortcuts",
			"General",
			"Workspace",
			"Editor",
			"Code",
			"Tools",
			"Dialogs",
			"File Explorer"
		]
	);
	for category in Section::ALL {
		let settings = Settings::new(&Config::default(), category);
		assert!(settings.shortcuts.collapsed.is_empty());
		for action in category
			.shortcut_category()
			.map(category_actions)
			.unwrap_or_default()
		{
			assert!(settings.shortcuts.rows.contains(&Row::Action(action.id)));
		}
	}
	let mut settings = Settings::for_scope(&Config::default(), InputScope::Grid);
	assert_eq!(settings.category, Section::Workspace);
	assert_eq!(settings.shortcuts.selected, Some(ActionId::FieldLeft));
	assert!(
		!settings
			.shortcuts
			.collapsed
			.contains(&(Section::Workspace, Subsection::FieldNavigation))
	);
	settings.select_action(ActionId::SelectLeft);
	assert_eq!(settings.category, Section::Editor);
	assert!(
		settings
			.shortcuts
			.rows
			.contains(&Row::Action(ActionId::SelectLeft))
	);
	assert!(
		settings
			.shortcuts
			.rows
			.contains(&Row::Action(ActionId::MoveLeft))
	);
	settings.toggle_section(Subsection::Selection);
	settings.event(event("2"));
	assert!(
		!matches!(settings.mode, Mode::Capture(_)),
		"A collapsed heading must not edit a hidden action"
	);
	settings.select_action(ActionId::ToggleCode);
	assert_eq!(settings.category, Section::Code);
}

#[test]
fn category_selector_and_footer_remain_usable_at_small_sizes() {
	let mut settings = Settings::new(&Config::default(), Section::Editor);
	for (width, height) in [
		(120, 32),
		(80, 24),
		(64, 24),
		(40, 16),
		(40, 12),
		(24, 8),
		(20, 5),
		(1, 1),
	] {
		let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
		terminal.draw(|frame| settings.draw(frame)).unwrap();
		for (rect, _) in &settings.hits {
			assert!(
				rect.right() <= width && rect.bottom() <= height,
				"{rect:?} outside {width}x{height}"
			);
		}
		if width >= 24 && height >= 8 {
			assert!(settings.hits.iter().any(|(_, h)| *h == Hit::Footer(1)));
			assert!(settings.hits.iter().any(|(_, h)| *h == Hit::Footer(2)));
			settings.event(event("Ctrl+l"));
			settings.event(event("g"));
			settings.event(event("g"));
			for _ in 0..3 {
				settings.event(event("j"));
			}
			terminal.draw(|frame| settings.draw(frame)).unwrap();
			assert_eq!(settings.category, Section::Timeout);
			settings.event(event("Enter"));
			assert_eq!(settings.focus, Focus::Leader);
			settings.event(event("Enter"));
			settings.event(event("Esc"));
			assert!(!matches!(settings.mode, Mode::LeaderCapture));
			settings.switch_category(Section::Editor);
			settings.focus = Focus::List;
		}
	}
}

#[test]
fn field_editing_owns_input_until_commit_or_cancel() {
	let mut settings = Settings::new(&Config::default(), Section::Appearance);
	settings.focus = Focus::List;
	settings.event(event("Enter"));
	settings.event(Event::Paste("#bad".into()));
	settings.event(event("Enter"));
	assert!(matches!(settings.mode, Mode::Color(_)));
	assert!(settings.error.is_some());
	assert!(matches!(settings.event(event("Ctrl+s")), Outcome::Keep));
	assert!(matches!(settings.mode, Mode::Color(_)));
	assert_eq!(settings.preview(), crate::theme::Theme::default());
	settings.event(event("Esc"));
	assert!(matches!(settings.mode, Mode::Browse));
	assert!(!settings.dirty());

	settings.event(event("Ctrl+t"));
	assert!(matches!(settings.mode, Mode::Timeout(_)));
	settings.event(event("Ctrl+l"));
	assert!(matches!(settings.mode, Mode::Timeout(_)));
	settings.event(event("0"));
	settings.event(event("Enter"));
	assert!(matches!(settings.mode, Mode::Timeout(_)));
	assert_eq!(settings.shortcuts.draft.timeout_ms, 1000);
	settings.event(event("Esc"));
	assert!(matches!(settings.mode, Mode::Browse));
}

#[test]
fn appearance_catalog_controls_display_order_without_numeric_role_indices() {
	use crate::theme::{Group, Role};
	let settings = Settings::new(&Config::default(), Section::Appearance);
	let rows = settings.theme_rows();
	assert_eq!(
		&rows[..6],
		&[
			ThemeRow::Section(Group::Interface),
			ThemeRow::Color(Role::Accent),
			ThemeRow::Color(Role::Text),
			ThemeRow::Color(Role::Muted),
			ThemeRow::Color(Role::Surface),
			ThemeRow::Color(Role::Border),
		]
	);
	assert_eq!(
		rows.iter()
			.filter(|row| matches!(row, ThemeRow::Color(_)))
			.count(),
		Role::ALL.len()
	);
}

#[test]
fn discard_confirmation_masks_underlying_page_actions() {
	let mut settings = Settings::new(&Config::default(), Section::Appearance);
	settings
		.appearance
		.draft
		.colors
		.insert("accent".into(), "#123456".into());
	settings.focus = Focus::List;
	settings.event(event("Esc"));
	assert!(matches!(settings.mode, Mode::Discard(false)));
	let draft = settings.appearance.draft.clone();
	settings.event(event("Delete"));
	settings.event(event("Ctrl+s"));
	assert!(matches!(settings.mode, Mode::Discard(false)));
	assert_eq!(settings.appearance.draft, draft);
	settings.event(event("Esc"));
	assert!(matches!(settings.mode, Mode::Browse));
}
