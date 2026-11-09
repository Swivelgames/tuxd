//! Project-local release tooling. The application has no dependency on this crate.
mod archive;
mod check;
mod git;
mod package;
mod process;
mod publish;
mod receipt;
mod release;
mod version;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use process::Runner;
use publish::Destinations;
use semver::Version;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
	name = "cargo xtask",
	bin_name = "cargo xtask",
	about = "tuxd validation and opt-in release automation"
)]
struct Cli {
	#[command(subcommand)]
	task: Task,
}

#[derive(Subcommand)]
enum Task {
	/// Rust checks on this host; --full adds required Linux terminal acceptance tests.
	Check {
		#[arg(long)]
		full: bool,
	},
	/// Regenerate documentation media on a Linux x86-64 runner.
	Screenshots,
	/// Check clean main, upstream ancestry, version, and tag availability (fetches remotes).
	Preflight {
		#[arg(value_parser = git::stable)]
		version: Version,
	},
	/// Update version through cargo-release. Preview unless --execute is supplied.
	Version {
		#[arg(value_parser = git::stable)]
		version: Version,
		#[arg(long)]
		execute: bool,
	},
	/// Commit expected generated files and annotate the version tag.
	Snapshot {
		#[arg(value_parser = git::stable)]
		version: Version,
		#[arg(long)]
		execute: bool,
	},
	/// Build and verify the tagged Linux-musl archives, source, and checksums.
	Package {
		#[arg(value_parser = git::stable)]
		version: Version,
		#[arg(long)]
		notes: PathBuf,
		#[arg(long)]
		output: PathBuf,
	},
	/// Publish a prepared release. Requires destination flags and --execute.
	Publish {
		directory: PathBuf,
		#[command(flatten)]
		destinations: Destinations,
		#[arg(long)]
		execute: bool,
	},
	/// Coordinate all release steps. Preview unless --execute is supplied.
	Release {
		#[arg(value_parser = git::stable)]
		version: Version,
		#[arg(long)]
		notes: Option<PathBuf>,
		#[arg(long)]
		output: Option<PathBuf>,
		#[command(flatten)]
		destinations: Destinations,
		#[arg(long)]
		execute: bool,
	},
}

fn main() -> Result<()> {
	let cli = Cli::parse();
	let cwd = std::env::current_dir()?;
	let root = cwd
		.ancestors()
		.find(|path| path.join("xtask/Cargo.toml").is_file() && path.join("Cargo.toml").is_file())
		.context("Run inside the tuxd checkout")?
		.to_owned();
	let mut runner = Runner::new(root);
	match cli.task {
		Task::Check { full } => {
			if full {
				check::full(&runner)
			} else {
				check::core(&runner)
			}
		}
		Task::Screenshots => check::screenshots(&runner),
		Task::Preflight { version } => git::preflight(&runner, &version),
		Task::Version { version, execute } => version::bump(&runner, &version, execute),
		Task::Snapshot { version, execute } => {
			if execute {
				git::snapshot(&runner, &version)
			} else {
				println!(
					"Would commit expected release files and tag v{version}. Add --execute to run."
				);
				Ok(())
			}
		}
		Task::Package {
			version,
			notes,
			output,
		} => package::build(&runner, &version, &output, &notes),
		Task::Publish {
			directory,
			destinations,
			execute,
		} => publish::run(&runner, &directory, &destinations, execute),
		Task::Release {
			version,
			notes,
			output,
			destinations,
			execute,
		} => release::run(
			&mut runner,
			&version,
			notes.as_deref(),
			output.as_deref(),
			&destinations,
			execute,
		),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn release_defaults_to_preview() {
		let parsed = Cli::try_parse_from(["xtask", "release", "0.3.0", "--all"]).unwrap();
		assert!(matches!(parsed.task, Task::Release { execute: false, .. }));
		assert!(Cli::try_parse_from(["xtask", "release", "0.3.0;evil"]).is_err());
	}
}
