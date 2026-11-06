#!/usr/bin/env python3
"""Real Alacritty rendering/input on isolated Xvfb; does not touch the desktop."""
import json
import os
from pathlib import Path
import subprocess
import sys
import uuid
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
subprocess.run(["cargo", "build", "--release", "--locked"], cwd=ROOT, check=True)
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version=1"], cwd=ROOT))
binary = str(Path(metadata["target_directory"]) / "release" / "terminal-ux-designer")
artifacts = Path(tempfile.mkdtemp(prefix="tuxd-alacritty-evidence-"))
with tempfile.TemporaryDirectory(prefix="tuxd-alacritty-config-") as directory:
	folder = Path(directory)
	config = folder / "alacritty.toml"
	config.write_text('[window]\ndimensions = { columns = 110, lines = 36 }\n[font]\nsize = 12\n[cursor]\nstyle = { shape = "Block", blinking = "Never" }\n')
	document = folder / "check.tuxd.json"
	document.write_bytes((ROOT / "examples/welcome.tuxd.json").read_bytes())
	original = json.loads(document.read_text())
	if "--unicode" in sys.argv:
		original["text"] = "café é 界 👩🏽‍💻 🇺🇳"
		original["styles"]["runs"] = [{"start": 0, "end": len(original["text"].encode()), "style": {**original["styles"]["runs"][0]["style"], "bold": False}}]
		document.write_text(json.dumps(original))
	read_fd, write_fd = os.pipe()
	xvfb = subprocess.Popen(["Xvfb", "-displayfd", str(write_fd), "-screen", "0", "1400x1000x24", "-nolisten", "tcp", "-ac"], pass_fds=[write_fd], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
	os.close(write_fd)
	with os.fdopen(read_fd) as pipe:
		display = ":" + pipe.readline().strip()
	env = {k:v for k,v in os.environ.items() if k not in ["WAYLAND_DISPLAY", "TMUX", "TMUX_PANE", "SSH_CONNECTION", "SSH_TTY", "XAUTHORITY"]}
	env.update(DISPLAY=display, XDG_CONFIG_HOME=directory, LIBGL_ALWAYS_SOFTWARE="1", WINIT_UNIX_BACKEND="x11")
	log = (artifacts / "alacritty.log").open("w")
	tmux = ["tmux", "-L", "tuxd-render-" + uuid.uuid4().hex]
	command = [binary, str(document)]
	if "--tmux" in sys.argv:
		command = tmux + ["-f", "/dev/null", "new-session", "-s", "test", *command]
	app = subprocess.Popen(["alacritty", "--config-file", str(config), "--title", "tuxd-isolated-check", "-e", *command], env=env, stdout=log, stderr=log)
	def xdo(*args):
		return subprocess.check_output(["xdotool", *args], env=env).decode().strip()
	try:
		window = None
		for _ in range(50):
			result = subprocess.run(["xdotool", "search", "--onlyvisible", "--class", "Alacritty"], env=env, capture_output=True)
			if result.returncode == 0:
				window = result.stdout.decode().splitlines()[0]
				break
			time.sleep(.1)
		assert window, "Alacritty window did not start; see log"
		time.sleep(.5)
		xdo("windowfocus", "--sync", window)
		if "--tmux" in sys.argv:
			subprocess.run(tmux + ["set-option", "prefix", "C-a"], check=True)
			subprocess.run(tmux + ["set-option", "-s", "escape-time", "10"], check=True)
		def keys(*names):
			xdo("key", "--clearmodifiers", *names)
			time.sleep(.15)
		def capture(name):
			subprocess.run(["import", "-window", window, str(artifacts / name)], env=env, check=True)
		capture("initial.png")
		if "--unicode" in sys.argv:
			keys("ctrl+End", "shift+Home")
			capture("unicode-selected.png")
			keys("Home", *(["Return"] * 10))
			capture("unicode-newlines.png")
			keys(*(["BackSpace"] * 10))
			capture("unicode-restored.png")
			# Compare the full body, excluding the title's dirty marker and status rows.
			w, h = map(int, subprocess.check_output(["identify", "-format", "%w %h", str(artifacts / "initial.png")]).split())
			cell_height = h // 36
			crop = f"[{w}x{h - 4 * cell_height}+0+{2 * cell_height}]"
			comparison = subprocess.run(["magick", "compare", "-metric", "AE", str(artifacts / "initial.png") + crop, str(artifacts / "unicode-restored.png") + crop, "null:"], capture_output=True)
			assert comparison.returncode == 0, "Stale pixels after selection/newlines: " + comparison.stderr.decode()
		keys("ctrl+End")
		xdo("type", "--clearmodifiers", "--delay", "10", " GUI verified")
		keys(*(["shift+Left"] * 13), "ctrl+b", "ctrl+s")
		saved = json.loads(document.read_text())
		assert saved["text"] == original["text"] + " GUI verified", repr(saved["text"])
		assert saved["styles"]["runs"][-1]["style"]["bold"]
		capture("edited.png")
		keys("F1")
		capture("settings.png")
		keys("Escape")
		time.sleep(.7)
		for width, height in [(430,360),(280,220),(1100,740)]:
			xdo("windowsize", "--sync", window, str(width), str(height))
			time.sleep(.2)
			keys("F2", "F2", "F5", "F5")
			assert app.poll() is None
			capture(f"resize-{width}.png")
		capture("before-exit.png")
		keys("ctrl+q")
		assert app.wait(timeout=5) == 0
		assert json.loads(document.read_text()) == saved
		print(f"PASS: Alacritty real key input, selection styling, save, settings, resize/slim/collapse, clean exit; screenshots: {artifacts}")
	finally:
		if app.poll() is None:
			app.terminate()
			app.wait(timeout=5)
		if "--tmux" in sys.argv:
			subprocess.run(tmux + ["kill-server"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
		xvfb.terminate()
		xvfb.wait(timeout=5)
		log.close()
