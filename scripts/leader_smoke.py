# /// script
# dependencies = ["pyte==0.8.2"]
# ///
"""General settings: leader confirmation, persistence and literal preservation."""
import json
import subprocess
import tempfile
from pathlib import Path
from terminal_smoke import ROOT, Session

subprocess.run(["cargo", "build", "--locked"], cwd=ROOT, check=True)
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version=1"], cwd=ROOT))
binary = str(Path(metadata["target_directory"]) / "debug" / "tuxd")
with tempfile.TemporaryDirectory() as directory:
	session = Session(binary, env={"XDG_CONFIG_HOME": directory})
	def leader():
		session.send("\x1bOP\x0cggjjj\r")
		session.send("\r")  # General opens with Leader focused; Enter records

	try:
		leader()
		session.send("\x19")
		assert "Redo" in "\n".join(session.rows())
		assert "unset" in "\n".join(session.rows())
		session.send("y\x13")
		path = Path(directory) / "tuxd" / "shortcuts.json"
		config = json.loads(path.read_text())
		assert config["leader"] is not None
		leader()
		session.send("\x1b")  # cancel capture, keep leader focused
		session.send("\x1b[3~")
		assert "Unset Leader?" not in "\n".join(session.rows())
		session.send("\x13")
		assert json.loads(path.read_text())["leader"] is None
		session.send("\x11")
		session.finish()
	finally:
		session.close()
print("PASS: General navigation, leader capture, affected-binding warning, explicit save and unset")
