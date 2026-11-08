use super::{BindingKey, Map, sequence_label};
use ratatui_which_key::NodeResult;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq)]
pub enum Dispatch<A> {
	Action(A),
	Pending,
	Consumed,
	Unbound,
}

pub struct Resolver<S, K> {
	pending: Vec<K>,
	scope: Option<S>,
	last: Option<Instant>,
}

impl<S, K> Default for Resolver<S, K> {
	fn default() -> Self {
		Self {
			pending: Vec::new(),
			scope: None,
			last: None,
		}
	}
}

impl<S: Copy + PartialEq, K: BindingKey> Resolver<S, K> {
	pub fn cancel(&mut self) {
		self.pending.clear();
		self.last = None;
		self.scope = None;
	}

	pub fn expire(&mut self, now: Instant, timeout_ms: u64) -> bool {
		if self.last.is_some_and(|last| {
			now.saturating_duration_since(last) >= Duration::from_millis(timeout_ms)
		}) {
			self.cancel();
			true
		} else {
			false
		}
	}

	pub fn pending_label(&self) -> Option<String> {
		(!self.pending.is_empty()).then(|| format!("{} …", sequence_label(&self.pending)))
	}

	pub fn dispatch<A: Clone>(
		&mut self,
		map: &Map<A, S, K>,
		timeout_ms: u64,
		key: K,
		scope: S,
		now: Instant,
	) -> Dispatch<A> {
		if self.scope != Some(scope) {
			self.cancel();
		}
		self.expire(now, timeout_ms);
		let had_prefix = !self.pending.is_empty();
		if had_prefix && key.escape() {
			self.cancel();
			return Dispatch::Consumed;
		}
		self.scope = Some(scope);
		self.pending.push(key);
		match map.navigate(&self.pending, &scope) {
			Some(NodeResult::Leaf { action }) => {
				self.cancel();
				Dispatch::Action(action)
			}
			Some(NodeResult::Branch { .. }) => {
				self.last = Some(now);
				Dispatch::Pending
			}
			None => {
				self.cancel();
				if had_prefix {
					Dispatch::Consumed
				} else {
					Dispatch::Unbound
				}
			}
		}
	}
}
