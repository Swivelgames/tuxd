# /// script
# requires-python = ">=3.10"
# dependencies = ["pyte==0.8.2"]
# ///
"""CLI ANSI import and controlling-terminal handoff acceptance checks."""
import json
import os
import pty
from pathlib import Path
import subprocess
import tempfile
from terminal_smoke import ROOT, Session

subprocess.run(["cargo", "build", "--locked"], cwd=ROOT, check=True)
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version=1"], cwd=ROOT))
binary = str(Path(metadata["target_directory"]) / "debug" / "tuxd")
with tempfile.TemporaryDirectory(prefix="tuxd-stdin-") as directory:
	folder = Path(directory)
	for index, (mode, data, review, accept, file_source) in enumerate([
		("--raw", b"\x1b[1;31mImported\x1b[0m", False, True, False),
		("--escaped", br"\x1b[1;31mImported\x1b[0m", False, True, False),
		("--raw", b"Imported\x1b[2J", True, True, False),
		("--escaped", br"Imported\x1b[2J", True, False, False),
		("--raw", b"", False, True, False),
		("--raw", b"Imported", False, True, True),
		("--escaped", br"Imported\x1b[2J", True, True, True),
	]):
		source = folder / f"source-{index}.txt"
		source.write_bytes(data)
		read_fd, write_fd = os.pipe()
		os.write(write_fd, data)
		os.close(write_fd)
		try:
			session = Session(binary, mode, str(source) if file_source else "-", stdin_source=None if file_source else read_fd)
		finally:
			os.close(read_fd)
		try:
			if review:
				assert "Unsupported ANSI" in "\n".join(session.rows())
				session.send("s" if accept else "\x1b")
			# Keyboard input after EOF must still work; save verifies imported text/style.
			session.send("\x13")
			assert "Save document as" in "\n".join(session.rows())
			destination = folder / f"saved-{index}.tuxd.json"
			session.paste(str(destination))
			session.send("\r")
			result = json.loads(destination.read_text())
			assert result["text"] == ("Imported" if data and accept else ""), result
			if index < 2:
				assert result["styles"]["runs"], result
			assert source.read_bytes() == data
			session.send("\x11")
			session.finish()
		finally:
			session.close()
	for payload, message in [(b"\xff", "UTF-8"), (b"x" * (16 * 1024 * 1024 + 1), "16 MiB")]:
		with tempfile.TemporaryFile() as source:
			source.write(payload)
			source.seek(0)
			session = Session(binary, "--raw", "-", stdin_source=source)
			try:
				assert session.process.wait(timeout=5) != 0
				assert message.encode() in session.output
				assert b"\x1b[?1049h" not in session.output
			finally:
				session.close()
	session = Session(binary, "--raw", "-")
	try:
		assert session.process.wait(timeout=5) != 0
		assert b"Expected piped or redirected stdin" in session.output
	finally:
		session.close()
	master, slave = pty.openpty()
	try:
		result = subprocess.run([binary, "--raw", "-"], input=b"text", stdout=slave, stderr=subprocess.PIPE, start_new_session=True)
		assert result.returncode != 0
		assert b"controlling terminal" in result.stderr
	finally:
		os.close(master)
		os.close(slave)
	for args in [["-"], ["--raw"], ["--raw", "--escaped"], ["--raw", "-"]]:
		result = subprocess.run([binary, *args], input=b"text", capture_output=True)
		assert result.returncode != 0
		assert b"\x1b[?1049h" not in result.stdout
print("PASS: raw/escaped stdin and files, keyboard handoff, review accept/abort, save, limits and terminal cleanup")
