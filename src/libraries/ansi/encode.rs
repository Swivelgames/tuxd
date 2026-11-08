use super::{Document, TextStyle};

impl Document {
	pub fn to_raw_ansi(&self) -> String {
		let text = self.text();
		let mut output = String::from("\x1b[0m");
		let mut previous = TextStyle::default();
		for run in &self.styles.runs {
			if run.style != previous {
				output.push_str(&run.style.ansi());
				previous = run.style;
			}
			output.push_str(&text[run.start..run.end]);
		}
		output.push_str("\x1b[0m");
		output
	}

	pub fn to_escaped_ansi(&self) -> String {
		escape(&self.to_raw_ansi())
	}
}

/// Encode ESC, newline, tab, and backslash without adding display rows.
/// This is notation conversion, not sanitization of arbitrary terminal controls.
pub fn escape(raw: &str) -> String {
	let mut output = String::new();
	for ch in raw.chars() {
		match ch {
			'\x1b' => output.push_str("\\x1b"),
			'\n' => output.push_str("\\n"),
			'\t' => output.push_str("\\t"),
			'\\' => output.push_str("\\\\"),
			ch => output.push(ch),
		}
	}
	output
}
