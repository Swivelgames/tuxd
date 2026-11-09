//! Release ordering; capabilities own their implementation and invariants.
use crate::{
	check, git, package,
	process::{Runner, outside_checkout},
	publish::{self, Destinations},
	version,
};
use anyhow::{Context, Result, ensure};
use semver::Version;
use std::{fs, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
	Preflight,
	Version,
	Validate,
	Screenshots,
	PackageCheck,
	Snapshot,
	Artifacts,
	Publish,
}

fn plan(destinations: &Destinations) -> Vec<Step> {
	let mut steps = vec![
		Step::Preflight,
		Step::Version,
		Step::Validate,
		Step::Screenshots,
		Step::PackageCheck,
		Step::Snapshot,
		Step::Artifacts,
	];
	if destinations.any() {
		steps.push(Step::Publish);
	}
	steps
}

fn perform(steps: &[Step], mut action: impl FnMut(Step) -> Result<()>) -> Result<()> {
	for &step in steps {
		action(step)
			.with_context(|| format!("Release step {step:?} failed; no rollback was attempted"))?;
	}
	Ok(())
}

pub fn run(
	runner: &mut Runner,
	version: &Version,
	notes: Option<&Path>,
	output: Option<&Path>,
	destinations: &Destinations,
	execute: bool,
) -> Result<()> {
	let steps = plan(destinations);
	println!("Release {version}: {steps:?}");
	println!(
		"Publication: {destinations:?}; push {:?}",
		destinations.remotes()
	);
	if !execute {
		println!(
			"Preview only. Add --execute and --notes FILE to run. Full preparation requires Linux x86-64."
		);
		return Ok(());
	}
	check::prerequisites(runner)?;
	package::prerequisite(runner)?;
	publish::prerequisites(runner, destinations)?;
	version::prerequisite(runner)?;
	let notes = notes
		.context("--execute requires --notes FILE")?
		.canonicalize()?;
	ensure!(
		!fs::read_to_string(&notes)?.trim().is_empty(),
		"Release notes must not be empty"
	);
	let directory = match output {
		Some(path) => {
			let path = outside_checkout(&runner.root, path)?;
			ensure!(!path.exists(), "Choose a new release output directory");
			fs::create_dir_all(&path)?;
			path
		}
		None => tempfile::Builder::new()
			.prefix(&format!("tuxd-v{version}-release-"))
			.tempdir()?
			.keep(),
	};
	runner.log = Some(directory.join("release.log"));
	println!("Retained output and command log: {}", directory.display());
	perform(&steps, |step| match step {
		Step::Preflight => git::preflight(runner, version),
		Step::Version => version::bump(runner, version, true),
		Step::Validate => check::full(runner),
		Step::Screenshots => check::screenshots(runner),
		Step::PackageCheck => runner.cargo(&["publish", "--dry-run", "--locked", "--allow-dirty"]),
		Step::Snapshot => git::snapshot(runner, version),
		Step::Artifacts => package::build(runner, version, &directory, &notes),
		Step::Publish => publish::run(runner, &directory, destinations, true),
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn local_plan_and_ordering() {
		let local = plan(&Destinations::default());
		assert!(!local.contains(&Step::Publish));
		assert!(
			local.iter().position(|s| *s == Step::Screenshots)
				< local.iter().position(|s| *s == Step::Snapshot)
		);
		assert_eq!(
			plan(&Destinations {
				all: true,
				..Default::default()
			})
			.last(),
			Some(&Step::Publish)
		);
	}
	#[test]
	fn failure_stops_before_publication() {
		let mut seen = Vec::new();
		let result = perform(
			&plan(&Destinations {
				all: true,
				..Default::default()
			}),
			|step| {
				seen.push(step);
				ensure!(step != Step::Validate, "test failure");
				Ok(())
			},
		);
		assert!(result.is_err());
		assert_eq!(seen, [Step::Preflight, Step::Version, Step::Validate]);
	}
}
