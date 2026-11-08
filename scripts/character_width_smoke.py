#!/usr/bin/env python3
"""Compare tuxd display widths to real cursor reports in isolated Alacritty/tmux.

Use --raw to investigate unprotected source characters instead of display fallback.
This measures cell advance, not font coverage or glyph ink extending outside cells.
"""
import json
import os
from pathlib import Path
import re
import select
import subprocess
import sys
import tempfile
import termios
import time
import tty
import uuid

ROOT = Path(__file__).resolve().parents[1]
CURSOR = re.compile(rb"\x1b\[(\d+);(\d+)R")


def probe(samples_path, output_path):
	fd = sys.stdin.fileno()
	before = termios.tcgetattr(fd)
	failures = []
	count = 0
	tty.setraw(fd)
	try:
		for line in Path(samples_path).read_text().splitlines():
			sample = json.loads(line)
			os.write(1, ("\x1b[1;1H\x1b[2K" + sample["display"] + "\x1b[6n").encode())
			reply = b""
			deadline = time.monotonic() + 2
			while not CURSOR.search(reply) and time.monotonic() < deadline:
				if select.select([fd], [], [], .1)[0]:
					reply += os.read(fd, 1024)
			match = CURSOR.search(reply)
			if not match:
				raise RuntimeError("Terminal did not answer cursor-position query")
			actual = int(match[2]) - 1
			if actual != sample["width"] or int(match[1]) != 1:
				failures.append({**sample, "actual": actual})
			count += 1
	finally:
		termios.tcsetattr(fd, termios.TCSANOW, before)
	Path(output_path).write_text(json.dumps({"count": count, "failures": failures}, indent=4))


def main():
	command = ["cargo", "run", "--quiet", "--locked", "--example", "character_widths"]
	if "--raw" in sys.argv:
		command += ["--", "--raw"]
	samples = subprocess.check_output(command, cwd=ROOT)
	with tempfile.TemporaryDirectory(prefix="tuxd-width-check-") as directory:
		folder = Path(directory)
		inputs = folder / "samples.jsonl"
		inputs.write_bytes(samples)
		config = folder / "alacritty.toml"
		config.write_text('[window]\ndimensions = { columns = 100, lines = 30 }\n')
		read_fd, write_fd = os.pipe()
		xvfb = subprocess.Popen(
			["Xvfb", "-displayfd", str(write_fd), "-screen", "0", "1400x1000x24", "-nolisten", "tcp", "-ac"],
			pass_fds=[write_fd], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
		)
		os.close(write_fd)
		with os.fdopen(read_fd) as pipe:
			display = ":" + pipe.readline().strip()
		env = {key: value for key, value in os.environ.items() if key not in (
			"WAYLAND_DISPLAY", "TMUX", "TMUX_PANE", "SSH_CONNECTION", "SSH_TTY", "XAUTHORITY",
		)}
		env.update(DISPLAY=display, XDG_CONFIG_HOME=directory, LIBGL_ALWAYS_SOFTWARE="1", WINIT_UNIX_BACKEND="x11")
		tmux = ["tmux", "-L", "tuxd-width-" + uuid.uuid4().hex]
		failed = False
		try:
			for mode in ["bare", "tmux"]:
				output = folder / (mode + ".json")
				child = [sys.executable, str(Path(__file__).resolve()), "--probe", str(inputs), str(output)]
				if mode == "tmux":
					child = tmux + ["-f", "/dev/null", "new-session", *child]
				subprocess.run(
					["alacritty", "--config-file", str(config), "-e", *child],
					env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=180, check=True,
				)
				result = json.loads(output.read_text())
				failures = result["failures"]
				print(f"{mode}: {result['count']} samples, {len(failures)} width mismatches", flush=True)
				if failures:
					failed = True
					print(json.dumps(failures, indent=4), flush=True)
		finally:
			subprocess.run(tmux + ["kill-server"], capture_output=True)
			xvfb.terminate()
			xvfb.wait()
		if failed:
			raise SystemExit(1)


if __name__ == "__main__":
	if len(sys.argv) > 1 and sys.argv[1] == "--probe":
		probe(sys.argv[2], sys.argv[3])
	else:
		main()
