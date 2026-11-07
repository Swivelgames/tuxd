# /// script
# requires-python = ">=3.10"
# dependencies = ["pyte==0.8.2"]
# ///
"""Clipboard forwarding in isolated PTYs; never touches a desktop clipboard."""
import base64
import json
from pathlib import Path
import re
import shlex
import subprocess
import tempfile
import uuid
import pyte

# tmux probes DEC private device status; pyte 0.8.2 lacks that keyword.
# Ignore this optional probe while preserving ordinary status handling.
_report_status = pyte.Screen.report_device_status
def report_status(self, *args, private=False):
	if not private:
		_report_status(self, *args)
pyte.Screen.report_device_status = report_status
from terminal_smoke import ROOT, Session

subprocess.run(["cargo", "build", "--locked"], cwd=ROOT, check=True)
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version=1"], cwd=ROOT))
binary = str(Path(metadata["target_directory"]) / "debug" / "terminal-ux-designer")
text = "escaped \\x1b[31m 界 " + "long payload " * 800
expected = ("\\x1b[0m" + text.replace("\\", "\\\\") + "\\x1b[0m").encode()

def exercise(session):
	session.paste(expected.decode())
	session.send("\x1b[17~\x1b[17~")
	before = len(session.output)
	session.send("\x03")
	outputs = re.findall(rb"\x1b\]52;[^;]*;([A-Za-z0-9+/=]*)(?:\x07|\x1b\\)", session.output[before:])
	assert outputs, "No OSC 52 forwarding observed"
	assert base64.b64decode(outputs[-1]) == expected
	assert "delivery unconfirmed" in "\n".join(session.rows())

session = Session(binary, env={"SSH_CONNECTION": "test", "DISPLAY": None, "WAYLAND_DISPLAY": None})
try:
	exercise(session)
	session.send("\x11d")
	session.finish()
finally:
	session.close()
print("PASS: direct SSH emits exact >10 KiB escaped ANSI payload")

socket = "tuxd-clipboard-" + uuid.uuid4().hex
with tempfile.TemporaryDirectory(prefix="tuxd-tmux-config-") as directory:
	command = "env XDG_CONFIG_HOME=" + shlex.quote(directory) + " " + shlex.quote(binary)
	tmux = ["tmux", "-L", socket]
	try:
		subprocess.run(tmux + ["-f", "/dev/null", "new-session", "-d", "-s", "test", command], check=True)
		subprocess.run(tmux + ["set-option", "-s", "set-clipboard", "external"], check=True)
		subprocess.run(tmux + ["set-option", "-as", "terminal-features", ",xterm*:clipboard"], check=True)
		session = Session("tmux", "-L", socket, "attach-session", "-t", "test")
		try:
			exercise(session)
			# The app's entire payload also exists in its named tmux buffer.
			names = subprocess.check_output(tmux + ["list-buffers", "-F", "#{buffer_name}"]).decode().splitlines()
			name = next(name for name in names if name.startswith("tuxd-"))
			assert subprocess.check_output(tmux + ["save-buffer", "-b", name, "-"]) == expected
			session.send("\x11d")
			assert session.process.wait(timeout=5) == 0
		finally:
			session.close()
	finally:
		subprocess.run(tmux + ["kill-server"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
print("PASS: application copy forwards through an isolated tmux server with set-clipboard external")

with tempfile.TemporaryDirectory(prefix="tuxd-no-helper-") as directory:
	session = Session(binary, env={"TMUX": "unavailable", "PATH": directory})
	try:
		session.send("\x1b[17~\x1b[17~\x03")
		assert "Copy failed: tmux" in "\n".join(session.rows())
		session.send("\x11")
		session.finish()
	finally:
		session.close()
print("PASS: unavailable tmux reports failure without copying to a different clipboard")
