# /// script
# requires-python = ">=3.10"
# dependencies = ["pyte==0.8.2"]
# ///
"""Editor clipboard workflows using isolated helpers, never the desktop clipboard."""
import json
from pathlib import Path
import shlex
import subprocess
import tempfile
from terminal_smoke import ROOT, Session

subprocess.run(["cargo", "build", "--locked"], cwd=ROOT, check=True)
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version=1"], cwd=ROOT))
binary = str(Path(metadata["target_directory"]) / "debug" / "tuxd")
with tempfile.TemporaryDirectory(prefix="tuxd-editor-clipboard-") as directory:
	folder = Path(directory)
	clipboard = folder / "clipboard.txt"
	clipboard.write_text("")
	helper = folder / "xclip"
	helper.write_text("#!/bin/sh\ncase \"$*\" in\n*-out*) /bin/cat " + shlex.quote(str(clipboard)) + ";;\n*) /bin/cat > " + shlex.quote(str(clipboard)) + ";;\nesac\n")
	helper.chmod(0o700)
	document = folder / "test.tuxd.json"
	document.write_text(json.dumps({"version": 1, "text": "", "styles": {"runs": []}}))
	session = Session(binary, str(document), env={"PATH": directory, "DISPLAY": ":test", "WAYLAND_DISPLAY": None})
	try:
		session.paste("café\n界é")
		session.send("\x01\x02\x03")  # select all, bold, copy
		copied = clipboard.read_text()
		assert "\\x1b[0;1m" in copied and "café\\n界é" in copied
		session.send("\x18")
		session.send("\x13")  # cut, save
		assert json.loads(document.read_text())["text"] == ""
		session.send("\x1a\x13")  # undo, save
		original = json.loads(document.read_text())
		assert original["text"] == "café\n界é"
		session.send("\x01\x16")
		session.send("\x13")  # replace selection by clipboard, save
		assert json.loads(document.read_text()) == original
		clipboard.write_text(r"\x1b[31mnew\x1b[2J")
		session.send("\x01\x16")
		assert "Unsupported ANSI" in "\n".join(session.rows())
		assert "Strip and paste" in "\n".join(session.rows())
		session.send("\x1b")  # abort preserves original selection/document
		session.send("\x13")
		assert json.loads(document.read_text()) == original
		session.send("\x16")
		session.send("s")  # positive review action
		session.send("\x13")
		assert json.loads(document.read_text())["text"] == "new"
		session.send("\x11")
		session.finish()
	finally:
		session.close()
print("PASS: editor styled copy/cut/paste, atomic undo, review abort/accept, exact persistence and cleanup")
