# /// script
# dependencies = ["pyte==0.8.2"]
# ///
"""Persistent theme overrides, safe fallback and document independence."""
import json
import subprocess
import tempfile
from pathlib import Path
from terminal_smoke import ROOT, Session

subprocess.run(["cargo", "build", "--locked"], cwd=ROOT, check=True)
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version=1"], cwd=ROOT))
binary = str(Path(metadata["target_directory"]) / "debug" / "terminal-ux-designer")
with tempfile.TemporaryDirectory() as directory:
	root = Path(directory)
	config = root / "tuxd" / "theme.json"
	config.parent.mkdir()
	doc = root / "test.tuxd.json"
	document = {"version": 1, "text": "Document", "styles": {"runs": [{"start": 0, "end": 8, "style": {"bold": False, "italic": False, "underline": False, "dim": False, "strike": False, "foreground": "Default", "background": "Default"}}]}}
	doc.write_text(json.dumps(document))
	for valid in [True, False]:
		config.write_text(json.dumps({"version": 1, "preset": "midnight", "colors": {"accent": "#123456", "surface": "#334455", "text": "#abcdef"}}) if valid else "{invalid")
		original = config.read_bytes()
		session = Session(binary, str(doc), env={"XDG_CONFIG_HOME": directory})
		try:
			if valid:
				assert session.screen.buffer[0][1].fg == "123456", (session.screen.buffer[0][1], session.rows())
				found = False
				for y, row in enumerate(session.rows()):
					if "Document" in row:
						x = row.index("Document")
						assert session.screen.buffer[y][x].fg == "default"
						assert session.screen.buffer[y][x].bg == "default"
						found = True
						break
				assert found
			else:
				assert "Theme:" in "\n".join(session.rows())
			session.send("\x13\x11")
			session.finish()
			assert json.loads(doc.read_text()) == document
			assert config.read_bytes() == original
		finally:
			session.close()
print("PASS: theme loading, RGB overrides, document colors and persistence, invalid configuration fallback")
