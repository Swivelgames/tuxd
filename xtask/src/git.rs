//! Repository invariants and explicit, non-forcing Git release operations.
use crate::process::Runner;
use anyhow::{Context, Result, ensure};
use semver::Version;

pub fn stable(value: &str) -> Result<Version> {
	let version = Version::parse(value)?;
	ensure!(
		version.pre.is_empty() && version.build.is_empty(),
		"Use a stable X.Y.Z version without a v prefix"
	);
	Ok(version)
}

pub fn current(runner: &Runner) -> Result<Version> {
	let doc = std::fs::read_to_string(runner.root.join("Cargo.toml"))?
		.parse::<toml_edit::DocumentMut>()?;
	stable(
		doc["package"]["version"]
			.as_str()
			.context("Package version missing")?,
	)
}

pub fn clean(runner: &Runner) -> Result<()> {
	ensure!(
		runner.git(&["status", "--porcelain"])?.is_empty(),
		"Commit or stash pending changes first"
	);
	Ok(())
}

pub fn remote(runner: &Runner, name: &str) -> Result<()> {
	let expected = match name {
		"origin" => "gitlab.com/swivel/tux-design",
		"public" => "github.com/swivelgames/tuxd",
		_ => anyhow::bail!("Unknown release remote"),
	};
	for args in [
		vec!["remote", "get-url", "--all", name],
		vec!["remote", "get-url", "--push", "--all", name],
	] {
		let url = runner.git(&args)?;
		ensure!(
			normalize_remote(&url) == expected,
			"{name} must have exactly one fetch/push URL for {expected}"
		);
	}
	Ok(())
}

fn normalize_remote(url: &str) -> String {
	url.trim_start_matches("https://")
		.trim_start_matches("ssh://git@")
		.trim_start_matches("git@")
		.replace(':', "/")
		.trim_end_matches(".git")
		.to_lowercase()
}

pub fn preflight(runner: &Runner, version: &Version) -> Result<()> {
	ensure!(*version > current(runner)?, "Release version must increase");
	ensure!(
		runner.git(&["branch", "--show-current"])? == "main",
		"Release from main after merging"
	);
	clean(runner)?;
	for name in ["origin", "public"] {
		remote(runner, name)?;
		runner.run(runner.command("git").args(["fetch", name]))?;
		runner.run(runner.command("git").args([
			"merge-base",
			"--is-ancestor",
			&format!("{name}/main"),
			"HEAD",
		]))?;
	}
	unused_tag(runner, version)
}

fn unused_tag(runner: &Runner, version: &Version) -> Result<()> {
	let status = runner
		.command("git")
		.args([
			"show-ref",
			"--verify",
			"--quiet",
			&format!("refs/tags/v{version}"),
		])
		.status()?;
	ensure!(
		status.code() == Some(1),
		"Tag exists or its availability could not be checked"
	);
	Ok(())
}

pub fn tagged(runner: &Runner, version: &Version) -> Result<String> {
	clean(runner)?;
	ensure!(current(runner)? == *version, "Manifest version mismatch");
	let commit = runner.git(&["rev-parse", &format!("v{version}^{{commit}}")])?;
	ensure!(
		commit == runner.git(&["rev-parse", "HEAD"])?,
		"Checkout must be at the release tag"
	);
	Ok(commit)
}

pub fn snapshot(runner: &Runner, version: &Version) -> Result<()> {
	unused_tag(runner, version)?;
	ensure!(
		runner.git(&["branch", "--show-current"])? == "main",
		"Snapshot requires main"
	);
	ensure!(current(runner)? == *version, "Manifest version mismatch");
	let changed = runner.git(&["status", "--porcelain", "-z"])?;
	for record in changed.split('\0').filter(|s| !s.is_empty()) {
		ensure!(
			allowed_change(record),
			"Unexpected release change: {record}"
		);
	}
	runner.run(runner.command("git").args(["diff", "--check"]))?;
	runner.run(runner.command("git").args([
		"add",
		"--",
		"Cargo.toml",
		"Cargo.lock",
		"licenses/dependencies.txt",
		"docs/assets",
	]))?;
	runner.run(runner.command("git").args([
		"commit",
		"-m",
		&format!("chore: release tuxd {version}"),
	]))?;
	runner.run(runner.command("git").args([
		"tag",
		"-a",
		&format!("v{version}"),
		"-m",
		&format!("tuxd {version}"),
	]))
}

fn allowed_change(record: &str) -> bool {
	let Some(name) = record.get(3..) else {
		return false;
	};
	!record[..2].contains(['R', 'C'])
		&& (matches!(
			name,
			"Cargo.toml" | "Cargo.lock" | "licenses/dependencies.txt"
		) || name.starts_with("docs/assets/"))
}

pub fn push(runner: &Runner, version: &Version, remotes: &[&str]) -> Result<()> {
	tagged(runner, version)?;
	for name in remotes {
		remote(runner, name)?;
	}
	for name in remotes {
		runner.run(runner.command("git").args([
			"push",
			"--atomic",
			name,
			"HEAD:refs/heads/main",
			&format!("refs/tags/v{version}"),
		]))?;
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn stable_versions_only() {
		assert!(stable("1.2.3").is_ok());
		for input in [
			"v1.2.3",
			"01.2.3",
			"1.2",
			"1.2.3-rc1",
			"1.2.3+meta",
			"1.2.3;echo bad",
		] {
			assert!(stable(input).is_err());
		}
	}
	#[test]
	fn expected_changes_only() {
		assert!(allowed_change(" M Cargo.toml"));
		assert!(allowed_change("?? docs/assets/new.png"));
		for input in [
			" M src/main.rs",
			"R  docs/assets/renamed.png",
			"Cargo.toml",
			"?? secret",
		] {
			assert!(!allowed_change(input));
		}
	}
	#[test]
	fn remote_identity() {
		assert_eq!(
			normalize_remote("git@github.com:Swivelgames/tuxd.git"),
			"github.com/swivelgames/tuxd"
		);
		assert_ne!(
			normalize_remote("https://github.com/other/tuxd.git"),
			"github.com/swivelgames/tuxd"
		);
	}
}
