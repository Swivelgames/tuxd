//! Bind prepared release notes and assets to a clean, tagged source commit.
use crate::{archive::digest, git, process::Runner};
use anyhow::{Result, ensure};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{
	collections::BTreeMap,
	fs,
	path::{Component, Path},
};

#[derive(Serialize, Deserialize)]
pub struct Receipt {
	pub version: Version,
	pub commit: String,
	pub files: BTreeMap<String, String>,
}

impl Receipt {
	pub fn save(&self, directory: &Path) -> Result<()> {
		write_json(&directory.join("release.json"), self)
	}
	pub fn load(runner: &Runner, directory: &Path) -> Result<Self> {
		let receipt: Self = serde_json::from_slice(&fs::read(directory.join("release.json"))?)?;
		git::stable(&receipt.version.to_string())?;
		ensure!(
			git::tagged(runner, &receipt.version)? == receipt.commit,
			"Prepared source commit changed"
		);
		receipt.verify(directory)?;
		Ok(receipt)
	}
	pub fn verify(&self, directory: &Path) -> Result<()> {
		let root = directory.canonicalize()?;
		ensure!(
			self.files.contains_key("notes.md") && self.files.contains_key("artifacts/SHA256SUMS"),
			"Incomplete release receipt"
		);
		for (name, expected) in &self.files {
			ensure!(
				Path::new(name)
					.components()
					.all(|c| matches!(c, Component::Normal(_)))
					&& !name.contains('\\'),
				"Invalid artifact path: {name}"
			);
			let path = root.join(name).canonicalize()?;
			ensure!(
				path.starts_with(&root),
				"Artifact escapes release directory"
			);
			ensure!(digest(&path)? == *expected, "Prepared file changed: {name}");
		}
		Ok(())
	}
	pub fn assets(&self) -> impl Iterator<Item = &str> {
		self.files
			.keys()
			.filter_map(|name| name.strip_prefix("artifacts/"))
	}
}

pub fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
	let mut bytes = Vec::new();
	let formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");
	value.serialize(&mut serde_json::Serializer::with_formatter(
		&mut bytes, formatter,
	))?;
	bytes.push(b'\n');
	fs::write(path, bytes)?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn receipt_rejects_tampering_and_traversal() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let root = temp.path();
		fs::create_dir(root.join("artifacts"))?;
		fs::write(root.join("notes.md"), "reviewed")?;
		fs::write(root.join("artifacts/SHA256SUMS"), "checksums")?;
		let mut receipt = Receipt {
			version: git::stable("0.3.0")?,
			commit: "abc".into(),
			files: BTreeMap::new(),
		};
		for name in ["notes.md", "artifacts/SHA256SUMS"] {
			receipt.files.insert(name.into(), digest(&root.join(name))?);
		}
		receipt.verify(root)?;
		fs::write(root.join("notes.md"), "changed")?;
		assert!(receipt.verify(root).is_err());
		fs::write(root.join("notes.md"), "reviewed")?;
		receipt.files.insert("../outside".into(), "hash".into());
		assert!(receipt.verify(root).is_err());
		Ok(())
	}
}
