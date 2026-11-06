use super::*;
use crate::{
	app::App,
	editor::Movement,
	files::{Action as FileAction, Dialog},
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers as M};

fn key(s: &str) -> ShortcutKey {
	key::defaults(s).remove(0)
}
fn event(s: &str) -> Event {
	let k = key(s);
	Event::Key(KeyEvent::new(k.code, k.modifiers))
}
fn bind(config: &mut Config, action: ActionId, slot: usize, sequence: &str) {
	config.bindings.get_mut(&action).unwrap()[slot] = Some(key::defaults(sequence));
}
fn install(runtime: &mut Runtime, config: Config) {
	runtime.map = config.build().unwrap();
	runtime.config = config;
	runtime.cancel();
}
#[test]
fn every_default_slot_routes_to_its_action_in_all_applicable_scopes() {
	let config = Config::default();
	let map = config.build().unwrap();
	for spec in ACTIONS {
		for keys in config.bindings[&spec.id].iter().flatten() {
			for scope in spec.scopes {
				assert!(
					matches!(map.navigate(keys, scope), Some(NodeResult::Leaf { action }) if action == spec.id),
					"{} {:?}",
					spec.label,
					scope
				);
			}
		}
	}
}
#[test]
fn adapter_retains_modifiers_unicode_and_roundtrips_library_tokens() {
	use ratatui_which_key::Key;
	assert_eq!(
		ShortcutKey::new(KeyCode::BackTab, M::NONE),
		key("Shift+Tab")
	);
	assert_eq!(
		ShortcutKey::new(KeyCode::Char('Z'), M::CONTROL | M::SHIFT),
		key("Ctrl+Shift+z")
	);
	assert_ne!(key("Ctrl+z"), key("Ctrl+Shift+z"));
	assert_ne!(key("Ctrl+Left"), key("Ctrl+Alt+Left"));
	assert_eq!(ShortcutKey::new(KeyCode::Char('M'), M::SHIFT), key("M"));
	assert_eq!(ShortcutKey::new(KeyCode::Char('m'), M::SHIFT), key("M"));
	for code in [
		KeyCode::Char('界'),
		KeyCode::Char('<'),
		KeyCode::Char('>'),
		KeyCode::Char(' '),
		KeyCode::F(24),
		KeyCode::Backspace,
	] {
		for modifiers in [
			M::NONE,
			M::CONTROL | M::SHIFT,
			M::ALT | M::SUPER | M::HYPER | M::META,
		] {
			let original = ShortcutKey::new(code, modifiers);
			let sequence =
				ratatui_which_key::parse_key_sequence(&original.token(), &ShortcutKey::space());
			assert_eq!(sequence.as_slice(), std::slice::from_ref(&original));
			assert_eq!(
				serde_json::from_str::<ShortcutKey>(&serde_json::to_string(&original).unwrap())
					.unwrap(),
				original
			);
		}
	}
}
#[test]
fn validation_rejects_overlap_prefixes_printable_prefixes_and_all_reserved_aliases() {
	for (a, b) in [
		("Ctrl+w", "Ctrl+w Right"),
		("Ctrl+w Right", "Ctrl+w"),
		("Ctrl+w", "Ctrl+w"),
	] {
		let mut config = Config::default();
		bind(&mut config, ActionId::FocusEditor, 0, a);
		bind(&mut config, ActionId::FocusTools, 2, b);
		let message = config.build().unwrap_err();
		assert!(
			message.contains("Focus Editor slot 1")
				&& message.contains("Focus Tools slot 3")
				&& message.contains("Editor")
		);
	}
	for action in [
		ActionId::MoveLeft,
		ActionId::FieldLeft,
		ActionId::FieldDeleteBefore,
	] {
		for sequence in ["h", "H", "g g", "Space Right"] {
			let mut config = Config::default();
			bind(&mut config, action, 0, sequence);
			assert!(config.build().unwrap_err().contains("ordinary text"));
		}
	}
	for sequence in ["F1", "Ctrl+/", "Ctrl+,", "Ctrl+7", "Ctrl+_", "Ctrl+w F1"] {
		let mut config = Config::default();
		bind(&mut config, ActionId::FocusTools, 0, sequence);
		assert!(config.build().unwrap_err().contains("reserved"));
	}
	assert!(!key("Ctrl+?").opener());
}
#[test]
fn context_reuse_and_shared_prefixes_work_without_prefix_ambiguity() {
	let mut config = Config::default();
	bind(&mut config, ActionId::FocusEditor, 0, "Ctrl+w Left");
	bind(&mut config, ActionId::FocusTools, 0, "Ctrl+w Right");
	let mut runtime = Runtime::default();
	install(&mut runtime, config);
	let now = Instant::now();
	assert!(matches!(
		runtime.dispatch(key("F3"), InputScope::Editor, now),
		Dispatch::Action(ActionId::EditorWrap)
	));
	assert!(matches!(
		runtime.dispatch(key("F3"), InputScope::Code, now),
		Dispatch::Action(ActionId::CodeWrap)
	));
	assert!(matches!(
		runtime.dispatch(key("Tab"), InputScope::Editor, now),
		Dispatch::Action(ActionId::InsertTab)
	));
	assert!(matches!(
		runtime.dispatch(key("Tab"), InputScope::Prompt, now),
		Dispatch::Action(ActionId::NextField)
	));
	assert!(matches!(
		runtime.dispatch(key("Ctrl+w"), InputScope::Editor, now),
		Dispatch::Pending
	));
	assert!(matches!(
		runtime.dispatch(key("Right"), InputScope::Editor, now),
		Dispatch::Action(ActionId::FocusTools)
	));
}
#[test]
fn timeout_mismatch_escape_and_scope_change_clear_pending_input() {
	let mut runtime = Runtime::default();
	let now = Instant::now();
	assert!(matches!(
		runtime.dispatch(key("g"), InputScope::Explorer, now),
		Dispatch::Pending
	));
	assert!(matches!(
		runtime.dispatch(key("x"), InputScope::Explorer, now),
		Dispatch::Consumed
	));
	assert!(runtime.pending_label().is_none());
	runtime.dispatch(key("g"), InputScope::Explorer, now);
	assert!(matches!(
		runtime.dispatch(key("Esc"), InputScope::Explorer, now),
		Dispatch::Consumed
	));
	runtime.dispatch(key("g"), InputScope::Explorer, now);
	assert!(!runtime.expire(now + Duration::from_millis(999)));
	assert!(runtime.expire(now + Duration::from_millis(1000)));
	assert!(runtime.pending_label().is_none());
	runtime.dispatch(key("g"), InputScope::Explorer, now);
	assert!(matches!(
		runtime.dispatch(key("g"), InputScope::Filename, now),
		Dispatch::Unbound
	));
	assert!(runtime.pending_label().is_none());
}
#[test]
fn config_is_atomic_strict_and_preserves_explicit_unset_slots() {
	let folder = tempfile::tempdir().unwrap();
	let path = folder.path().join("tuxd/shortcuts.json");
	let mut config = Config::default();
	config.bindings.get_mut(&ActionId::Undo).unwrap()[0] = None;
	bind(&mut config, ActionId::FocusTools, 2, "Ctrl+w Right");
	config.timeout_ms = 225;
	config.write(&path).unwrap();
	assert_eq!(Config::read(&path).unwrap(), config);
	assert!(
		std::fs::read_to_string(&path)
			.unwrap()
			.contains("\n\t\"version\"")
	);
	let before = std::fs::read(&path).unwrap();
	config.timeout_ms = 0;
	assert!(config.write(&path).is_err());
	assert_eq!(std::fs::read(&path).unwrap(), before);
	let value: serde_json::Value = serde_json::from_slice(&before).unwrap();
	for invalid in [
		serde_json::json!({"version": 99, "timeout_ms": 1000, "bindings": {}}),
		serde_json::json!({"version": 1, "timeout_ms": 1000, "bindings": {"not_an_action": [null,null,null]}}),
	] {
		std::fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
		assert!(Config::read(&path).is_err());
	}
	let mut partial = value;
	partial["bindings"].as_object_mut().unwrap().remove("bold");
	std::fs::write(&path, serde_json::to_vec(&partial).unwrap()).unwrap();
	assert_eq!(
		Config::read(&path).unwrap().bindings[&ActionId::Bold],
		default_slots(ActionId::Bold)
	);
}
#[test]
fn failed_load_or_save_leaves_active_config_and_original_file_untouched() {
	let folder = tempfile::tempdir().unwrap();
	let path = folder.path().join("shortcuts.json");
	let mut runtime = Runtime {
		path: Ok(path.clone()),
		..Runtime::default()
	};
	runtime.load().unwrap();
	assert!(!path.exists());
	std::fs::write(&path, "broken config").unwrap();
	assert!(runtime.load().is_err());
	assert_eq!(std::fs::read_to_string(&path).unwrap(), "broken config");
	assert_eq!(runtime.config, Config::default());
	runtime.path = Ok(path.join("impossible.json"));
	let mut draft = runtime.config.clone();
	draft.timeout_ms = 300;
	assert!(runtime.save(draft).is_err());
	assert_eq!(runtime.config, Config::default());
}
#[test]
fn app_dispatch_obeys_unset_bindings_selection_independence_and_extra_modifiers() {
	let mut app = App::default();
	let mut config = Config::default();
	config.bindings.get_mut(&ActionId::Bold).unwrap()[0] = None;
	bind(&mut config, ActionId::MoveLeft, 0, "Ctrl+g");
	install(&mut app.shortcuts, config);
	app.handle(Event::Paste("abc".into()));
	app.handle(event("Ctrl+b"));
	assert!(!app.editor.typing_style().bold);
	app.handle(event("Ctrl+g"));
	assert_eq!(app.editor.cursor.byte, 2);
	app.handle(event("Shift+Left"));
	assert!(app.editor.selection().is_some());
	app.editor.anchor = None;
	let before = app.editor.cursor;
	app.handle(event("Ctrl+Alt+Left"));
	assert_eq!(app.editor.cursor, before);
	app.handle(event("h"));
	assert!(app.editor.document.lines[0].contains('h'));
}
#[test]
fn settings_openers_preserve_selection_and_underlying_dialog_context() {
	let mut app = App::default();
	app.handle(Event::Paste("keep".into()));
	app.editor.navigate(Movement::Home, true);
	let selection = app.editor.selection();
	let document = app.editor.document.clone();
	for opener in ["F1", "Ctrl+/", "Ctrl+,", "Ctrl+7"] {
		app.handle(event(opener));
		assert!(app.settings.is_some());
		app.handle(Event::Paste("ignored".into()));
		app.handle(event("Esc"));
		assert_eq!(app.editor.selection(), selection);
		assert_eq!(app.editor.document, document);
	}
	app.file_action(FileAction::Open);
	let scope = app.input_scope();
	app.handle(event("F1"));
	app.handle(event("Esc"));
	assert_eq!(app.input_scope(), scope);
	assert!(matches!(app.files.dialog, Some(Dialog::Path { .. })));
	app.handle(event("Esc"));
	app.file_action(FileAction::Quit);
	app.handle(event("Ctrl+,"));
	app.handle(event("Esc"));
	assert!(matches!(app.files.dialog, Some(Dialog::Dirty)));
	app.handle(event("Esc"));
	assert_eq!(app.editor.document, document);
}
#[test]
fn remapped_prompt_responses_and_two_choice_fallback_use_actions() {
	let mut app = App::default();
	let mut config = Config::default();
	bind(&mut config, ActionId::Negative, 0, "x");
	install(&mut app.shortcuts, config);
	app.handle(Event::Paste("dirty".into()));
	app.file_action(FileAction::New);
	app.handle(event("d"));
	assert!(matches!(app.files.dialog, Some(Dialog::Dirty)));
	app.handle(event("x"));
	assert!(app.editor.document.is_empty());
	let folder = tempfile::tempdir().unwrap();
	let path = folder.path().join("existing.txt");
	std::fs::write(&path, "keep").unwrap();
	for binding in ["x", "a", "Esc"] {
		app.files.dialog = Some(Dialog::Overwrite {
			action: FileAction::ExportRaw,
			path: path.clone(),
		});
		app.handle(event(binding));
		assert!(app.files.dialog.is_none());
		assert_eq!(std::fs::read_to_string(&path).unwrap(), "keep");
	}
}
#[test]
fn pending_sequences_never_leak_into_text_and_cancel_on_nonkey_events() {
	let mut app = App::default();
	let mut config = Config::default();
	bind(&mut config, ActionId::FocusTools, 0, "Ctrl+w Right");
	install(&mut app.shortcuts, config);
	app.handle(event("Ctrl+w"));
	app.handle(event("x"));
	assert!(app.editor.document.is_empty());
	for interrupt in [
		Event::FocusLost,
		Event::Resize(80, 24),
		Event::Paste("a".into()),
	] {
		app.handle(event("Ctrl+w"));
		app.handle(interrupt);
		assert!(app.shortcuts.pending_label().is_none());
	}
	app.handle(event("Ctrl+w"));
	app.handle(event("F1"));
	assert!(app.shortcuts.pending_label().is_none());
}

#[test]
fn one_shared_field_binding_operates_tools_picker_review_and_prompts() {
	use crate::{import_review::Review, panel::Control};
	let mut app = App::default();
	let mut config = Config::default();
	bind(&mut config, ActionId::NextField, 0, "F8");
	install(&mut app.shortcuts, config);
	app.focus = 1;
	app.tool_index = app
		.panel
		.items()
		.iter()
		.position(|i| i.control == Control::Target(false))
		.unwrap();
	app.handle(event("F8"));
	assert!(matches!(
		app.panel.items()[app.tool_index].control,
		Control::Palette(_)
	));
	app.handle(event("Esc"));
	app.file_action(FileAction::Open);
	assert_eq!(app.input_scope(), InputScope::Explorer);
	app.handle(event("F8"));
	assert_eq!(app.input_scope(), InputScope::Filename);
	app.handle(event("Esc"));
	app.files.review = Review::default();
	let report = crate::ansi_import::parse("\x1b[2J", crate::ansi_import::Mode::Raw).unwrap();
	app.files.dialog = Some(Dialog::Review(report));
	assert_eq!(app.input_scope(), InputScope::ReviewButtons);
	app.handle(event("F8"));
	assert_eq!(app.input_scope(), InputScope::ReviewContent);
	app.handle(event("Esc"));
	app.focus = 0;
	app.handle(Event::Paste("keep".into()));
	app.file_action(FileAction::New);
	// Cancel is initially focused. Next wraps to Save, previous returns to Cancel.
	app.handle(event("F8"));
	app.handle(event("Shift+Tab"));
	app.handle(event("Enter"));
	assert!(app.files.dialog.is_none());
	assert_eq!(app.editor.document.lines[0], "keep");
	app.handle(event("Tab"));
	assert!(app.editor.document.lines[0].ends_with('\t'));
}

#[test]
fn shared_directions_ranges_and_selection_keep_control_specific_semantics() {
	use crate::{
		panel::Control,
		styling::{Action, Ink},
	};
	let mut app = App::default();
	let mut config = Config::default();
	bind(&mut config, ActionId::FieldRight, 0, "F8");
	bind(&mut config, ActionId::RangeIncrease10, 0, "F9");
	bind(&mut config, ActionId::SelectAll, 0, "F10");
	install(&mut app.shortcuts, config);
	app.focus = 1;
	let choose = |app: &mut App, control| {
		app.tool_index = app
			.panel
			.items()
			.iter()
			.position(|i| i.control == control)
			.unwrap();
	};
	choose(&mut app, Control::Target(false));
	app.handle(event("F8"));
	assert!(app.panel.background);
	choose(&mut app, Control::Palette(0));
	app.handle(event("F8"));
	assert_eq!(app.panel.palette, 1);
	choose(
		&mut app,
		Control::Style(Action::Background(Ink::Indexed(0))),
	);
	let before = app.editor.typing_style();
	app.handle(event("F8"));
	assert_eq!(app.editor.typing_style(), before);
	app.handle(event("Enter"));
	assert_eq!(app.editor.typing_style().background, Ink::Indexed(1));
	choose(&mut app, Control::Glyph('─'));
	app.handle(event("F8"));
	assert!(app.editor.document.is_empty());
	app.handle(event("Enter"));
	assert_eq!(app.editor.document.lines[0], "│");
	app.handle(event("Tab"));
	app.handle(event("Shift+Tab"));
	assert_eq!(
		app.panel.items()[app.tool_index].control,
		Control::Glyph('│')
	);
	app.panel.palette = 2;
	for field in 1..4 {
		choose(&mut app, Control::Field(field));
		app.panel.set_component(field, 100);
		app.handle(event("F9"));
		assert_eq!(app.panel.fields[field], "110");
	}
	choose(&mut app, Control::Field(0));
	app.handle(event("F10"));
	app.handle(Event::Paste("#aabbcc".into()));
	assert_eq!(app.panel.fields[0], "#aabbcc");
	app.handle(event("Home"));
	app.handle(event("F8"));
	app.handle(event("Backspace"));
	assert_eq!(app.panel.fields[0], "aabbcc");
}
