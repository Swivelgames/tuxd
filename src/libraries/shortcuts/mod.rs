//! Supplied catalogs, binding data, validation and sequence resolution. No views or I/O.
mod catalog;
mod compiler;
mod configuration;
mod resolver;
pub use catalog::{Catalog, Context, Descriptor};
pub use configuration::Configuration;
pub use resolver::{Dispatch, Resolver};

pub trait BindingKey: ratatui_which_key::Key + Clone + Eq {
	fn leader() -> Self;
	fn is_leader(&self) -> bool;
	fn normalized(self) -> Self;
	fn token(&self) -> String;
	fn label(&self) -> String;
	/// Whether this encoding may be used as a leader (independent of reserved keys).
	fn leader_allowed(&self) -> bool;
	fn escape(&self) -> bool;
	fn is_printable(&self) -> bool;
}
pub type Slots<K> = [Option<Vec<K>>; 3];
pub type Map<A, S, K> = ratatui_which_key::Keymap<K, S, A, &'static str>;
pub fn sequence_label<K: BindingKey>(keys: &[K]) -> String {
	keys.iter()
		.map(BindingKey::label)
		.collect::<Vec<_>>()
		.join(" ")
}
