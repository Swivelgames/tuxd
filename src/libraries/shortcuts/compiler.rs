use super::{BindingKey, Catalog, Configuration, Map, sequence_label};

impl<
	A: Copy + Ord + std::fmt::Debug + std::fmt::Display,
	S: Copy + Ord + std::fmt::Debug,
	K: BindingKey,
> Catalog<A, S, K>
{
	pub fn compile(&self, config: &Configuration<A, K>) -> Result<Map<A, S, K>, String> {
		if config.version != 2 {
			return Err(format!(
				"Unsupported shortcuts version {} (expected 2)",
				config.version
			));
		}
		if !(1..=60_000).contains(&config.timeout_ms) {
			return Err("Sequence timeout must be 1–60000 milliseconds".into());
		}
		if let Some(leader) = &config.leader
			&& (leader.is_leader()
				|| self.reserved.iter().any(|(key, _)| key == leader)
				|| !leader.leader_allowed()
				|| *leader != leader.clone().normalized())
		{
			return Err("This key cannot be used as Leader".into());
		}
		if config
			.bindings
			.keys()
			.any(|id| !self.actions.iter().any(|action| action.id == *id))
		{
			return Err("Unknown action in configuration".into());
		}

		let mut entries = Vec::new();
		for spec in &self.actions {
			let slots = config
				.bindings
				.get(&spec.id)
				.ok_or_else(|| format!("Missing action: {}", spec.label))?;
			for (slot, keys) in slots.iter().enumerate() {
				let Some(keys) = keys else {
					continue;
				};
				let symbolic = keys.first().is_some_and(K::is_leader);
				if keys.iter().skip(1).any(K::is_leader) || (symbolic && keys.len() < 2) {
					return Err("Leader must prefix at least one key".into());
				}
				if keys.is_empty() || keys.len() > 10 {
					return Err(format!(
						"{} slot {}: use 1–10 keys or unset",
						spec.label,
						slot + 1
					));
				}
				let mut keys = keys.clone();
				if symbolic {
					keys[0] = config.leader.clone().ok_or("Leader is unset")?;
				} else if keys.first() == config.leader.as_ref() && config.leader.is_some() {
					return Err("Use <leader> as the dedicated prefix".into());
				}
				for key in &keys {
					if *key != key.clone().normalized() {
						return Err(format!(
							"{} slot {}: noncanonical key {}",
							spec.label,
							slot + 1,
							key.label()
						));
					}
					if let Some((_, purpose)) =
						self.reserved.iter().find(|(reserved, _)| reserved == key)
					{
						return Err(format!("{} is reserved for {purpose}", key.label()));
					}
				}
				if keys.len() > 1 && keys.iter().any(|k| k.escape()) {
					return Err("Esc cancels sequences and cannot be part of one".into());
				}
				if keys[0].is_printable() && spec.scopes.iter().any(|scope| self.contexts[scope]) {
					return Err(format!(
						"{} would consume ordinary text in an editable context ({})",
						spec.label,
						keys[0].label()
					));
				}
				entries.push((spec, slot, keys));
			}
		}

		for (i, (a, slot_a, keys_a)) in entries.iter().enumerate() {
			for (b, slot_b, keys_b) in &entries[i + 1..] {
				if !(keys_a.starts_with(keys_b) || keys_b.starts_with(keys_a)) {
					continue;
				}
				let scopes: Vec<_> = a.scopes.iter().filter(|s| b.scopes.contains(s)).collect();
				if !scopes.is_empty() {
					return Err(format!(
						"{} slot {} ({}) conflicts with {} slot {} ({}) in {:?}",
						a.id,
						slot_a + 1,
						sequence_label(keys_a),
						b.id,
						slot_b + 1,
						sequence_label(keys_b),
						scopes
					));
				}
			}
		}

		let mut map = Map::new();
		for (spec, _, keys) in &entries {
			let token: String = keys.iter().map(K::token).collect();
			for scope in &spec.scopes {
				map.bind(&token, spec.id, spec.category, *scope);
				// Name groups from the sections of their effective descendants.
				for length in 1..keys.len() {
					let prefix = &keys[..length];
					let same_section = entries
						.iter()
						.filter(|(s, _, seq)| s.scopes.contains(scope) && seq.starts_with(prefix))
						.all(|(s, _, _)| s.category == spec.category);
					map.describe_group_for_scope(
						&prefix.iter().map(K::token).collect::<String>(),
						if same_section {
							spec.category
						} else {
							"Actions"
						},
						*scope,
					);
				}
			}
		}
		Ok(map)
	}
}
