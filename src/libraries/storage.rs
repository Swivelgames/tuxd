//! Bounded byte reads and atomic single-file publication; callers own codecs and policy.
use std::{
	fs,
	io::{self, Read, Write},
	path::Path,
};

pub struct ReadPolicy {
	pub limit: u64,
	pub overflow: &'static str,
	pub special_file: &'static str,
	pub error_kind: io::ErrorKind,
}

impl ReadPolicy {
	pub fn read(&self, reader: impl Read) -> io::Result<Vec<u8>> {
		let mut bytes = Vec::new();
		reader.take(self.limit + 1).read_to_end(&mut bytes)?;
		if bytes.len() as u64 > self.limit {
			return Err(io::Error::new(self.error_kind, self.overflow));
		}
		Ok(bytes)
	}

	pub fn file(&self, path: &Path) -> io::Result<Vec<u8>> {
		use std::os::unix::fs::OpenOptionsExt;
		let file = fs::OpenOptions::new()
			.read(true)
			.custom_flags(libc::O_NONBLOCK)
			.open(path)?;
		let metadata = file.metadata()?;
		if !metadata.is_file() {
			return Err(io::Error::new(self.error_kind, self.special_file));
		}
		if metadata.len() > self.limit {
			return Err(io::Error::new(self.error_kind, self.overflow));
		}
		self.read(file)
	}
}

/// Document callers validate destination type and supply existing permissions.
/// Configuration callers replace the directory entry using private temporary-file permissions.
pub fn publish(
	path: &Path,
	bytes: &[u8],
	replace: bool,
	permissions: Option<fs::Permissions>,
) -> io::Result<()> {
	let parent = path
		.parent()
		.filter(|p| !p.as_os_str().is_empty())
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
	use std::os::unix::fs::PermissionsExt;

	const POLICY: ReadPolicy = ReadPolicy {
		limit: 4,
		overflow: "too large",
		special_file: "not regular",
		error_kind: io::ErrorKind::InvalidData,
	};

	#[test]
	fn bounded_stream_and_file_reads_preserve_failure_distinctions() {
		assert_eq!(POLICY.read(&b"four"[..]).unwrap(), b"four");
		assert!(POLICY.read(io::repeat(b'x')).is_err());
		let directory = tempfile::tempdir().unwrap();
		assert_eq!(
			POLICY.file(directory.path()).unwrap_err().to_string(),
			"not regular"
		);
		let path = directory.path().join("source");
		assert_eq!(
			POLICY.file(&path).unwrap_err().kind(),
			io::ErrorKind::NotFound
		);
		fs::write(&path, b"large").unwrap();
		assert_eq!(POLICY.file(&path).unwrap_err().to_string(), "too large");
	}

	#[test]
	fn fifo_is_rejected_without_waiting_for_a_writer() {
		use std::os::unix::ffi::OsStrExt;
		let directory = tempfile::tempdir().unwrap();
		let path = directory.path().join("fifo");
		let name = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
		assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
		assert_eq!(POLICY.file(&path).unwrap_err().to_string(), "not regular");
	}

	#[test]
	fn publication_keeps_no_clobber_and_explicit_permission_policies() {
		let directory = tempfile::tempdir().unwrap();
		let path = directory.path().join("destination");
		publish(&path, b"original", false, None).unwrap();
		let error = publish(&path, b"lost", false, None).unwrap_err();
		assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
		assert_eq!(fs::read(&path).unwrap(), b"original");
		publish(
			&path,
			b"replacement",
			true,
			Some(fs::Permissions::from_mode(0o640)),
		)
		.unwrap();
		assert_eq!(
			fs::metadata(&path).unwrap().permissions().mode() & 0o777,
			0o640
		);
		publish(&path, b"config", true, None).unwrap();
		assert_eq!(
			fs::metadata(&path).unwrap().permissions().mode() & 0o777,
			0o600
		);
		assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
	}
}
