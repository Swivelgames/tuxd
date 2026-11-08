//! Public shortcut contracts exercised without terminal keys, application IDs, or views.
use ratatui_which_key::Key;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use terminal_ux_designer::libraries::shortcuts::{
	BindingKey, Catalog, Configuration, Context, Descriptor, Dispatch, Resolver,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Input(char);
impl Key for Input {
	fn display(&self) -> String {
		self.0.to_string()
	}
	fn is_backspace(&self) -> bool {
		false
	}
	fn space() -> Self {
		Self(' ')
	}
	fn from_char(ch: char) -> Option<Self> {
		Some(Self(ch))
	}
	fn from_special_name(_: &str) -> Option<Self> {
		None
	}
}
impl BindingKey for Input {
	fn leader() -> Self {
		Self('@')
	}
	fn is_leader(&self) -> bool {
		self.0 == '@'
	}
	fn normalized(self) -> Self {
		self
	}
	fn token(&self) -> String {
		self.0.to_string()
	}
	fn label(&self) -> String {
		self.0.to_string()
	}
	fn leader_allowed(&self) -> bool {
		!self.escape()
	}
	fn escape(&self) -> bool {
		self.0 == '!'
	}
	fn is_printable(&self) -> bool {
		self.0.is_ascii_lowercase()
	}
}
fn descriptor(id: u8, scope: u8, keys: &str) -> Descriptor<u8, u8, Input> {
	Descriptor {
		id,
		label: "Example",
		category: "Examples",
		scopes: vec![scope],
		defaults: [Some(keys.chars().map(Input).collect()), None, None],
	}
}
fn catalog() -> Catalog<u8, u8, Input> {
	Catalog::new(
		[
			descriptor(1, 0, "gg"),
			descriptor(2, 1, "gg"),
			descriptor(3, 2, "Z"),
		],
		[
			Context {
				id: 0,
				editable: false,
			},
			Context {
				id: 1,
				editable: false,
			},
			Context {
				id: 2,
				editable: true,
			},
		],
		vec![(Input('?'), "Help")],
	)
	.unwrap()
}
#[test]
fn supplied_catalog_validates_identity_and_contexts() {
	let contexts = || {
		[Context {
			id: 0,
			editable: false,
		}]
	};
	assert!(
		Catalog::new(
			[descriptor(1, 0, "a"), descriptor(1, 0, "b")],
			contexts(),
			vec![]
		)
		.unwrap_err()
		.contains("Duplicate action")
	);
	assert!(
		Catalog::new([descriptor(1, 1, "a")], contexts(), vec![])
			.unwrap_err()
			.contains("Invalid contexts")
	);
	assert!(
		Catalog::new(
			[descriptor(1, 0, "a")],
			[contexts()[0].clone(), contexts()[0].clone()],
			vec![]
		)
		.unwrap_err()
		.contains("Duplicate context")
	);
	assert_eq!(catalog().actions().len(), 3);
}
#[test]
fn reusable_scopes_prefix_conflicts_and_text_protection() {
	let catalog = catalog();
	let mut config = catalog.defaults();
	catalog.compile(&config).unwrap();
	config.bindings.get_mut(&1).unwrap()[1] = Some(vec![Input('g')]);
	assert!(catalog.compile(&config).unwrap_err().contains("conflicts"));
	config.bindings.get_mut(&1).unwrap()[1] = None;
	config.bindings.get_mut(&3).unwrap()[0] = Some(vec![Input('a')]);
	assert!(
		catalog
			.compile(&config)
			.unwrap_err()
			.contains("ordinary text")
	);
	config.bindings.get_mut(&3).unwrap()[0] = Some(vec![Input('?')]);
	assert!(
		catalog
			.compile(&config)
			.unwrap_err()
			.contains("reserved for Help")
	);
}
#[test]
fn sequences_cancel_on_context_timeout_escape_and_explicit_transition() {
	let catalog = catalog();
	let config = catalog.defaults();
	let map = catalog.compile(&config).unwrap();
	let mut resolver = Resolver::default();
	let now = Instant::now();
	assert_eq!(
		resolver.dispatch(&map, 1000, Input('g'), 0, now),
		Dispatch::Pending
	);
	// A new context starts its own sequence rather than completing the old one.
	assert_eq!(
		resolver.dispatch(&map, 1000, Input('g'), 1, now),
		Dispatch::Pending
	);
	assert_eq!(
		resolver.dispatch(&map, 1000, Input('g'), 1, now),
		Dispatch::Action(2)
	);
	assert_eq!(
		resolver.dispatch(&map, 1000, Input('g'), 0, now),
		Dispatch::Pending
	);
	assert_eq!(
		resolver.dispatch(&map, 1000, Input('x'), 0, now),
		Dispatch::Consumed
	);
	assert_eq!(
		resolver.dispatch(&map, 1000, Input('x'), 0, now),
		Dispatch::Unbound
	);
	resolver.dispatch(&map, 1000, Input('g'), 0, now);
	assert!(!resolver.expire(now + Duration::from_millis(999), 1000));
	assert!(resolver.expire(now + Duration::from_millis(1000), 1000));
	resolver.dispatch(&map, 1000, Input('g'), 0, now);
	assert_eq!(
		resolver.dispatch(&map, 1000, Input('!'), 0, now),
		Dispatch::Consumed
	);
	resolver.dispatch(&map, 1000, Input('g'), 0, now);
	resolver.cancel();
	assert!(resolver.pending_label().is_none());
}
#[test]
fn leader_conversion_roundtrip_and_default_inheritance_are_in_memory() {
	let catalog = catalog();
	let mut config = catalog.defaults();
	config.change_leader(Some(Input('g')), false);
	assert_eq!(config.bindings[&1][0], Some(vec![Input('@'), Input('g')]));
	catalog.compile(&config).unwrap();
	let bytes = serde_json::to_vec(&config).unwrap();
	let mut decoded: Configuration<u8, Input> = serde_json::from_slice(&bytes).unwrap();
	assert_eq!(decoded, config);
	decoded.bindings.remove(&3);
	catalog.inherit_defaults(&mut decoded);
	assert_eq!(decoded, config);
	decoded.change_leader(None, false);
	assert_eq!(decoded.hint(1), "g g");
	assert_eq!(decoded, catalog.defaults());
	decoded.change_leader(Some(Input('Z')), false);
	assert!(decoded.bindings[&3][0].is_none());
	catalog.compile(&decoded).unwrap();
	decoded.change_leader(Some(Input('g')), false);
	decoded.change_leader(None, true);
	assert!(decoded.bindings[&1][0].is_none());
}
