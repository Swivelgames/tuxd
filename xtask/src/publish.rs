//! Opt-in publication adapters; prepared artifact integrity is checked first.
use crate::{archive, git, process::Runner, receipt::Receipt};
use anyhow::{Result, ensure};
use clap::Args;
use std::path::Path;

const GITHUB: &str = "Swivelgames/tuxd";
const GITLAB: &str = "swivel/tux-design";

#[derive(Args, Debug, Default, Clone)]
pub struct Destinations {
	/// Publish a GitHub release and push the public remote.
	#[arg(long)]
	pub github: bool,
	/// Publish a GitLab release and push the origin remote.
	#[arg(long)]
	pub gitlab: bool,
	/// Publish to crates.io (also push public source).
	#[arg(long)]
	pub crates_io: bool,
	/// Publish to all three destinations.
	#[arg(long)]
	pub all: bool,
	/// Push both Git remotes, independently of hosted publication.
	#[arg(long)]
	pub push: bool,
}

impl Destinations {
	pub fn github(&self) -> bool {
		self.all || self.github
	}
	pub fn gitlab(&self) -> bool {
		self.all || self.gitlab
	}
	pub fn crates(&self) -> bool {
		self.all || self.crates_io
	}
	pub fn any(&self) -> bool {
		self.github() || self.gitlab() || self.crates() || self.push
	}
	pub fn remotes(&self) -> Vec<&'static str> {
		let mut result = Vec::new();
		if self.push || self.gitlab() {
			result.push("origin");
		}
		if self.push || self.github() || self.crates() {
			result.push("public");
		}
		result
	}
}

pub fn prerequisites(runner: &Runner, destinations: &Destinations) -> Result<()> {
	runner.require(&["git", "cargo"])?;
	if destinations.github() {
		runner.require(&["gh"])?;
	}
	if destinations.gitlab() {
		runner.require(&["glab"])?;
	}
	Ok(())
}

pub fn run(
	runner: &Runner,
	directory: &Path,
	destinations: &Destinations,
	execute: bool,
) -> Result<()> {
	ensure!(
		destinations.any(),
		"Choose --github, --gitlab, --crates-io, --all, or --push"
	);
	let directory = directory.canonicalize()?;
	let receipt = Receipt::load(runner, &directory)?;
	println!(
		"Publish {}: {destinations:?}; push {:?}",
		receipt.version,
		destinations.remotes()
	);
	if !execute {
		println!("Preview only; add --execute to publish.");
		return Ok(());
	}
	prerequisites(runner, destinations)?;
	git::push(runner, &receipt.version, &destinations.remotes())?;
	if destinations.crates() {
		registry(runner, &receipt)?;
	}
	if destinations.github() {
		github(runner, &directory, &receipt)?;
	}
	if destinations.gitlab() {
		gitlab(runner, &directory, &receipt)?;
	}
	Ok(())
}

fn registry(runner: &Runner, receipt: &Receipt) -> Result<()> {
	runner.cargo(&["publish", "--locked"])?;
	let installed = tempfile::tempdir()?;
	runner.run(
		runner
			.command("cargo")
			.args([
				"install",
				"tuxd",
				"--version",
				&format!("={}", receipt.version),
				"--locked",
				"--root",
			])
			.arg(installed.path()),
	)?;
	let binary = installed
		.path()
		.join("bin")
		.join(format!("tuxd{}", std::env::consts::EXE_SUFFIX));
	ensure!(
		runner.output(runner.command(binary).arg("--version"))?
			== format!("tuxd {}", receipt.version),
		"Registry installation version mismatch"
	);
	Ok(())
}

fn github(runner: &Runner, directory: &Path, receipt: &Receipt) -> Result<()> {
	let tag = format!("v{}", receipt.version);
	let mut create = runner.command("gh");
	create
		.args([
			"release",
			"create",
			&tag,
			"--repo",
			GITHUB,
			"--verify-tag",
			"--draft",
			"--title",
			&format!("tuxd {tag}"),
			"--notes-file",
		])
		.arg(directory.join("notes.md"));
	for asset in receipt.assets() {
		create.arg(directory.join("artifacts").join(asset));
	}
	runner.run(&mut create)?;
	let downloaded = tempfile::tempdir()?;
	runner.run(
		runner
			.command("gh")
			.args(["release", "download", &tag, "--repo", GITHUB, "--dir"])
			.arg(downloaded.path()),
	)?;
	for asset in receipt.assets() {
		ensure!(
			archive::digest(&downloaded.path().join(asset))?
				== receipt.files[&format!("artifacts/{asset}")],
			"Uploaded asset changed: {asset}"
		);
	}
	runner.run(runner.command("gh").args([
		"release",
		"edit",
		&tag,
		"--repo",
		GITHUB,
		"--draft=false",
		"--latest",
	]))
}

fn gitlab(runner: &Runner, directory: &Path, receipt: &Receipt) -> Result<()> {
	let tag = format!("v{}", receipt.version);
	let mut create = runner.command("glab");
	create
		.args([
			"release",
			"create",
			&tag,
			"--repo",
			GITLAB,
			"--name",
			&format!("tuxd {tag}"),
			"--no-update",
			"--no-close-milestone",
			"--notes-file",
		])
		.arg(directory.join("notes.md"));
	for asset in receipt.assets() {
		create.arg(directory.join("artifacts").join(asset));
	}
	runner.run(&mut create)?;
	runner.run(
		runner
			.command("glab")
			.args(["release", "view", &tag, "--repo", GITLAB]),
	)
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn publication_is_opt_in() {
		let none = Destinations::default();
		assert!(!none.any());
		assert!(none.remotes().is_empty());
		let all = Destinations {
			all: true,
			..Default::default()
		};
		assert!(all.github() && all.gitlab() && all.crates());
		assert_eq!(all.remotes(), ["origin", "public"]);
		assert_eq!(
			Destinations {
				crates_io: true,
				..Default::default()
			}
			.remotes(),
			["public"]
		);
		assert_eq!(
			Destinations {
				gitlab: true,
				..Default::default()
			}
			.remotes(),
			["origin"]
		);
		let push = Destinations {
			push: true,
			..Default::default()
		};
		assert!(!push.github() && !push.gitlab() && !push.crates());
		assert_eq!(push.remotes(), ["origin", "public"]);
	}
}
