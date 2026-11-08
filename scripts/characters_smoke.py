# /// script
# requires-python = ">=3.10"
# dependencies = ["pyte==0.8.2"]
# ///
"""Character discovery through real terminal events: uv run scripts/characters_smoke.py."""
import json
from pathlib import Path
import subprocess
import tempfile
from terminal_smoke import ROOT, Session


def main():
	subprocess.run(["cargo", "build", "--locked"], cwd=ROOT, check=True)
	metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version=1"], cwd=ROOT))
	binary = str(Path(metadata["target_directory"]) / "debug/tuxd")
	with tempfile.TemporaryDirectory(prefix="tuxd-characters-") as directory:
		path = Path(directory) / "characters.tuxd.json"
		document = json.loads((ROOT / "examples/welcome.tuxd.json").read_text())
		document["text"] = ""
		document["styles"]["runs"] = []
		path.write_text(json.dumps(document))
		session = Session(binary, str(path))
		try:
			session.resize(64, 36)
			session.send("\x1b[17~\x1b[1;3C\x1b[1;3C\x1b[1;3C\x1b[15~")  # Tools, Characters, maximize lower.
			assert "Group:" in "\n".join(session.rows()), session.rows()
			session.send("\x06")
			session.paste("U+2500")
			session.send("\r")  # Focus results only.
			assert "U+2500" in "\n".join(session.rows()), session.rows()
			session.send("\x1b[H\r")  # Collapse the result type without inserting.
			assert "▸ Box Drawing" in "\n".join(session.rows()), session.rows()
			session.send("\r\x1b[B")  # Expand and move back to the glyph.
			assert "▾ Box Drawing" in "\n".join(session.rows()), session.rows()
			session.send("\r\r\x1b")
			session.send("\x13")
			assert json.loads(path.read_text())["text"] == "──", session.rows()
			session.send("\x1a\x13")
			assert json.loads(path.read_text())["text"] == "─"
			session.send("\x1b[17~\x1b[15~\x06\x01")
			session.paste("md home")
			session.send("\r\x1b[F\x1b[5~\x1b[H")
			assert "Nerd Icons" in "\n".join(session.rows()), session.rows()
			for columns, rows in [(40, 12), (120, 40), (64, 36)]:
				session.resize(columns, rows)
				assert session.process.poll() is None
			# The same character control stays focused across wide/slim changes.
			session.send("\x06\x01")
			session.paste("U+1F680")
			assert "Emoji" in "\n".join(session.rows()), session.rows()
			session.send("\r\r\x1b")
			session.send("\x13")
			assert json.loads(path.read_text())["text"] == "─🚀", session.rows()
			session.send("\x1a\x13")
			assert json.loads(path.read_text())["text"] == "─"
			session.send("\x1b[17~\x1b[15~\x06\x01")
			session.paste("U+200D")
			assert "No matching characters" in "\n".join(session.rows()), session.rows()
			session.send("\r\x1b")
			session.send("\x11")
			session.finish()
		finally:
			session.close()
	print("PASS: character search, focus vs insertion, repeated insertion, save/undo, large-grid navigation, resize, unsupported exclusion, exit")


if __name__ == "__main__":
	main()
