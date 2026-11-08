//! RGB draft text may be incomplete. Valid edits synchronize all four fields;
//! applying remains an explicit caller operation.
use crate::{
	components::field::{self, Field, FieldAction},
	libraries::ansi::Ink,
};

pub struct RgbDraft {
	pub fields: [Field; 4],
	pub replace_field: bool,
	pub error: Option<String>,
}
impl Default for RgbDraft {
	fn default() -> Self {
		Self {
			fields: ["#80c0ff", "128", "192", "255"].map(|s| Field::new(s.into())),
			replace_field: true,
			error: None,
		}
	}
}
impl RgbDraft {
	pub fn rgb(&self) -> Result<Ink, String> {
		let text = self.fields[0].text().trim();
		let hex = text.strip_prefix('#').unwrap_or(text);
		if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
			return Err("Use #RRGGBB (6 hex digits)".into());
		}
		let mut values = [0; 3];
		for (i, value) in values.iter_mut().enumerate() {
			*value = self.fields[i + 1]
				.text()
				.parse::<u8>()
				.map_err(|_| "RGB components: 0–255".to_owned())?;
		}
		Ok(Ink::Rgb(values[0], values[1], values[2]))
	}
	pub fn edit(&mut self, index: usize, text: &str, backspace: bool) {
		let clean = crate::libraries::text::display::sanitize(text).replace(['\n', '\t'], " ");
		let field = &mut self.fields[index];
		if self.replace_field {
			field.select_all();
		}
		self.replace_field = false;
		if backspace {
			field.apply(FieldAction::Backspace);
		} else {
			field.insert(&clean, 16);
		}
		self.synchronize(index);
	}
	fn synchronize(&mut self, index: usize) {
		if index == 0 {
			let value = self.fields[0].text().trim();
			let value = value.strip_prefix('#').unwrap_or(value);
			if value.len() == 6 && value.bytes().all(|b| b.is_ascii_hexdigit()) {
				let value = u32::from_str_radix(value, 16).unwrap();
				for (i, shift) in [16, 8, 0].into_iter().enumerate() {
					self.fields[i + 1].set(((value >> shift) & 255).to_string());
				}
			}
		} else if let (Ok(r), Ok(g), Ok(b)) = (
			self.fields[1].text().parse::<u8>(),
			self.fields[2].text().parse::<u8>(),
			self.fields[3].text().parse::<u8>(),
		) {
			self.fields[0].set(format!("#{r:02x}{g:02x}{b:02x}"));
		}
		for (n, field) in self.fields.iter_mut().enumerate() {
			if n != index {
				field.apply(FieldAction::End);
			}
		}
		self.error = self.rgb().err();
	}
	pub fn move_caret(&mut self, index: usize, action: crate::shortcuts::ActionId) {
		if let Some(action) = field::action(action) {
			self.fields[index].apply(action);
		}
		self.replace_field = false;
	}
	pub fn set_component(&mut self, index: usize, value: u8) {
		self.replace_field = true;
		self.edit(index, &value.to_string(), false);
		self.replace_field = true;
	}
}
