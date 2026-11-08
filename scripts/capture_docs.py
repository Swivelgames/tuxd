#!/usr/bin/env python3
"""Regenerate documentation screenshots and animation in isolated Alacritty."""
import base64
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "docs/assets"


def social_card(overview, output, folder):
	# Render a fresh card from text and the real screenshot, retaining readable UI.
	encoded = base64.b64encode(overview.read_bytes()).decode()
	svg = f'''<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="1280" height="640">
	<rect width="1280" height="640" fill="#161923"/>
	<g font-family="DejaVu Sans">
		<text x="42" y="185" font-size="80" font-weight="bold" fill="#88d2c8">tuxd</text>
		<g fill="#e0e4ee" font-size="32">
			<text x="42" y="238">Design styled</text>
			<text x="42" y="279">terminal text.</text>
		</g>
		<g fill="#a5b1c7" font-size="23">
			<text x="42" y="353">Visual editing.</text>
			<text x="42" y="387">Live ANSI output.</text>
			<text x="42" y="421">Built for your terminal.</text>
		</g>
		<text x="42" y="565" font-size="20" fill="#88d2c8">github.com/swivelgames/tuxd</text>
	</g>
	<image x="470" y="10" width="800" height="620" preserveAspectRatio="xMidYMid meet" xlink:href="data:image/png;base64,{encoded}"/>
</svg>'''
	source = folder / "social.svg"
	source.write_text(svg)
	subprocess.run(["magick", "-background", "none", str(source), str(output)], check=True)


def main():
	subprocess.run(["cargo", "build", "--release", "--locked"], cwd=ROOT, check=True)
	metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version=1"], cwd=ROOT))
	binary = str(Path(metadata["target_directory"]) / "release/tuxd")
	source = (ROOT / "src/libraries/themes/presets.rs").read_text().split("pub const PRESETS:")[1].split("];")[0]
	presets = re.findall(r'\("([^"]+)", "([^"]+)"\)', source)
	assert len(presets) == 14
	with tempfile.TemporaryDirectory(prefix="tuxd-doc-media-") as temporary:
		folder = Path(temporary)
		fixture = folder / "showcase.tuxd.json"
		fixture.write_bytes((ROOT / "examples/showcase.tuxd.json").read_bytes())
		config = folder / "alacritty.toml"
		config.write_text('[window]\ndimensions = { columns = 116, lines = 48 }\npadding = { x = 12, y = 10 }\n[font]\nsize = 13\nnormal = { family = "DejaVu Sans Mono" }\n[cursor]\nstyle = { shape = "Block", blinking = "Never" }\n')
		read_fd, write_fd = os.pipe()
		server = subprocess.Popen(["Xvfb", "-displayfd", str(write_fd), "-screen", "0", "1800x1600x24", "-nolisten", "tcp", "-ac"], pass_fds=[write_fd], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
		os.close(write_fd)
		with os.fdopen(read_fd) as pipe:
			display = ":" + pipe.readline().strip()
		env = {key: value for key, value in os.environ.items() if key not in (
			"WAYLAND_DISPLAY", "TMUX", "TMUX_PANE", "SSH_CONNECTION", "SSH_TTY", "XAUTHORITY",
		)}
		env.update(DISPLAY=display, XDG_CONFIG_HOME=temporary, LIBGL_ALWAYS_SOFTWARE="1", WINIT_UNIX_BACKEND="x11")
		theme_dir = folder / "tuxd"
		theme_dir.mkdir()
		try:
			for identifier, name in presets:
				(theme_dir / "theme.json").write_text(json.dumps({"version": 1, "preset": identifier, "colors": {}}))
				with (folder / "alacritty.log").open("w") as log:
					app = subprocess.Popen(["alacritty", "--config-file", str(config), "-e", binary, str(fixture)], env=env, stdout=log, stderr=log)
					try:
						for _ in range(60):
							found = subprocess.run(["xdotool", "search", "--onlyvisible", "--class", "Alacritty"], env=env, capture_output=True)
							if found.returncode == 0:
								break
							time.sleep(.1)
						else:
							raise RuntimeError("Alacritty did not start")
						window = found.stdout.decode().splitlines()[0]
						subprocess.run(["xdotool", "windowfocus", "--sync", window], env=env, check=True)
						time.sleep(.7)

						def keys(*names):
							subprocess.run(["xdotool", "key", "--clearmodifiers", *names], env=env, check=True)
							time.sleep(.25)

						def capture(path):
							subprocess.run(["import", "-window", window, str(path)], env=env, check=True)

						capture(ASSETS / "themes" / f"{identifier}.png")
						if identifier == "midnight":
							capture(ASSETS / "overview.png")
							frames = []

							def frame():
								path = folder / f"frame-{len(frames):02}.png"
								capture(path)
								frames.append(str(path))

							frame()
							for sequence in [("Down", "Down", "Home", "shift+End"), ("ctrl+u",), ("Escape",), ("F2",), ("F2",), ("F1",)]:
								keys(*sequence)
								frame()
							capture(ASSETS / "settings.png")
							keys("Escape")
							frame()
							subprocess.run(["magick", "-delay", "140", *frames, "-loop", "0", "-layers", "Optimize", str(ASSETS / "demo.gif")], check=True)
						print(f"Captured {name}", flush=True)
					finally:
						app.terminate()
						app.wait(timeout=5)
		finally:
			server.terminate()
			server.wait(timeout=5)
		social_card(ASSETS / "overview.png", ASSETS / "social-preview.png", folder)
	print("Updated 16 screenshots, demo GIF, and social preview.")


if __name__ == "__main__":
	main()
