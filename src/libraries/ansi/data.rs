use super::{Document, Styles};
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

/// Untrusted, serializable document data. Convert into Document to validate it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentData {
	pub text: String,
	pub styles: Styles,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
	InvalidPosition,
	ReversedRange,
	UnsupportedControl,
	InvalidColor,
	InvalidStyleRanges,
	IncompleteStyleCoverage,
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			Self::InvalidPosition => "Position must be a document grapheme boundary",
			Self::ReversedRange => "Range end must not precede its start",
			Self::UnsupportedControl => "Document contains unsupported control characters",
			Self::InvalidColor => "ANSI color must be between 0 and 15",
			Self::InvalidStyleRanges => {
				"Style ranges must cover text contiguously at grapheme boundaries"
			}
			Self::IncompleteStyleCoverage => "Style ranges do not cover the document",
		})
	}
}

impl std::error::Error for Error {}

pub(super) fn validate_text(text: &str) -> Result<(), Error> {
	if text
		.chars()
		.any(|ch| ch.is_control() && !matches!(ch, '\n' | '\t'))
	{
		return Err(Error::UnsupportedControl);
	}
	Ok(())
}

impl TryFrom<DocumentData> for Document {
	type Error = Error;

	fn try_from(data: DocumentData) -> Result<Self, Error> {
		validate_text(&data.text)?;
		let boundaries: std::collections::HashSet<_> = data
			.text
			.grapheme_indices(true)
			.map(|(offset, _)| offset)
			.chain([data.text.len()])
			.collect();
		let mut end = 0;
		let mut styles = Styles::default();
		for run in data.styles.runs {
			if run.start != end
				|| run.end <= run.start
				|| !boundaries.contains(&run.start)
				|| !boundaries.contains(&run.end)
			{
				return Err(Error::InvalidStyleRanges);
			}
			run.style.validate()?;
			styles.push(run.start, run.end, run.style);
			end = run.end;
		}
		if end != data.text.len() {
			return Err(Error::IncompleteStyleCoverage);
		}
		Ok(Self {
			lines: data.text.split('\n').map(str::to_owned).collect(),
			styles,
		})
	}
}

impl Document {
	pub fn to_data(&self) -> DocumentData {
		DocumentData {
			text: self.text(),
			styles: self.styles.clone(),
		}
	}
}
