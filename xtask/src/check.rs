//! Portable Rust checks and explicitly Linux-specific acceptance/media tasks.
use crate::process::Runner;
use anyhow::{Result, ensure};

pub fn linux() -> Result<()> {
	ensure!(
		std::env::consts::OS == "linux" && std::env::consts::ARCH == "x86_64",
		"This step requires a Linux x86-64 runner (Alacritty/Xvfb, PTYs and musl); it has not been skipped"
	);
	Ok(())
}

pub fn core(runner: &Runner) -> Result<()> {
	for manifest in ["Cargo.toml", "xtask/Cargo.toml"] {
		runner.cargo(&["fmt", "--manifest-path", manifest, "--check"])?;
		runner.cargo(&["test", "--manifest-path", manifest, "--locked"])?;
		runner.cargo(&[
			"clippy",
			"--manifest-path",
			manifest,
			"--locked",
			"--all-targets",
			"--",
			"-D",
			"warnings",
		])?;
	}
	runner.cargo(&["test", "--release", "--locked", "--", "--include-ignored"])
}

pub fn prerequisites(runner: &Runner) -> Result<()> {
	linux()?;
	runner.require(&["alacritty", "Xvfb", "xdotool", "tmux", "magick", "import"])
}

pub fn full(runner: &Runner) -> Result<()> {
	prerequisites(runner)?;
	core(runner)?;
	crate::terminal::run(runner, crate::terminal::Suite::All)?;
	crate::desktop::render(runner, crate::desktop::Mode::Basic, false)?;
	for mode in [
		crate::desktop::Mode::Workspace,
		crate::desktop::Mode::Unicode,
		crate::desktop::Mode::Characters,
		crate::desktop::Mode::Symbols,
	] {
		crate::desktop::render(runner, mode, false)?;
		crate::desktop::render(runner, mode, true)?;
	}
	crate::desktop::check(runner, false)?;
	crate::generate::run(runner, crate::generate::Task::Notices)
}

pub fn screenshots(runner: &Runner) -> Result<()> {
	crate::desktop::capture(runner, None)
}
