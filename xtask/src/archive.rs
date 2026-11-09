//! Portable archive and checksum operations; no tar/sha256sum executables required.
use anyhow::{Result, ensure};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use sha2::{Digest, Sha256};
use std::{
	fs::{self, File},
	io::Read,
	path::Path,
};

pub fn digest(path: &Path) -> Result<String> {
	let mut file = File::open(path)?;
	let mut hash = Sha256::new();
	let mut buffer = [0; 65536];
	loop {
		let count = file.read(&mut buffer)?;
		if count == 0 {
			break;
		}
		hash.update(&buffer[..count]);
	}
	Ok(format!("{:x}", hash.finalize()))
}

pub fn pack(folder: &Path, destination: &Path) -> Result<()> {
	let encoder = GzEncoder::new(File::create(destination)?, Compression::default());
	let mut archive = tar::Builder::new(encoder);
	archive.append_dir_all(folder.file_name().unwrap(), folder)?;
	archive.into_inner()?.finish()?;
	Ok(())
}

pub fn unpack(source: &Path, destination: &Path) -> Result<()> {
	tar::Archive::new(GzDecoder::new(File::open(source)?)).unpack(destination)?;
	Ok(())
}

pub fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
	fs::create_dir_all(destination)?;
	for entry in fs::read_dir(source)? {
		let entry = entry?;
		let target = destination.join(entry.file_name());
		if entry.file_type()?.is_dir() {
			copy_tree(&entry.path(), &target)?;
		} else {
			ensure!(
				entry.file_type()?.is_file(),
				"Unexpected non-file in release tree: {:?}",
				entry.path()
			);
			fs::copy(entry.path(), target)?;
		}
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn archive_roundtrip_and_checksum() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let source = temp.path().join("source");
		fs::create_dir(&source)?;
		fs::write(source.join("hello.txt"), "hello")?;
		let before = digest(&source.join("hello.txt"))?;
		assert_eq!(
			before,
			"2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
		);
		let bundle = temp.path().join("source.tar.gz");
		pack(&source, &bundle)?;
		let extracted = temp.path().join("extracted");
		unpack(&bundle, &extracted)?;
		assert_eq!(before, digest(&extracted.join("source/hello.txt"))?);
		Ok(())
	}
}
