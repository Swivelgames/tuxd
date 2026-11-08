use super::{BindingKey, Configuration, Slots};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct Descriptor<A, S, K> {
	pub id: A,
	pub label: &'static str,
	pub category: &'static str,
	pub scopes: Vec<S>,
	pub defaults: Slots<K>,
}

#[derive(Clone, Debug)]
pub struct Context<S> {
	pub id: S,
	pub editable: bool,
}

#[derive(Debug)]
pub struct Catalog<A, S, K> {
	pub(super) actions: Vec<Descriptor<A, S, K>>,
	pub(super) contexts: BTreeMap<S, bool>,
	pub(super) reserved: Vec<(K, &'static str)>,
}

impl<A: Copy + Ord + std::fmt::Debug, S: Copy + Ord + std::fmt::Debug, K: BindingKey>
	Catalog<A, S, K>
{
	pub fn new(
		actions: impl IntoIterator<Item = Descriptor<A, S, K>>,
		contexts: impl IntoIterator<Item = Context<S>>,
		reserved: Vec<(K, &'static str)>,
	) -> Result<Self, String> {
		let mut rules = BTreeMap::new();
		for context in contexts {
			if rules.insert(context.id, context.editable).is_some() {
				return Err(format!("Duplicate context: {:?}", context.id));
			}
		}
		let actions: Vec<_> = actions.into_iter().collect();
		let mut ids = BTreeSet::new();
		for action in &actions {
			if !ids.insert(action.id) {
				return Err(format!("Duplicate action: {:?}", action.id));
			}
			if action.scopes.is_empty()
				|| action.scopes.iter().any(|scope| !rules.contains_key(scope))
			{
				return Err(format!("Invalid contexts for {:?}", action.id));
			}
		}
		Ok(Self {
			actions,
			contexts: rules,
			reserved,
		})
	}

	pub fn actions(&self) -> &[Descriptor<A, S, K>] {
		&self.actions
	}

	pub fn defaults(&self) -> Configuration<A, K> {
		Configuration {
			version: 2,
			timeout_ms: 1000,
			leader: None,
			bindings: self
				.actions
				.iter()
				.map(|a| (a.id, a.defaults.clone()))
				.collect(),
		}
	}

	pub fn inherit_defaults(&self, config: &mut Configuration<A, K>) {
		for action in &self.actions {
			config
				.bindings
				.entry(action.id)
				.or_insert_with(|| action.defaults.clone());
		}
	}
}
