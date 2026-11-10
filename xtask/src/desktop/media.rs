use super::display::{Display, Window};
use crate::{process::Runner, terminal::fixtures};
use anyhow::{Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::{fs, path::Path};

pub fn capture(runner: &Runner, output: Option<&Path>) -> Result<()> {
	crate::check::prerequisites(runner)?;
	let binary = fixtures::binary(runner, true)?;
	let assets = output
		.map(Path::to_owned)
		.unwrap_or_else(|| runner.root.join("docs/assets"));
	fs::create_dir_all(assets.join("themes"))?;
	let source = fs::read_to_string(runner.root.join("src/libraries/themes/presets.rs"))?;
	let source = source
		.split("pub const PRESETS:")
		.nth(1)
		.unwrap()
		.split("];")
		.next()
		.unwrap();
	let regex = regex::Regex::new(r#"\("([^"]+)", "([^"]+)"\)"#)?;
	let presets: Vec<_> = regex.captures_iter(source).collect();
	ensure!(
		presets.len() == 14,
		"Theme gallery count changed; review capture coverage"
	);
	let temp = tempfile::tempdir()?;
	let folder = temp.path();
	let examples = runner.root.join("examples");
	let config = folder.join("alacritty.toml");
	fs::write(
		&config,
		"[window]\ndimensions = { columns = 200, lines = 60 }\npadding = { x = 12, y = 10 }\n[font]\nsize = 13\nnormal = { family = \"DejaVu Sans Mono\" }\n[cursor]\nstyle = { shape = \"Block\", blinking = \"Never\" }\n",
	)?;
	let display = Display::new(folder, 2560, 1800)?;
	fs::create_dir(folder.join("tuxd"))?;
	for preset in presets {
		let identifier = &preset[1];
		fixtures::write(
			&folder.join("tuxd/theme.json"),
			&serde_json::json!({"version":1,"preset":identifier,"colors":{}}),
		)?;
		let window = Window::open(
			&display,
			&config,
			&examples,
			&[binary.clone().into_os_string()],
			&folder.join("alacritty.log"),
		)?;
		// Jump to the directory heading, then open the first example in the tree.
		window.keys(&["Tab", "Down", "Return"])?;
		// Clear the open notice and capture the normal cursor status.
		window.keys(&["Home"])?;
		window.capture(&assets.join("themes").join(format!("{identifier}.png")))?;
		if identifier == "midnight" {
			demo(runner, &window, folder, &assets)?;
		}
		println!("Captured {}", &preset[2]);
	}
	let encoded = STANDARD.encode(fs::read(assets.join("overview.png"))?);
	let svg = folder.join("social.svg");
	fs::write(
		&svg,
		include_str!("../../templates/social.svg").replace("{{IMAGE}}", &encoded),
	)?;
	runner.run(
		runner
			.command("magick")
			.args(["-background", "none"])
			.arg(svg)
			.arg(assets.join("social-preview.png")),
	)?;
	println!(
		"Updated 16 screenshots, demo GIF and social preview in {}",
		assets.display()
	);
	Ok(())
}

fn demo(runner: &Runner, window: &Window<'_>, folder: &Path, assets: &Path) -> Result<()> {
	window.capture(&assets.join("overview.png"))?;
	let mut frames = Vec::new();
	for (index, keys) in [
		vec![],
		vec!["Down", "Down", "Home", "shift+End"],
		vec!["ctrl+u"],
		vec!["Escape"],
		vec!["F2"],
		vec!["F2"],
		vec!["F1"],
		vec!["Escape"],
	]
	.iter()
	.enumerate()
	{
		if !keys.is_empty() {
			window.keys(keys)?;
		}
		let path = folder.join(format!("frame-{index:02}.png"));
		window.capture(&path)?;
		frames.push(path);
		if index == 6 {
			window.capture(&assets.join("settings.png"))?;
		}
	}
	runner.run(
		runner
			.command("magick")
			.args(["-delay", "140"])
			.args(frames)
			.args(["-loop", "0", "-layers", "Optimize"])
			.arg(assets.join("demo.gif")),
	)
}
