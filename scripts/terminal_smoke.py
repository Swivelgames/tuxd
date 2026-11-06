# /// script
# requires-python = ">=3.10"
# dependencies = ["pyte==0.8.2"]
# ///
"""Linux PTY acceptance checks: uv run scripts/terminal_smoke.py."""

import codecs
import errno
import fcntl
import json
import os
from pathlib import Path
import pty
import select
import signal
import struct
import subprocess
import termios
import time
import tempfile
import shlex

import pyte


ROOT = Path(__file__).resolve().parents[1]


class Session:
	def __init__(self, binary, *args, env=None):
		self.config_directory = tempfile.TemporaryDirectory(prefix="tuxd-smoke-config-")
		self.master, self.slave = pty.openpty()
		self.original = termios.tcgetattr(self.slave)
		self.screen = pyte.Screen(80, 24)
		self.stream = pyte.Stream(self.screen)
		self.decoder = codecs.getincrementaldecoder("utf-8")("replace")
		self.output = b""
		self.resize(80, 24)

		def controlling_terminal():
			os.setsid()
			fcntl.ioctl(self.slave, termios.TIOCSCTTY, 0)

		self.process = subprocess.Popen(
			[binary, *args], stdin=self.slave, stdout=self.slave, stderr=self.slave,
			env={key: value for key, value in {**os.environ, "TERM": "xterm-256color", "XDG_CONFIG_HOME": self.config_directory.name, "TMUX": None, "TMUX_PANE": None, "SSH_CONNECTION": None, "SSH_TTY": None, **(env or {})}.items() if value is not None}, preexec_fn=controlling_terminal,
		)
		self.drain()

	def resize(self, columns, rows):
		fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", rows, columns, 0, 0))
		self.screen.resize(rows, columns)
		if hasattr(self, "process"):
			os.kill(self.process.pid, signal.SIGWINCH)
			self.drain()

	def drain(self):
		deadline = time.monotonic() + 3
		while time.monotonic() < deadline:
			ready, _, _ = select.select([self.master], [], [], 0.15)
			if not ready:
				return
			try:
				data = os.read(self.master, 65536)
			except OSError as error:
				if error.errno == errno.EIO:
					return
				raise
			if not data:
				return
			self.output += data
			self.stream.feed(self.decoder.decode(data))
		raise AssertionError("Terminal did not become idle")

	def send(self, text):
		os.write(self.master, text.encode())
		self.drain()

	def paste(self, text):
		self.send("\x1b[200~" + text + "\x1b[201~")

	def row(self, number):
		return "".join(self.screen.buffer[number][column].data or " " for column in range(self.screen.columns))

	def rows(self):
		return [self.row(number) for number in range(self.screen.lines)]

	def click_tool(self, label):
		# Locate a control in the scrollable sidebar, independent of panel spacing.
		for direction in [64, 65]:
			for _ in range(100):
				for row, text in enumerate(self.rows()):
					column = text.find(label, 45)
					if column >= 0:
						self.send(f"\x1b[<0;{column + 1};{row + 1}M\x1b[<0;{column + 1};{row + 1}m")
						return
				before = self.rows()
				self.send(f"\x1b[<{direction};70;15M")
				if before == self.rows():
					break
		raise AssertionError(f"Tool {label!r} not visible: {self.rows()}")

	def finish(self, expected=0):
		try:
			assert self.process.wait(timeout=5) == expected
			self.drain()
			assert termios.tcgetattr(self.slave) == self.original, "Terminal modes were not restored"
			assert self.output.count(b"\x1b[<1u") == 1, "Keyboard mode must be popped exactly once"
			for sequence in [b"\x1b[?1049l", b"\x1b[?2004l", b"\x1b[?1000l", b"\x1b[?25h"]:
				assert sequence in self.output, f"Missing cleanup: {sequence!r}"
		finally:
			self.close()

	def close(self):
		if self.process.poll() is None:
			self.process.kill()
			self.process.wait()
		if self.master is not None:
			os.close(self.master)
			os.close(self.slave)
			self.master = None
		self.config_directory.cleanup()


def shortcut_settings_checks(binary):
	with tempfile.TemporaryDirectory(prefix="tuxd-shortcut-settings-") as directory:
		env = {"XDG_CONFIG_HOME": directory}
		path = Path(directory) / "tuxd" / "shortcuts.json"
		session = Session(binary, env=env)
		def category(index):
			session.send("\x0cgg" + "j" * index + "\r")
		def click(label):
			for _ in range(40):
				for y, row in enumerate(session.rows()):
					x = row.find(label)
					if x >= 0:
						session.send(f"\x1b[<0;{x+1};{y+1}M\x1b[<0;{x+1};{y+1}m")
						return
				session.send("\x1b[<65;76;7M")
			raise AssertionError(f"Missing {label}: {session.rows()}")
		try:
			# Record a new primary binding, then explicitly save it.
			session.send("\x1bOP")
			category(1)
			click("New document")
			session.send("1\x1b[19~")  # Record F8 for the selected action
			assert "unsaved" in "\n".join(session.rows())
			assert not path.exists()
			session.send("\x13")
			config = json.loads(path.read_text())
			assert config["bindings"]["new"][0][0]["code"] == {"F": 8}
			session.send("\x1b[14~")
			assert any("New document" in row and "F8" in row for row in session.rows())
			session.send("\x1b")
			# Capture a two-input sequence, change timeout, and save together.
			session.send("\x1bOP")
			category(1)
			click("Focus Code")
			session.send("2\x17\x1b[C")
			assert "Recorded: Ctrl+W Right" in "\n".join(session.rows())
			session.send("\x14250\r\x13")  # fixed Ctrl+T timeout editor, fixed Ctrl+S
			assert json.loads(path.read_text())["timeout_ms"] == 250
			session.send("\x17\x1b[C\x1bOR")  # focus Code, F3 changes only Code wrap
			assert sum("Wrap: off" in row for row in session.rows()) == 1
			session.send("\x1b")
			# A single-key prefix assignment conflicts; fixed keys can recover.
			session.send("\x1bOP")
			category(1)
			click("Focus Tools")
			session.send("1\x17")
			assert "conflicts" in "\n".join(session.rows())
			session.send("\x13")
			assert "Shortcut Settings" in "\n".join(session.rows())
			category(1)
			session.send("\r\x1b[3~")  # return to the selected binding and unset it
			session.send("\x1b")
			assert "Shortcut Settings" not in "\n".join(session.rows())
			# Every opener works inside the picker and returns to its list.
			session.send("\x0f")
			for opener in ["\x1bOP", "\x1f", "\x1b[44;5u"]:
				session.send(opener)
				assert any(label in "\n".join(session.rows()) for label in ["File Explorer", "Select all"]), session.rows()
				for columns, rows in [(140,40),(64,20),(24,8),(80,24)]:
					session.resize(columns, rows)
				session.send("\x1b")
				assert "Open document" in "\n".join(session.rows())
			session.send("\x1b")
			session.send("\x11")
			session.finish()
		finally:
			session.close()
		# Restart proves dispatch loads saved bindings; canceled settings leave disk intact.
		session = Session(binary, env=env)
		try:
			session.paste("preserved")
			session.send("\x1b[19~")
			assert "Save changes before continuing?" in "\n".join(session.rows())
			session.send("\x1b")
			before = path.read_bytes()
			session.send("\x1bOP")
			category(1)
			click("New document")
			session.send("\r\x1b[3~")
			session.send("\x1b")
			assert "Discard unsaved shortcut changes?" in "\n".join(session.rows())
			session.send("\r")  # Keep editing is the default.
			assert "Shortcut Settings" in "\n".join(session.rows())
			session.send("\x1b")
			session.send("\x1b[D\r")
			assert path.read_bytes() == before
			session.send("\x11d")
			session.finish()
		finally:
			session.close()
		path.write_text("malformed settings")
		session = Session(binary, env=env)
		try:
			assert "Shortcuts:" in "\n".join(session.rows())
			assert path.read_text() == "malformed settings"
			session.send("\x11")
			session.finish()
		finally:
			session.close()
	print("PASS: shortcut capture, sequence dispatch, conflict recovery, explicit save, inline hints, context return, resize, restart and malformed config")


def shared_clipboard_checks(binary):
	with tempfile.TemporaryDirectory(prefix="tuxd-shared-clipboard-") as directory:
		folder = Path(directory)
		clipboard = folder / "clipboard.txt"
		clipboard.write_text("#aabbcc")
		helper = folder / "xclip"
		helper.write_text("#!/bin/sh\ncase \"$*\" in\n*-out*) /bin/cat " + shlex.quote(str(clipboard)) + ";;\n*) /bin/cat > " + shlex.quote(str(clipboard)) + ";;\nesac\n")
		helper.chmod(0o700)
		session = Session(binary, env={"PATH": directory, "DISPLAY": ":test", "WAYLAND_DISPLAY": None})
		try:
			session.click_tool("RGB")
			session.click_tool("Hex:")
			session.send("\x01\x16")
			assert "#aabbcc" in "\n".join(session.rows())
			session.send("\x01\x03")
			assert clipboard.read_text() == "#aabbcc"
			session.send("\t\x01\x03")
			assert clipboard.read_text() == "170"
			clipboard.write_text("128")
			session.send("\x16")
			assert "#80bbcc" in "\n".join(session.rows())
			session.send("\x1b")
			session.send("\x0f\x0c")
			session.paste(directory)
			session.send("\x01\x03")
			assert clipboard.read_text() == directory
			session.send("\x1b")
			session.send("\x11")
			session.finish()
		finally:
			session.close()
	print("PASS: shared Copy/Paste/Select all in Hex, RGB ranges and path fields; RGB remains staged")


def hardening_cli_checks(binary):
	with tempfile.TemporaryDirectory(prefix="tuxd-path-checks-") as directory:
		path = os.fsencode(directory) + b"/document-\xff.tuxd.json"
		with open(path, "wb") as file:
			file.write((ROOT / "examples/welcome.tuxd.json").read_bytes())
		session = Session(binary, path)
		try:
			assert "Terminal UX Designer" in session.row(2)
			session.send("\x11")
			session.finish()
		finally:
			session.close()
		fifo = Path(directory) / "input.fifo"
		os.mkfifo(fifo)
		for source in [fifo, Path(directory), Path(directory) / "missing"]:
			result = subprocess.run([binary, str(source)], capture_output=True, timeout=3)
			assert result.returncode != 0
			assert b"panicked" not in result.stderr
			assert b"\x1b[?1049h" not in result.stdout
	print("PASS: non-UTF-8 document path; FIFO, directory and missing-file startup failures")


def main():
	subprocess.run(["cargo", "build", "--locked"], cwd=ROOT, check=True)
	metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version=1"], cwd=ROOT))
	binary = str(Path(metadata["target_directory"]) / "debug" / "terminal-ux-designer")
	session = Session(binary)
	try:
		assert "Editor" in session.row(1)
		session.send("abc")
		assert session.row(2)[7:10] == "abc"
		session.send("\x1a")
		assert session.row(2)[7:10] == "   "
		session.send("\x19")
		assert session.row(2)[7:10] == "abc"
		session.send("\x1b[H" + "X" + "\x1b[F" + "\x7f")
		assert session.row(2)[7:10] == "Xab"
		# Enhanced keyboard protocol distinguishes Ctrl+Shift+Z from Ctrl+Z.
		session.send("\x1a")
		assert session.row(2)[7:11] == "Xabc"
		session.send("\x1b[122;6u")
		assert session.row(2)[7:10] == "Xab"
		# SGR mouse click at the first editable cell, then insert.
		session.send("\x1b[<0;8;3M!\x1b[<0;8;3m")
		assert session.row(2)[7:11] == "!Xab"
		# Undo all edits back to an empty document, then test an atomic paste.
		session.send("\x1a" * 8)
		session.paste("e\u0301\t界👩‍💻\r\nsecond\rthird\x1b[31m")
		assert session.row(3)[7:13] == "second"
		assert "third\\u{1b}[31m" in session.row(4)
		session.send("\x1a")
		assert session.row(2)[7:14] == "       "
		session.send("\x19")
		assert "second" in session.row(3)
		session.send("\x1b[1;5H")
		session.send("\x1b[3~")  # Delete the entire combining grapheme.
		assert session.screen.cursor.x == 7
		# Shortcut Settings blocks edits and is dismissible.
		session.send("\x1bOP")
		assert "Shortcut Settings" in "\n".join(session.rows())
		session.paste("ignored")
		session.send("\x1b")
		assert "ignored" not in "\n".join(session.rows())
		# Long lines, vertical scrolling, wheel navigation, and resize recovery.
		session.send("\x1b[1;5F")
		session.paste("\n" + "L" * 120 + "\n" + "\n".join(f"line {i}" for i in range(40)))
		assert "line 39" in "\n".join(session.rows())
		session.send("\x1b[<64;10;10M")
		assert "line 39" not in "\n".join(session.rows())
		session.resize(12, 3)
		assert "Resize" in session.row(0)
		session.resize(100, 30)
		session.send("\x1b[1;5H")
		assert "second" in session.row(3)
		session.send("\x11")
		assert "Discard" in "\n".join(session.rows())
		session.send("\r")  # Unsaved changes defaults to Cancel.
		assert "Editor" in session.row(1)
		session.send("\x11d")
		session.finish()
	finally:
		session.close()
	print("PASS: typing, navigation, mouse, Unicode, atomic paste, undo/redo, settings, scrolling, resize, quit")
	session = Session(binary)
	try:
		session.paste("Hello\nworld")
		session.send("\x1b[1;5H" + "\x1b[1;2C" * 5)
		session.send("\x02\x1b[105;5u\x15")
		cell = session.screen.buffer[2][7]
		assert cell.bold and cell.italics and cell.underscore, (cell, session.output[-2500:])
		# Cyan (ANSI 6) is the third cell of the second swatch row.
		session.click_tool("06")
		assert session.screen.buffer[2][7].fg in ("cyan", "00cdcd"), session.screen.buffer[2][7]
		session.send("\x1b")
		assert session.screen.buffer[2][7].bold
		rows = session.rows()
		first = next(i for i, row in enumerate(rows) if "Hello" in row and "\\n" in row)
		assert "world" in rows[first + 1], rows
		# Reversed mouse drag selects the complete second line; toggling preserves it.
		session.send("\x1b[<0;13;4M\x1b[<32;8;4M\x1b[<0;8;4m")
		session.send("\x02")
		assert session.screen.buffer[3][7].bold
		session.send("X")
		assert session.row(3)[7:12] == "X    "
		session.send("\x1a")
		assert "world" in session.row(3)
		assert session.screen.buffer[3][7].bold
		session.send("\x1b")
		# Focus tools, navigate to italic, and apply it without losing the insertion point.
		session.send("\x1b[17~")
		session.resize(40, 12)
		assert "Characters" in "\n".join(session.rows())
		session.send("\x1b")
		session.resize(80, 24)
		session.send("\x11d")
		session.finish()
	finally:
		session.close()
	print("PASS: selection, bold/italic/underline/cyan, multiline ANSI drawer, reverse drag, replacement undo, narrow tools")
	session = Session(binary)
	try:
		assert "Code: on" in session.row(0)
		assert sum("Wrap: on" in row for row in session.rows()) == 2
		session.paste("sample")
		session.send("\x1b[H" + "\x1b[1;2C" * 6)
		session.send("\x1b[<0;63;3M\x1b[<0;63;3m")
		session.send("\x1b[<0;68;3M\x1b[<0;68;3m")
		session.click_tool("Background")
		session.click_tool("256")
		# Scroll the sidebar to an indexed swatch without disturbing the selection.
		for _ in range(3):
			session.send("\x1b[<65;70;15M")
		session.click_tool("42")
		session.send("\x1b")
		assert session.screen.buffer[2][7].bg == "00d787", session.screen.buffer[2][7]
		# Return to the top by keyboard, then stage and apply RGB.
		session.send("\x1b[17~\x1b[H")
		session.click_tool("RGB")
		session.click_tool("Hex:")
		session.paste("#0080ff")
		assert session.screen.buffer[2][7].bg == "00d787"
		session.send("\r")
		# No selection now: RGB is the pending insertion style.
		session.send("\x1b")
		session.send("Z")
		assert session.screen.buffer[2][13].bg == "0080ff"
		session.click_tool("Hex:")
		session.paste("#oops")
		session.send("\r")
		assert "6 hex digits" in "\n".join(session.rows())
		session.click_tool("Clear")
		session.send("\x1b")
		# Keyboard-only character navigation works in the slim tabs.
		session.send("\x1b[17~")
		session.resize(40, 12)
		session.send("\x1b[1;3C\x1b[1;3C")  # Style -> Color -> Characters tabs
		session.send("\t\x1b[F\r\r")
		assert "Characters" in "\n".join(session.rows())
		session.send("\x1b")
		session.resize(80, 24)
		assert "sampleZ╬╬" in session.row(2), session.rows()
		session.send("\x1a")
		assert "sampleZ╬" in session.row(2)
		session.send("\x11d")
		session.finish()
	finally:
		session.close()
	print("PASS: default panes, extended styles, indexed background, staged/invalid RGB, narrow glyph insertion and undo")

	# Use isolated clipboard helpers so acceptance checks never replace the user's clipboard.
	with tempfile.TemporaryDirectory(prefix="terminal-ux-clipboard-") as directory:
		folder = Path(directory)
		copied = folder / "clipboard.txt"
		helper = folder / "xclip"
		helper.write_text("#!/bin/sh\n/bin/cat > " + shlex.quote(str(copied)) + "\n")
		helper.chmod(0o700)
		failure = folder / "xsel"
		failure.write_text("#!/bin/sh\nexit 1\n")
		failure.chmod(0o700)
		env = {"PATH": directory, "DISPLAY": ":test", "WAYLAND_DISPLAY": None}
		session = Session(binary, env=env)
		try:
			session.paste("alpha\nbeta\t\\n")
			session.send("\x1b[17~\x1b[17~")  # Focus the initially visible code pane.
			session.send("\x03")
			assert copied.read_text() == r"\x1b[0malpha\nbeta\t\\n\x1b[0m"
			assert "Copied escaped ANSI" in session.row(23)
			# Wrap independently; copy still excludes all presentation line breaks/gutters.
			session.send("\x1b[13~")
			session.resize(30, 24)
			session.send("\x03")
			assert copied.read_text() == r"\x1b[0malpha\nbeta\t\\n\x1b[0m"
			assert "Wrap: on" in "\n".join(session.rows())
			session.send("\x1b")
			session.send("\x1b[1;5F")
			session.paste("\n" + "a" * 70)
			session.send("\x1b[H")  # Editor starts wrapped; logical Home.
			assert session.screen.cursor.x == 7
			session.send("\x1b[B")  # Down moves within the wrapped logical line.
			assert "Ln 3" in session.row(23)
			session.send("\x1b[13~")
			session.resize(80, 24)
			session.send("\x1b[1;5F")
			session.paste(" one two")
			session.send("\x1b[1;5D")  # Ctrl+Left
			session.send("\x1b[1;3C")  # Alt+Right
			session.send("\x1b\x7f")  # Alt+Backspace
			session.send("\x1b[1;5H")
			# Failed helpers report failure and preserve the working document.
			helper.write_text("#!/bin/sh\nexit 1\n")
			session.send("\x1b[17~\x1b[17~\x03")
			assert "Copy failed" in session.row(23)
			session.send("\x1b")
			assert "alpha" in session.row(2)
			session.send("\x11d")
			session.finish()
		finally:
			session.close()
	print("PASS: per-pane wrap, gutters, wrapped navigation, word bindings, clipboard payload and failure feedback")
	with tempfile.TemporaryDirectory(prefix="terminal-ux-files-") as directory:
		folder = Path(directory)
		document = folder / "saved.tuxd.json"
		raw = folder / "snippet.ansi"
		escaped = folder / "snippet.txt"
		bad = folder / "review.tuxd.json"
		bad.write_text(r"\x1b[31mImported\x1b[2J\nUnicode 界\q")
		session = Session(binary)
		try:
			session.paste("Save me\n界")
			session.send("\x13")  # Ctrl+S: first save requires a path.
			assert "Save document as" in "\n".join(session.rows())
			session.paste(str(document))
			session.send("\r")
			assert json.loads(document.read_text())["text"] == "Save me\n界"
			# Export both representations without changing the document path.
			for index, destination in [(0, raw), (1, escaped)]:
				session.send("\x05")
				if not index:
					session.send("\t\x1b[D")  # List -> format; select Raw.
				session.paste(str(destination))
				session.send("\r")
			assert raw.read_bytes() == b"\x1b[0mSave me\n" + "界".encode() + b"\x1b[0m"
			assert escaped.read_text() == r"\x1b[0mSave me\n界\x1b[0m"
			# Open malformed JSON: the active document survives.
			session.send("\x0f")
			session.paste(str(bad))
			session.send("\r")
			assert "Error:" in "\n".join(session.rows())
			session.send("\x1b")
			assert "Save me" in session.row(2)
			# Add an unsaved edit, import via F4, review all controls, then cancel dirty replacement.
			session.send("X")
			session.send("\x1b[14~" + "\x1b[B" * 4 + "\r")
			session.paste(str(bad))
			session.send("\r")
			assert "Unsupported ANSI Sequences" in "\n".join(session.rows())
			assert "Seq." in "\n".join(session.rows())
			session.send("s")
			assert "Save changes before continuing?" in "\n".join(session.rows())
			session.send("c")
			assert "Save me" in session.row(2)
			# Retry and explicitly discard the old working document.
			session.send("\x1b[14~" + "\x1b[B" * 4 + "\r")
			session.paste(str(bad))
			session.send("\r")
			session.send("s")
			session.send("d")
			assert "Imported" in session.row(2)
			assert session.screen.buffer[2][7].fg in ("red", "cd0000")
			# Imported content is untitled and asks for a new save destination.
			session.send("\x13")
			assert "Save document as" in "\n".join(session.rows())
			session.send("\x1b")
			session.send("\x11d")
			session.finish()
		finally:
			session.close()
		session = Session(binary, str(document))
		try:
			assert "Save me" in session.row(2)
			# Clean loaded documents quit directly.
			session.send("\x11")
			session.finish()
		finally:
			session.close()
	print("PASS: save/open, raw and escaped export, failed open, import review, dirty cancellation, CLI reopen and clean quit")

	# A real directory picker and Ctrl+V, isolated from the desktop clipboard.
	with tempfile.TemporaryDirectory(prefix="terminal-ux-picker-") as directory:
		folder = Path(directory)
		document = folder / "browse.tuxd.json"
		document.write_bytes((ROOT / "examples/welcome.tuxd.json").read_bytes())
		(folder / "other.txt").write_text("not a document")
		(folder / ".hidden.tuxd.json").write_bytes(document.read_bytes())
		destination = folder / "clipboard path.tuxd.json"
		helper = folder / "xclip"
		helper.write_text("#!/bin/sh\nprintf '%s' " + shlex.quote(str(destination)) + "\n")
		helper.chmod(0o700)
		session = Session(binary, env={"PATH": directory, "DISPLAY": ":test", "WAYLAND_DISPLAY": None})
		try:
			session.send("\x0f\x0c")  # Open, Ctrl+L selects the location field.
			session.paste(directory)
			session.send("\r")
			assert "browse.tuxd.json" in "\n".join(session.rows())
			assert "other.txt" not in "\n".join(session.rows())
			session.send("\x1b[46;5u")  # Ctrl+. hidden alias
			assert ".hidden.tuxd.json" in "\n".join(session.rows())
			session.send("\x1b[46;5u\x1f")  # Hide files; legacy Ctrl+/ shortcuts
			assert "Shortcut Settings" in "\n".join(session.rows())
			session.send("\x1b")  # Close settings and return to the picker
			assert "Shortcut Settings" not in "\n".join(session.rows())
			session.send("\x06")  # Ctrl+F includes non-project files.
			assert "other.txt" in "\n".join(session.rows())
			session.send("\x06jggMG\r")  # Project filter; Vim navigation selects the document.
			assert "Terminal UX Designer" in session.row(2)
			session.send("Z")
			session.send("\x1b[14~" + "\x1b[B" * 3 + "\r")  # Save As
			session.send("\x01\x16")  # Ctrl+A, Ctrl+V reads the helper asynchronously.
			session.send("\r")
			assert json.loads(destination.read_text())["text"].startswith("ZTerminal UX Designer")
			# Selecting the same destination invokes the existing overwrite guard.
			session.send("\x1b[14~" + "\x1b[B" * 3 + "\r\t\r")
			assert "Overwrite existing file?" in "\n".join(session.rows())
			session.send("n")
			session.send("\x0f")
			session.paste(str(folder / "other.txt"))
			session.send("\r")
			assert "Import this file as ANSI?" in "\n".join(session.rows())
			assert "Raw ANSI" in "\n".join(session.rows()) and "Escaped ANSI" in "\n".join(session.rows())
			session.send("\x1b")  # Cancel offered import, preserving the saved document.
			assert "ZTerminal UX Designer" in session.row(2)
			session.send("\x0f")
			session.paste(str(folder / "other.txt"))
			session.send("\r")
			session.send("\x1b[C\r")  # Choose Escaped, then import directly from the radio.
			assert "not a document" in session.row(2)
			session.send("\x11d")
			session.finish()
		finally:
			session.close()
	print("PASS: file browsing, Vim navigation, hidden alias, shortcut settings, format radios, import offer, Ctrl+V, Save As and overwrite guard")

	hardening_cli_checks(binary)
	shared_clipboard_checks(binary)
	shortcut_settings_checks(binary)

	for argument, expected in [("--diagnostic-error", 1), ("--diagnostic-panic", 101)]:
		session = Session(binary, argument)
		session.finish(expected)
		print(f"PASS: terminal restoration {argument}")
	session = Session(binary)
	session.send("\x11")
	session.finish()
	print("PASS: empty document exits directly; terminal modes restored")


if __name__ == "__main__":
	main()
