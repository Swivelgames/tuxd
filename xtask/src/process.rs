//! Shell-free subprocess execution with an optional retained release log.
mod child;
use anyhow::{Context, Result, ensure};
pub use child::ManagedChild;
use std::{
	fs::OpenOptions,
	io::Write,
	path::{Path, PathBuf},
	process::{Command, Stdio},
};

pub struct Runner {
	pub root: PathBuf,
	pub log: Option<PathBuf>,
}

impl Runner {
	pub fn new(root: PathBuf) -> Self {
		Self { root, log: None }
	}
	pub fn require(&self, programs: &[&str]) -> Result<()> {
		let search = std::env::var_os("PATH").unwrap_or_default();
		let directories: Vec<_> = std::env::split_paths(&search).collect();
		for program in programs {
			let found = directories.iter().any(|directory| {
				directory.join(program).is_file()
					|| directory
						.join(format!("{program}{}", std::env::consts::EXE_SUFFIX))
						.is_file()
			});
			ensure!(found, "Required tool not found on PATH: {program}");
		}
		Ok(())
	}

	pub fn command(&self, program: impl AsRef<std::ffi::OsStr>) -> Command {
		let mut command = Command::new(program);
		command.current_dir(&self.root);
		command
	}
	pub fn run(&self, command: &mut Command) -> Result<()> {
		println!("+ {command:?}");
		if let Some(path) = &self.log {
			let mut log = OpenOptions::new().create(true).append(true).open(path)?;
			writeln!(log, "+ {command:?}")?;
			command
				.stdout(Stdio::from(log.try_clone()?))
				.stderr(Stdio::from(log));
		}
		let status = command
			.status()
			.with_context(|| format!("Could not start {command:?}"))?;
		ensure!(
			status.success(),
			"Command failed ({status}): {command:?}; log: {:?}",
			self.log
		);
		Ok(())
	}
	pub fn output(&self, command: &mut Command) -> Result<String> {
		let result = command
			.output()
			.with_context(|| format!("Could not start {command:?}"))?;
		ensure!(
			result.status.success(),
			"Command failed: {command:?}\n{}",
			String::from_utf8_lossy(&result.stderr)
		);
		Ok(String::from_utf8(result.stdout)?
			.trim_end_matches('\n')
			.trim_end_matches('\r')
			.to_owned())
	}
	pub fn git(&self, args: &[&str]) -> Result<String> {
		self.output(self.command("git").args(args))
	}
	pub fn cargo(&self, args: &[&str]) -> Result<()> {
		self.run(self.command("cargo").args(args))
	}
}

pub fn outside_checkout(root: &Path, path: &Path) -> Result<PathBuf> {
	let absolute = std::path::absolute(path)?;
	// Resolve an existing ancestor so symlinks cannot place generated output in the checkout.
	let mut ancestor = absolute.as_path();
	let mut suffix = Vec::new();
	while !ancestor.exists() {
		suffix.push(
			ancestor
				.file_name()
				.context("Invalid output directory")?
				.to_owned(),
		);
		ancestor = ancestor.parent().context("Invalid output parent")?;
	}
	let mut resolved = ancestor.canonicalize()?;
	for part in suffix.into_iter().rev() {
		resolved.push(part);
	}
	ensure!(
		!resolved.starts_with(root.canonicalize()?),
		"Output must be outside the checkout"
	);
	Ok(resolved)
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn output_is_outside_checkout() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let root = temp.path().join("repo");
		std::fs::create_dir(&root)?;
		assert!(outside_checkout(&root, &root.join("generated")).is_err());
		assert_eq!(
			outside_checkout(&root, &temp.path().join("release/new"))?,
			temp.path().canonicalize()?.join("release/new")
		);
		Ok(())
	}
	#[cfg(unix)]
	#[test]
	fn output_cannot_follow_symlink_into_checkout() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let root = temp.path().join("repo");
		std::fs::create_dir(&root)?;
		let link = temp.path().join("link");
		std::os::unix::fs::symlink(&root, &link)?;
		assert!(outside_checkout(&root, &link.join("generated")).is_err());
		Ok(())
	}
}
