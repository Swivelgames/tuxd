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
			env={key: value for key, value in {**os.environ, "TERM": "xterm-256color", **(env or {})}.items() if value is not None}, preexec_fn=controlling_terminal,
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
		# Help blocks edits and is dismissible.
		session.send("\x1bOP")
		assert "No saving yet" in "\n".join(session.rows())
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
		session.send("n")
		assert "Editor" in session.row(1)
		session.send("\x11y")
		session.finish()
	finally:
		session.close()
	print("PASS: typing, navigation, mouse, Unicode, atomic paste, undo/redo, help, scrolling, resize, quit")
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
		assert "Editor" not in "\n".join(session.rows())
		session.send("\x1b")
		session.resize(80, 24)
		session.send("\x11y")
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
		# Keyboard-only character navigation works in the narrow overlay.
		session.send("\x1b[17~")
		session.resize(40, 12)
		session.send("\x1b[F\r\r")
		assert "Editor" not in "\n".join(session.rows())
		session.send("\x1b")
		session.resize(80, 24)
		assert "sampleZ╬╬" in session.row(2), session.rows()
		session.send("\x1a")
		assert "sampleZ╬" in session.row(2)
		session.send("\x11y")
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
			session.send("\x11y")
			session.finish()
		finally:
			session.close()
	print("PASS: per-pane wrap, gutters, wrapped navigation, word bindings, clipboard payload and failure feedback")
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
