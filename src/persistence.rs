//! Versioned document storage. UI/session state is intentionally excluded.
use crate::{
	document::Document,
	styling::{Ink, Styles},
};
use serde::{Deserialize, Serialize};
use std::{
	fs,
	io::{self, Read, Write},
	path::Path,
};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileDocument {
	version: u32,
	text: String,
	styles: Styles,
}

pub fn encode(document: &Document) -> io::Result<Vec<u8>> {
	let file = FileDocument {
		version: 1,
		text: document.lines.join("\n"),
		styles: document.styles.clone(),
	};
	validate(&file)?;
	let mut bytes = Vec::new();
	let formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");
	file.serialize(&mut serde_json::Serializer::with_formatter(
		&mut bytes, formatter,
	))
	.map_err(io::Error::other)?;
	bytes.push(b'\n');
	Ok(bytes)
}

fn invalid(message: &str) -> io::Error {
	io::Error::new(io::ErrorKind::InvalidData, message)
}

fn validate(file: &FileDocument) -> io::Result<()> {
	if file.version != 1 {
		return Err(invalid("Unsupported document version"));
	}
	if file
		.text
		.chars()
		.any(|ch| ch.is_control() && !matches!(ch, '\n' | '\t'))
	{
		return Err(invalid("Document contains unsupported control characters"));
	}
	let boundaries: std::collections::HashSet<_> = file
		.text
		.grapheme_indices(true)
		.map(|(offset, _)| offset)
		.chain([file.text.len()])
		.collect();
	let mut end = 0;
	for run in &file.styles.runs {
		if run.start != end
			|| run.end <= run.start
			|| !boundaries.contains(&run.start)
			|| !boundaries.contains(&run.end)
		{
			return Err(invalid(
				"Style ranges must cover text contiguously at grapheme boundaries",
			));
		}
		for ink in [run.style.foreground, run.style.background] {
			if matches!(ink, Ink::Ansi(n) if n > 15) {
				return Err(invalid("ANSI color must be between 0 and 15"));
			}
		}
		end = run.end;
	}
	if end != file.text.len() {
		return Err(invalid("Style ranges do not cover the document"));
	}
	Ok(())
}

pub fn decode(bytes: &[u8]) -> io::Result<Document> {
	let file: FileDocument = serde_json::from_slice(bytes).map_err(io::Error::other)?;
	validate(&file)?;
	let mut styles = Styles::default();
	for run in file.styles.runs {
		styles.push(run.start, run.end, run.style);
	}
	Ok(Document {
		lines: file.text.split('\n').map(str::to_owned).collect(),
		styles,
	})
}

pub fn load(path: &Path) -> io::Result<Document> {
	decode(&read_file(path)?)
}

/// Bound reads and refuse special files so a FIFO cannot block the event loop.
pub fn read_file(path: &Path) -> io::Result<Vec<u8>> {
	use std::os::unix::fs::OpenOptionsExt;
	const LIMIT: u64 = 16 * 1024 * 1024;
	let file = fs::OpenOptions::new()
		.read(true)
		.custom_flags(libc::O_NONBLOCK)
		.open(path)?;
	let metadata = file.metadata()?;
	if !metadata.is_file() {
		return Err(invalid("Source must be a regular file"));
	}
	if metadata.len() > LIMIT {
		return Err(invalid("Document/import exceeds the 16 MiB limit"));
	}
	let mut bytes = Vec::new();
	file.take(LIMIT + 1).read_to_end(&mut bytes)?;
	if bytes.len() as u64 > LIMIT {
		return Err(invalid("Document/import exceeds the 16 MiB limit"));
	}
	Ok(bytes)
}

/// The caller must obtain overwrite confirmation before passing `replace = true`.
pub fn write_atomic(path: &Path, bytes: &[u8], replace: bool) -> io::Result<()> {
	let permissions = match fs::symlink_metadata(path) {
		Ok(metadata) => {
			if !metadata.is_file() {
				return Err(invalid(
					"Destination must be a regular file, not a symlink or directory",
				));
			}
			if !replace {
				return Err(io::Error::new(
					io::ErrorKind::AlreadyExists,
					"Destination already exists",
				));
			}
			Some(metadata.permissions())
		}
		Err(error) if error.kind() == io::ErrorKind::NotFound => None,
		Err(error) => return Err(error),
	};
	let parent = path
		.parent()
		.filter(|path| !path.as_os_str().is_empty())
		.unwrap_or(Path::new("."));
	let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
	if let Some(permissions) = permissions {
		temporary.as_file().set_permissions(permissions)?;
	}
	temporary.write_all(bytes)?;
	temporary.as_file().sync_all()?;
	if replace {
		temporary.persist(path).map_err(|error| error.error)?;
	} else {
		temporary
			.persist_noclobber(path)
			.map_err(|error| error.error)?;
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{document::Position, styling::TextStyle};
	#[test]
	fn included_example_is_a_valid_styled_document() {
		let document = decode(include_bytes!("../examples/welcome.tuxd.json")).unwrap();
		assert!(document.styles.runs.iter().any(|run| run.style.bold));
		assert_eq!(decode(&encode(&document).unwrap()).unwrap(), document);
	}

	#[test]
	fn roundtrip_preserves_unicode_and_all_color_kinds() {
		for ink in [
			Ink::Default,
			Ink::Ansi(15),
			Ink::Indexed(255),
			Ink::Rgb(0, 128, 255),
		] {
			let mut document = Document::default();
			document.insert(Position::default(), "e\u{301}\t界\n👩‍💻\n");
			document
				.styles
				.apply(0, document.lines.join("\n").len(), |_| TextStyle {
					bold: true,
					dim: true,
					italic: true,
					underline: true,
					strike: true,
					foreground: ink,
					background: ink,
				});
			assert_eq!(decode(&encode(&document).unwrap()).unwrap(), document);
		}
	}
	#[test]
	fn rejects_invalid_version_ranges_controls_and_colors() {
		let mut doc = Document::default();
		doc.insert(Position::default(), "e\u{301}");
		let original: serde_json::Value = serde_json::from_slice(&encode(&doc).unwrap()).unwrap();
		for (pointer, value) in [
			("/version", serde_json::json!(2)),
			("/styles/runs/0/end", serde_json::json!(1)),
			("/text", serde_json::json!("\u{1b}")),
			(
				"/styles/runs/0/style/foreground",
				serde_json::json!({"Ansi":16}),
			),
		] {
			let mut file = original.clone();
			*file.pointer_mut(pointer).unwrap() = value;
			assert!(decode(&serde_json::to_vec(&file).unwrap()).is_err());
		}
	}
	#[test]
	fn writes_require_explicit_replace_and_failures_preserve_existing_file() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("test.tuxd.json");
		write_atomic(&path, &encode(&Document::default()).unwrap(), false).unwrap();
		assert_eq!(load(&path).unwrap(), Document::default());
		let before = fs::read(&path).unwrap();
		assert!(write_atomic(&path, b"new", false).is_err());
		assert_eq!(fs::read(&path).unwrap(), before);
		write_atomic(&path, b"new", true).unwrap();
		assert_eq!(fs::read(&path).unwrap(), b"new");
		let link = dir.path().join("link");
		std::os::unix::fs::symlink(&path, &link).unwrap();
		assert!(write_atomic(&link, b"bad", true).is_err());
		assert_eq!(fs::read(&path).unwrap(), b"new");
	}
}
