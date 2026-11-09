//! Offline notices and checksum-pinned character catalog generation.
mod catalog;
mod catalog_policy;
mod notices;

use crate::process::Runner;
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Task {
	/// Download pinned Unicode/Nerd Fonts data and regenerate catalog.tsv.
	Catalog,
	/// Regenerate license notices from locked, locally cached Cargo packages.
	Notices,
}

pub fn run(runner: &Runner, task: Task) -> Result<()> {
	match task {
		Task::Catalog => catalog::run(runner),
		Task::Notices => notices::run(runner),
	}
}
