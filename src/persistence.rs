//! Filesystem operations. Native encoding is a separate adapter.
use crate::adapters::native_document::decode;
use crate::libraries::ansi::Document;
use std::{fs, io, path::Path};

fn invalid(message: &str) -> io::Error {
	io::Error::new(io::ErrorKind::InvalidData, message)
}

pub fn load(path: &Path) -> io::Result<Document> {
	decode(&read_file(path)?)
}

/// Bound reads and refuse special files so a FIFO cannot block the event loop.
pub fn read_file(path: &Path) -> io::Result<Vec<u8>> {
	crate::libraries::storage::ReadPolicy {
		limit: 16 * 1024 * 1024,
		overflow: "Document/import exceeds the 16 MiB limit",
		special_file: "Source must be a regular file",
		error_kind: io::ErrorKind::InvalidData,
	}
	.file(path)
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
	crate::libraries::storage::publish(path, bytes, replace, permissions)
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn writes_require_explicit_replace_and_failures_preserve_existing_file() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("test.tuxd.json");
		write_atomic(
			&path,
			&crate::adapters::native_document::encode(&Document::default()).unwrap(),
			false,
		)
		.unwrap();
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
