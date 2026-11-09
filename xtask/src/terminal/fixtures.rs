//! Test document fixtures and application builds.
use crate::process::Runner;
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
	fs,
	path::{Path, PathBuf},
};

pub fn read(path: &Path) -> Result<Value> {
	Ok(serde_json::from_slice(&fs::read(path)?)?)
}

pub fn write(path: &Path, value: &Value) -> Result<()> {
	fs::write(path, serde_json::to_vec(value)?)?;
	Ok(())
}

pub fn empty(path: &Path) -> Result<()> {
	write(path, &json!({"version":1,"text":"","styles":{"runs":[]}}))
}

pub fn text(path: &Path, expected: &str) -> Result<()> {
	ensure!(
		read(path)?["text"] == expected,
		"Unexpected document text at {}",
		path.display()
	);
	Ok(())
}

pub fn binary(runner: &Runner, release: bool) -> Result<PathBuf> {
	let mut build = runner.command("cargo");
	build.args(["build", "--locked"]);
	if release {
		build.arg("--release");
	}
	runner.run(&mut build)?;
	let metadata: Value = serde_json::from_str(&runner.output(runner.command("cargo").args([
		"metadata",
		"--no-deps",
		"--format-version=1",
	]))?)?;
	Ok(PathBuf::from(
		metadata["target_directory"]
			.as_str()
			.context("Missing target directory")?,
	)
	.join(if release { "release" } else { "debug" })
	.join(format!("tuxd{}", std::env::consts::EXE_SUFFIX)))
}
