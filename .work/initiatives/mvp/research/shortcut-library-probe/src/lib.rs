#[cfg(test)]
mod tests {
	use crossterm::event::{KeyCode, KeyEvent, KeyModifiers as M};
	use ratatui_which_key::{Key, Keymap, NodeResult, WhichKeyState};
	fn key(code: KeyCode, mods: M) -> KeyEvent {
		KeyEvent::new(code, mods)
	}
	fn map() -> Keymap<KeyEvent, u8, &'static str, String> {
		let mut map = Keymap::new();
		map.bind("<c-w><right>", "tools", "focus".into(), 0);
		map.bind("<c-w><left>", "editor", "focus".into(), 0);
		map.bind("gg", "top", "navigation".into(), 1);
		map.bind_global("<F1>", "settings", "global".into());
		map
	}
	#[test]
	fn which_key_sequences_scopes_and_global() {
		let map = map();
		let prefix = key(KeyCode::Char('w'), M::CONTROL);
		assert!(matches!(
			map.navigate(&[prefix], &0),
			Some(NodeResult::Branch { .. })
		));
		assert!(matches!(
			map.navigate(&[prefix, key(KeyCode::Right, M::NONE)], &0),
			Some(NodeResult::Leaf { action: "tools" })
		));
		assert!(map.navigate(&[prefix], &1).is_none());
		assert!(matches!(
			map.navigate(&[key(KeyCode::F(1), M::NONE)], &1),
			Some(NodeResult::Leaf { action: "settings" })
		));
	}
	#[test]
	fn which_key_scope_change_requires_explicit_cancel() {
		let mut state = WhichKeyState::new(map(), 0);
		state.handle_key(key(KeyCode::Char('w'), M::CONTROL));
		assert!(state.is_pending());
		state.set_scope(1);
		assert!(state.is_pending());
		state.dismiss();
		assert!(!state.is_pending());
	}
	#[test]
	fn which_key_stock_formatter_loses_shift_on_ctrl_char() {
		assert_eq!(
			Key::display(&key(KeyCode::Char('z'), M::CONTROL | M::SHIFT)),
			Key::display(&key(KeyCode::Char('z'), M::CONTROL))
		);
	}
	#[derive(Clone, PartialEq, Eq)]
	struct Exact(KeyEvent);
	impl Key for Exact {
		fn display(&self) -> String {
			format!("{:?}", self.0)
		}
		fn is_backspace(&self) -> bool {
			self.0.code == KeyCode::Backspace
		}
		fn space() -> Self {
			Self(key(KeyCode::Char(' '), M::NONE))
		}
		fn from_char(c: char) -> Option<Self> {
			Some(Self(key(KeyCode::Char(c), M::NONE)))
		}
		fn from_special_name(name: &str) -> Option<Self> {
			if name == "cs-z" {
				Some(Self(key(KeyCode::Char('z'), M::CONTROL | M::SHIFT)))
			} else {
				KeyEvent::from_special_name(name).map(Self)
			}
		}
	}
	#[test]
	fn which_key_custom_key_retains_combined_modifiers_and_unicode() {
		let mut map: Keymap<Exact, u8, &str, String> = Keymap::new();
		map.bind("<cs-z>", "redo", "editor".into(), 0)
			.bind("é", "unicode", "explorer".into(), 1);
		assert!(matches!(
			map.navigate(&[Exact(key(KeyCode::Char('z'), M::CONTROL | M::SHIFT))], &0),
			Some(NodeResult::Leaf { action: "redo" })
		));
		assert!(
			map.navigate(&[Exact(key(KeyCode::Char('z'), M::CONTROL))], &0)
				.is_none()
		);
		assert!(matches!(
			map.navigate(&[Exact(key(KeyCode::Char('é'), M::NONE))], &1),
			Some(NodeResult::Leaf { action: "unicode" })
		));
	}
	#[test]
	fn keybinds_sequences_and_pending_state() {
		let mut map = keybinds::Keybinds::default();
		map.bind("Ctrl+w Right", "tools").unwrap();
		map.bind("Ctrl+w Left", "editor").unwrap();
		assert_eq!(map.dispatch(key(KeyCode::Char('w'), M::CONTROL)), None);
		assert!(map.is_ongoing());
		assert_eq!(map.dispatch(key(KeyCode::Right, M::NONE)), Some(&"tools"));
		assert!(!map.is_ongoing());
	}
	#[test]
	fn keybinds_timeout_reset_and_prefix_validation_gap() {
		let mut map = keybinds::Keybinds::default();
		map.bind("g g", "top").unwrap();
		map.set_timeout(std::time::Duration::from_millis(2));
		map.dispatch('g');
		std::thread::sleep(std::time::Duration::from_millis(10));
		assert_eq!(map.dispatch('g'), None); // starts a new sequence after expiration
		map.reset();
		assert!(!map.is_ongoing());
		assert!(map.bind("g", "single").is_ok()); // application must reject prefix conflicts
	}
	#[test]
	fn keybinds_backtab_unicode_and_json_roundtrip() {
		let backtab = keybinds::KeyInput::from(key(KeyCode::BackTab, M::NONE));
		assert_eq!(
			backtab,
			keybinds::KeyInput::new(keybinds::Key::Tab, keybinds::Mods::SHIFT)
		);
		let seq: keybinds::KeySeq = "Ctrl+Z é".parse().unwrap();
		let encoded = serde_json::to_string(&seq).unwrap();
		assert_eq!(
			serde_json::from_str::<keybinds::KeySeq>(&encoded).unwrap(),
			seq
		);
	}
	#[test]
	fn keybinds_stock_conversion_loses_shift_on_lowercase_ctrl_char() {
		assert_eq!(
			keybinds::KeyInput::from(key(KeyCode::Char('z'), M::CONTROL | M::SHIFT)),
			keybinds::KeyInput::from(key(KeyCode::Char('z'), M::CONTROL))
		);
		assert!("Ctrl+Shift+z".parse::<keybinds::KeySeq>().is_err());
	}
}
