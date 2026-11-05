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

import pyte


ROOT = Path(__file__).resolve().parents[1]


class Session:
	def __init__(self, binary, *args):
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
			env={**os.environ, "TERM": "xterm-256color"}, preexec_fn=controlling_terminal,
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
		assert session.row(2)[1:4] == "abc"
		session.send("\x1a")
		assert session.row(2)[1:4] == "   "
		session.send("\x19")
		assert session.row(2)[1:4] == "abc"
		session.send("\x1b[H" + "X" + "\x1b[F" + "\x7f")
		assert session.row(2)[1:4] == "Xab"
		# Enhanced keyboard protocol distinguishes Ctrl+Shift+Z from Ctrl+Z.
		session.send("\x1a")
		assert session.row(2)[1:5] == "Xabc"
		session.send("\x1b[122;6u")
		assert session.row(2)[1:4] == "Xab"
		# SGR mouse click at the first editable cell, then insert.
		session.send("\x1b[<0;2;3M!\x1b[<0;2;3m")
		assert session.row(2)[1:5] == "!Xab"
		# Undo all edits back to an empty document, then test an atomic paste.
		session.send("\x1a" * 8)
		session.paste("e\u0301\t界👩‍💻\r\nsecond\rthird\x1b[31m")
		assert session.row(3)[1:7] == "second"
		assert "third\\u{1b}[31m" in session.row(4)
		session.send("\x1a")
		assert session.row(2)[1:8] == "       "
		session.send("\x19")
		assert "second" in session.row(3)
		session.send("\x1b[1;5H")
		session.send("\x1b[3~")  # Delete the entire combining grapheme.
		assert session.screen.cursor.x == 1
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
