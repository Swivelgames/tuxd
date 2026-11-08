use super::{BindingKey, Slots, sequence_label};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
	deny_unknown_fields,
	bound(deserialize = "A: Ord + Deserialize<'de>, K: Deserialize<'de>")
)]
pub struct Configuration<A: Ord, K> {
	pub version: u32,
	pub timeout_ms: u64,
	#[serde(default)]
	pub leader: Option<K>,
	pub bindings: BTreeMap<A, Slots<K>>,
}

impl<A: Ord, K: BindingKey> Configuration<A, K> {
	pub fn change_leader(&mut self, leader: Option<K>, remove: bool) {
		let old = self.leader.clone();
		for slots in self.bindings.values_mut() {
			for slot in slots {
				let Some(keys) = slot else {
					continue;
				};
				if let Some(new) = &leader {
					if keys.first() == Some(new) {
						if keys.len() == 1 {
							*slot = None;
						} else {
							keys[0] = K::leader();
						}
					}
				} else if keys.first().is_some_and(K::is_leader) {
					if remove {
						*slot = None;
					} else if let Some(old) = &old {
						keys[0] = old.clone();
					}
				}
			}
		}
		self.leader = leader;
	}

	pub fn hint(&self, action: A) -> String {
		self.bindings
			.get(&action)
			.and_then(|slots| slots.iter().flatten().next())
			.map(|s| sequence_label(s))
			.unwrap_or_default()
	}
}
