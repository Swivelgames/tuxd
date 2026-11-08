# /// script
# dependencies = ["pyte==0.8.2"]
# ///
"""Unified Settings preview, discard, save and restart acceptance."""
import json
import subprocess
import tempfile
from pathlib import Path
from terminal_smoke import ROOT, Session
subprocess.run(["cargo", "build", "--locked"], cwd=ROOT, check=True)
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version=1"], cwd=ROOT))
binary = str(Path(metadata["target_directory"]) / "debug" / "tuxd")
with tempfile.TemporaryDirectory() as directory:
	path = Path(directory) / "tuxd" / "theme.json"
	session = Session(binary, env={"XDG_CONFIG_HOME": directory})
	try:
		baseline_color = session.screen.buffer[0][1].fg
		def appearance():
			session.send("\x1bOP\x0cggj\r")
			assert "Appearance" in "\n".join(session.rows())
			session.send("\x1b[B")  # preset to color roles
		appearance()
		session.send("\r")
		session.paste("#123456")
		assert not path.exists()
		session.send("\r\x1b")
		assert "Discard unsaved settings" in "\n".join(session.rows())
		session.send("\x1b[D\r")
		assert not path.exists()
		assert session.screen.buffer[0][1].fg == baseline_color
		appearance()
		session.send("\r")
		session.paste("#123456")
		session.send("\r\x13")
		assert json.loads(path.read_text())["colors"]["accent"] == "#123456"
		assert session.screen.buffer[0][1].fg == "123456"
		shortcuts_path = path.with_name("shortcuts.json")
		shortcuts = shortcuts_path.read_bytes()
		session.send("\x1bOP\x0cggj\r\x1b[C\x13")
		assert json.loads(path.read_text())["preset"] == "light"
		assert json.loads(path.read_text())["colors"]["accent"] == "#123456"
		session.send("\x1bOP\x0cggj\r\t\r\x13")
		assert json.loads(path.read_text())["preset"] == "light"
		assert json.loads(path.read_text())["colors"] == {}
		assert shortcuts_path.read_bytes() == shortcuts
		assert session.screen.buffer[0][1].fg == "005a80"
		session.send("\x1bOP\x0cggj\r\x1b[C")
		session.send("\x1b")
		session.send("\x1b[D\r")
		assert session.screen.buffer[0][1].fg == "005a80"
		session.send("\x1bOP\x0cggj\r\rG")
		assert "Catppuccin Mocha" in "\n".join(session.rows())
		session.send("\r\x13")
		assert json.loads(path.read_text())["preset"] == "catppuccin-mocha"
		assert session.screen.buffer[0][1].fg == "89b4fa"
		session.send("\x11")
		session.finish()
	finally:
		session.close()
	session = Session(binary, env={"XDG_CONFIG_HOME": directory})
	try:
		assert session.screen.buffer[0][1].fg == "89b4fa"
		session.send("\x11")
		session.finish()
	finally:
		session.close()
print("PASS: Settings theme preview, unsaved discard, save and restart")
