//! Delegate manifest and lockfile edits to cargo-release's version-only step.
use crate::{git, process::Runner};
use anyhow::{Context, Result, ensure};
use semver::Version;

pub const CARGO_RELEASE: &str = "1.1.5";

pub fn prerequisite(runner: &Runner) -> Result<()> {
	let version = runner
		.output(runner.command("cargo").args(["release", "--version"]))
		.with_context(|| {
			format!(
				"Install the pinned helper: cargo install cargo-release --version {CARGO_RELEASE} --locked"
			)
		})?;
	ensure!(
		version.split_whitespace().last() == Some(CARGO_RELEASE),
		"Install the pinned helper: cargo install cargo-release --version {CARGO_RELEASE} --locked"
	);
	Ok(())
}

pub fn bump(runner: &Runner, version: &Version, execute: bool) -> Result<()> {
	ensure!(
		*version > git::current(runner)?,
		"Release version must increase"
	);
	prerequisite(runner)?;
	let mut command = runner.command("cargo");
	command
		.args([
			"release",
			"version",
			&version.to_string(),
			"--isolated",
			"--config",
		])
		.arg(runner.root.join("xtask/release.toml"));
	if execute {
		command.args(["--execute", "--no-confirm"]);
	}
	runner.run(&mut command)?;
	if execute {
		ensure!(
			git::current(runner)? == *version,
			"cargo-release did not set the requested version"
		);
	}
	Ok(())
}
