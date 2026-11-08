//! Encode, publish, then update the session's saved association only after success.
use crate::{adapters::native_document, persistence, session::DocumentSession};
use std::{io, path::PathBuf};

#[derive(Clone, Copy)]
pub enum Format {
	Native,
	Raw,
	Escaped,
}
impl Format {
	pub fn notice(self) -> &'static str {
		match self {
			Self::Native => "Document saved",
			Self::Raw | Self::Escaped => "ANSI exported",
		}
	}
}

pub fn publish(
	session: &mut DocumentSession,
	path: PathBuf,
	format: Format,
	replace: bool,
) -> io::Result<()> {
	if !matches!(format, Format::Native)
		&& let Some(current) = &session.path
	{
		let same = path == *current
			|| path
				.canonicalize()
				.ok()
				.zip(current.canonicalize().ok())
				.is_some_and(|(a, b)| a == b);
		if same {
			return Err(io::Error::other(
				"Choose an export destination other than the open document",
			));
		}
	}
	let document = &session.editing.document;
	let bytes = match format {
		Format::Native => native_document::encode(document)?,
		Format::Raw => document.to_raw_ansi().into_bytes(),
		Format::Escaped => document.to_escaped_ansi().into_bytes(),
	};
	persistence::write_atomic(&path, &bytes, replace)?;
	match format {
		Format::Native => session.saved(path),
		Format::Raw | Format::Escaped => session.exported(path),
	}
	Ok(())
}
