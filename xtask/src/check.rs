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
	runner.require(&[
		"uv",
		"alacritty",
		"Xvfb",
		"xdotool",
		"tmux",
		"magick",
		"import",
	])
}

pub fn full(runner: &Runner) -> Result<()> {
	prerequisites(runner)?;
	core(runner)?;
	for name in [
		"terminal",
		"characters",
		"clipboard",
		"editor_clipboard",
		"stdin_import",
		"leader",
		"theme",
		"settings_theme",
	] {
		runner.python(&format!("{name}_smoke.py"), &[])?;
	}
	runner.python("alacritty_smoke.py", &[])?;
	for mode in ["--unicode", "--characters", "--symbols"] {
		runner.python("alacritty_smoke.py", &[mode])?;
		runner.python("alacritty_smoke.py", &[mode, "--tmux"])?;
	}
	runner.python("character_width_smoke.py", &[])?;
	runner.python("update_dependency_notices.py", &[])
}

pub fn screenshots(runner: &Runner) -> Result<()> {
	prerequisites(runner)?;
	runner.python("capture_docs.py", &[])
}
