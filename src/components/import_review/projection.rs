use crate::libraries::ansi::Import;

pub struct Projection {
	pub lines: Vec<String>,
	pub marks: Vec<Vec<bool>>,
}

impl Projection {
	pub fn new(import: &Import) -> Self {
		let mut lines = vec![String::new()];
		let mut marks = vec![Vec::new()];
		let mut issue_index = 0;
		for (offset, ch) in import.source.char_indices() {
			if ch == '\n' {
				lines.push(String::new());
				marks.push(Vec::new());
				continue;
			}
			let value = display_character(ch);
			while issue_index < import.issues.len() && import.issues[issue_index].end <= offset {
				issue_index += 1;
			}
			let error = import
				.issues
				.get(issue_index)
				.is_some_and(|issue| offset >= issue.start && offset < issue.end);
			lines.last_mut().unwrap().push_str(&value);
			marks
				.last_mut()
				.unwrap()
				.extend(std::iter::repeat_n(error, value.len()));
		}
		Self { lines, marks }
	}
}

/// Review notation preserves printable source and exposes controls without executing them.
pub(super) fn display_character(ch: char) -> String {
	if ch == '\x1b' {
		"\\x1b".to_owned()
	} else if ch.is_control() {
		ch.escape_default().to_string()
	} else {
		ch.to_string()
	}
}
