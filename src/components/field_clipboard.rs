//! Completion ownership for field copies; payloads are captured at activation.
use crate::{
	features::clipboard_notice,
	libraries::clipboard::{Service, WriteRequest},
};
use std::io::Write;

#[derive(Default)]
pub struct Copy {
	pending: Option<WriteRequest>,
}
impl Copy {
	pub fn begin(&mut self, service: &Service, text: String) -> Option<String> {
		if self.pending.is_some() {
			return None;
		}
		self.pending = Some(service.write(text)?);
		Some("Copying selection…".into())
	}
	pub fn poll(&mut self, output: &mut impl Write) -> Option<String> {
		let completion = self.pending.as_mut()?.poll(output)?;
		self.pending = None;
		Some(clipboard_notice::copied(completion.result, "selection"))
	}
}
