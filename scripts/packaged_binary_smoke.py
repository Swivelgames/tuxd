# /// script
# dependencies = ["pyte==0.8.2"]
# ///
"""Smoke-test an extracted release executable without replacing the installed app."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
from terminal_smoke import ROOT, Session


def main():
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("binary", type=Path)
	parser.add_argument("version")
	args = parser.parse_args()
	binary = str(args.binary.resolve())
	assert subprocess.check_output([binary, "--version"], text=True).strip() == f"tuxd {args.version}"
	with tempfile.TemporaryDirectory(prefix="tuxd-release-smoke-") as temporary:
		document = Path(temporary) / "check.tuxd.json"
		shutil.copyfile(ROOT / "examples/welcome.tuxd.json", document)
		original = json.loads(document.read_text())["text"]
		session = Session(binary, str(document))
		try:
			assert "Editor" in "\n".join(session.rows())
			session.send("\x1b[1;5F")
			session.paste(" release café 界")
			session.send("\x13")
			assert json.loads(document.read_text())["text"] == original + " release café 界"
			session.send("\x1bOP")
			assert "Settings" in "\n".join(session.rows())
			session.send("\x1b")
			session.resize(60, 24)
			assert "Code" in "\n".join(session.rows())
			session.send("\x11")
			session.finish()
		finally:
			session.close()
	print("PASS: extracted release executable and terminal restoration")


if __name__ == "__main__":
	main()
