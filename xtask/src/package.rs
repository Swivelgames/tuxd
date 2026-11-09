//! The reviewed Linux-musl binary, corresponding source, and rebuild verification.
use crate::{
	archive, check, git,
	process::{Runner, outside_checkout},
	receipt::{Receipt, write_json},
};
use anyhow::{Result, ensure};
use semver::Version;
use std::{
	collections::BTreeMap,
	fs,
	path::{Path, PathBuf},
	process::Command,
};

pub const TARGET: &str = "x86_64-unknown-linux-musl";
pub const COMPILER: &str = "1.95.0";

pub fn prerequisite(runner: &Runner) -> Result<PathBuf> {
	check::linux()?;
	runner.require(&["git", "cargo", "rustc", "rustup", "strip", "readelf"])?;
	ensure!(
		runner
			.output(runner.command("rustc").arg("--version"))?
			.split_whitespace()
			.nth(1) == Some(COMPILER),
		"Release packaging requires reviewed Rust {COMPILER}"
	);
	ensure!(
		runner
			.output(
				runner
					.command("rustup")
					.args(["target", "list", "--installed"])
			)?
			.lines()
			.any(|line| line == TARGET),
		"Install the {TARGET} target"
	);
	let notice =
		PathBuf::from(runner.output(runner.command("rustc").args(["--print", "sysroot"]))?)
			.join("share/doc/rust/COPYRIGHT-library.html");
	ensure!(
		notice.is_file(),
		"Install rust-docs to retain runtime notices"
	);
	Ok(notice)
}

fn musl(runner: &Runner, operation: &str, source: &Path, target: &Path) -> Command {
	let mut command = runner.command("cargo");
	command
		.current_dir(source)
		.env_remove("CARGO_ENCODED_RUSTFLAGS")
		.env("RUSTFLAGS", "-C target-cpu=x86-64");
	command
		.args([
			operation,
			"--release",
			"--locked",
			"--offline",
			"--target",
			TARGET,
			"--config",
			&format!("target.{TARGET}.linker=\"rust-lld\""),
			"--target-dir",
		])
		.arg(target);
	command
}

pub fn build(runner: &Runner, version: &Version, output: &Path, notes: &Path) -> Result<()> {
	let notice = prerequisite(runner)?;
	let commit = git::tagged(runner, version)?;
	let notes_text = fs::read_to_string(notes)?;
	ensure!(
		!notes_text.trim().is_empty(),
		"Release notes must not be empty"
	);
	let directory = outside_checkout(&runner.root, output)?;
	fs::create_dir_all(&directory)?;
	let artifacts = directory.join("artifacts");
	fs::create_dir(&artifacts)?;
	let source = prepare_source(runner, version, &directory, &notice)?;
	let binary = build_binary(runner, version, &directory, &source)?;
	write_metadata(runner, version, &commit, &source, &binary, &artifacts)?;
	verify_archives(runner, version, &directory, &source, &binary, &artifacts)?;
	write_receipt(version, commit, &directory, &artifacts, notes_text)?;
	println!("Prepared release: {}", directory.display());
	Ok(())
}

fn prepare_source(
	runner: &Runner,
	version: &Version,
	directory: &Path,
	notice: &Path,
) -> Result<PathBuf> {
	let source = directory.join(format!("tuxd-v{version}-source"));
	fs::create_dir(&source)?;
	let source_tar = directory.join("tag.tar");
	runner.run(
		runner
			.command("git")
			.args(["archive", "--output"])
			.arg(&source_tar)
			.arg(format!("v{version}")),
	)?;
	tar::Archive::new(fs::File::open(&source_tar)?).unpack(&source)?;
	fs::remove_file(source_tar)?;

	// Preserve the Cargo alias while adding offline dependency sources.
	let config = source.join(".cargo/config.toml");
	fs::create_dir_all(config.parent().unwrap())?;
	let existing = if config.exists() {
		fs::read_to_string(&config)?
	} else {
		String::new()
	};
	ensure!(
		existing
			.parse::<toml_edit::DocumentMut>()?
			.get("source")
			.is_none(),
		"Review existing Cargo source replacement before packaging"
	);
	let vendor = runner.output(runner.command("cargo").current_dir(&source).args([
		"vendor",
		"--locked",
		"--versioned-dirs",
		"vendor",
	]))?;
	fs::write(config, format!("{existing}\n{vendor}\n"))?;
	fs::copy(
		notice,
		source.join("licenses/runtime/rust-standard-library-COPYRIGHT.html"),
	)?;

	Ok(source)
}

fn build_binary(
	runner: &Runner,
	version: &Version,
	directory: &Path,
	source: &Path,
) -> Result<PathBuf> {
	let build = directory.join("build");
	runner.run(&mut musl(runner, "build", source, &build))?;
	runner.run(musl(runner, "test", source, &build).args(["--", "--include-ignored"]))?;
	let binary_dir = directory.join(format!("tuxd-v{version}-{TARGET}"));
	fs::create_dir(&binary_dir)?;
	let binary = binary_dir.join("tuxd");
	fs::copy(build.join(TARGET).join("release/tuxd"), &binary)?;
	runner.run(runner.command("strip").arg("--strip-all").arg(&binary))?;
	ensure!(
		runner.output(runner.command(&binary).arg("--version"))? == format!("tuxd {version}"),
		"Binary version mismatch"
	);
	ensure!(
		!runner
			.output(runner.command("readelf").arg("-l").arg(&binary))?
			.contains("INTERP"),
		"Binary requires a dynamic loader"
	);
	ensure!(
		!runner
			.output(runner.command("readelf").arg("-d").arg(&binary))?
			.contains("NEEDED"),
		"Binary requires shared libraries"
	);
	for name in ["LICENSE", "THIRD_PARTY_NOTICES.md"] {
		fs::copy(source.join(name), binary_dir.join(name))?;
	}
	for name in ["licenses", "examples"] {
		archive::copy_tree(&source.join(name), &binary_dir.join(name))?;
	}

	Ok(binary_dir)
}

fn write_metadata(
	runner: &Runner,
	version: &Version,
	commit: &str,
	source: &Path,
	binary_dir: &Path,
	artifacts: &Path,
) -> Result<()> {
	let building = include_str!("../templates/BUILDING.txt")
		.replace("$version", &version.to_string())
		.replace("$commit", commit)
		.replace("$target", TARGET)
		.replace("$compiler", COMPILER);
	let info = serde_json::json!({"version": version, "commit": commit, "target": TARGET, "rustc": runner.output(runner.command("rustc").arg("-Vv"))?, "rustflags": "-C target-cpu=x86-64", "linker": "rust-lld"});
	for folder in [source, binary_dir, artifacts] {
		write_json(&folder.join("BUILD-INFO.json"), &info)?;
		fs::write(folder.join("BUILDING.txt"), &building)?;
	}
	fs::write(
		binary_dir.join("README.txt"),
		format!(
			"tuxd {version}: static Linux x86-64 executable.\nRun ./tuxd --version, then ./tuxd in a UTF-8 terminal.\nInstall into a directory on PATH. Retain all license notices.\nSee BUILDING.txt for source and modification instructions.\n"
		),
	)?;

	Ok(())
}

fn verify_archives(
	runner: &Runner,
	version: &Version,
	directory: &Path,
	source: &Path,
	binary_dir: &Path,
	artifacts: &Path,
) -> Result<()> {
	let extracted = directory.join("extracted");
	fs::create_dir(&extracted)?;
	for folder in [source, binary_dir] {
		let bundle = artifacts.join(format!(
			"{}.tar.gz",
			folder.file_name().unwrap().to_string_lossy()
		));
		archive::pack(folder, &bundle)?;
		archive::unpack(&bundle, &extracted)?;
	}
	runner.run(&mut musl(
		runner,
		"build",
		&extracted.join(source.file_name().unwrap()),
		&directory.join("offline-build"),
	))?;
	let extracted_binary = extracted.join(binary_dir.file_name().unwrap()).join("tuxd");
	crate::terminal::packaged(runner, &extracted_binary, &version.to_string())?;

	Ok(())
}

fn write_receipt(
	version: &Version,
	commit: String,
	directory: &Path,
	artifacts: &Path,
	notes_text: String,
) -> Result<()> {
	let mut files = BTreeMap::new();
	for entry in fs::read_dir(artifacts)? {
		let path = entry?.path();
		files.insert(
			format!("artifacts/{}", path.file_name().unwrap().to_string_lossy()),
			archive::digest(&path)?,
		);
	}
	let sums: String = files
		.iter()
		.map(|(name, hash)| format!("{hash}  {}\n", name.strip_prefix("artifacts/").unwrap()))
		.collect();
	fs::write(artifacts.join("SHA256SUMS"), sums)?;
	files.insert(
		"artifacts/SHA256SUMS".into(),
		archive::digest(&artifacts.join("SHA256SUMS"))?,
	);
	fs::write(directory.join("notes.md"), notes_text)?;
	files.insert(
		"notes.md".into(),
		archive::digest(&directory.join("notes.md"))?,
	);
	Receipt {
		version: version.clone(),
		commit,
		files,
	}
	.save(directory)?;
	Ok(())
}
