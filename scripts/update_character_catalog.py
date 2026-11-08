#!/usr/bin/env python3
"""Regenerate bundled character metadata from checksum-pinned upstream datasets."""
import hashlib
import json
from pathlib import Path
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "licenses/characters/sources.json"
OUTPUT = ROOT / "src/libraries/characters/data/catalog.tsv"

ICON_FAMILIES = {
	'cod': 'Codicons',
	'custom': 'Custom',
	'dev': 'Devicons',
	'extra': 'Extra',
	'fa': 'Font Awesome',
	'fae': 'Font Awesome Extension',
	'iec': 'IEC Power',
	'indent': 'Indentation',
	'indentation': 'Indentation',
	'linux': 'Linux logos',
	'md': 'Material Design',
	'oct': 'Octicons',
	'pl': 'Powerline',
	'ple': 'Powerline Extra',
	'pom': 'Pomicons',
	'seti': 'Seti',
	'weather': 'Weather',
}


def fetch(source):
	data = urllib.request.urlopen(source["url"], timeout=60).read()
	if hashlib.sha256(data).hexdigest() != source["sha256"]:
		raise ValueError(f"Checksum mismatch: {source['url']}")
	return data.decode("utf-8")


def ranges(text, property_name=None):
	for line in text.splitlines():
		line = line.split("#")[0].strip()
		if not line:
			continue
		span, name = map(str.strip, line.split(";")[:2])
		if property_name and name != property_name:
			continue
		ends = span.split("..")
		yield int(ends[0], 16), int(ends[-1], 16), name


# Admission is deliberately about terminal-design use, not all assigned Unicode.
# Keep useful families complete; do not grow the picker by admitting every script.
LETTER_BLOCKS = {
	"Basic Latin": "Basic Latin",
	"Latin-1 Supplement": "Latin accents",
	"Latin Extended-A": "Latin accents",
	"Latin Extended-B": "Latin accents",
	"Greek and Coptic": "Greek",
	"Cyrillic": "Cyrillic",
	"Cyrillic Supplement": "Cyrillic",
	"Enclosed Alphanumerics": "Enclosed letters & numbers",
	"Enclosed Alphanumeric Supplement": "Enclosed letters & numbers",
	"Number Forms": "Fractions & Roman numerals",
}

DESIGN_BLOCKS = {
	"Arrows": ("Arrows", "Standard"),
	"Supplemental Arrows-A": ("Arrows", "Supplemental"),
	"Supplemental Arrows-B": ("Arrows", "Supplemental"),
	"Supplemental Arrows-C": ("Arrows", "Supplemental"),
	"Miscellaneous Symbols and Arrows": ("Shapes", "Additional shapes"),
	"Geometric Shapes": ("Shapes", "Geometric"),
	"Geometric Shapes Extended": ("Shapes", "Geometric"),
	"Mathematical Operators": ("Mathematics", "Operators"),
	"Supplemental Mathematical Operators": ("Mathematics", "Operators"),
	"Miscellaneous Mathematical Symbols-A": ("Mathematics", "Notation"),
	"Miscellaneous Mathematical Symbols-B": ("Mathematics", "Notation"),
	"Mathematical Alphanumeric Symbols": ("Mathematics", "Styled alphabets"),
	"Superscripts and Subscripts": ("Mathematics", "Superscripts & subscripts"),
	"Letterlike Symbols": ("Mathematics", "Letterlike symbols"),
	"General Punctuation": ("Typography", "Punctuation"),
	"Supplemental Punctuation": ("Typography", "Punctuation"),
	"Currency Symbols": ("Typography", "Currency"),
	"CJK Symbols and Punctuation": ("Typography", "Wide punctuation"),
	"Miscellaneous Technical": ("Symbols", "Technical & interface"),
	"Control Pictures": ("Symbols", "Control pictures"),
	"Optical Character Recognition": ("Symbols", "Technical & interface"),
	"Miscellaneous Symbols": ("Symbols", "General symbols"),
	"Miscellaneous Symbols Supplement": ("Symbols", "General symbols"),
	"Dingbats": ("Symbols", "Dingbats"),
	"Ornamental Dingbats": ("Symbols", "Ornaments"),
	"Miscellaneous Symbols and Pictographs": ("Symbols", "Pictographs"),
	"Transport and Map Symbols": ("Symbols", "Transport & maps"),
	"Musical Symbols": ("Symbols", "Music"),
	"Chess Symbols": ("Symbols", "Games"),
	"Playing Cards": ("Symbols", "Games"),
	"Domino Tiles": ("Symbols", "Games"),
	"Mahjong Tiles": ("Symbols", "Games"),
	"Symbols for Legacy Computing": ("Blocks & Shading", "Legacy computing"),
	"Symbols for Legacy Computing Supplement": ("Blocks & Shading", "Legacy computing"),
}


def emoji_catalog(text):
	"""Only standalone, fully-qualified emoji; no components or sequences."""
	emojis = {}
	group = None
	for line in text.splitlines():
		if line.startswith("# group: "):
			group = line.removeprefix("# group: ")
		if not line or line.startswith("#"):
			continue
		data, description = line.split("#", 1)
		codes, status = map(str.strip, data.split(";"))
		if status != "fully-qualified" or len(codes.split()) != 1:
			continue
		# The description contains the rendered glyph, emoji version, and CLDR name.
		name = description.strip().split(" ", 2)[2]
		emojis[int(codes, 16)] = (group, name, len(emojis))
	return emojis


def classification(name, category, block):
	if block == "Box Drawing":
		if ("LIGHT" in name and "HEAVY" in name) or ("SINGLE" in name and "DOUBLE" in name):
			return "Box Drawing", "Mixed junctions"
		for token, label in [
			("ARC", "Rounded"), ("DASH", "Dashed"), ("DIAGONAL", "Diagonal"),
			("DOUBLE", "Double"), ("HEAVY", "Heavy"), ("LIGHT", "Light"),
		]:
			if token in name:
				return "Box Drawing", label
	if block == "Block Elements":
		for token, label in [
			("SHADE", "Shading"), ("QUADRANT", "Quadrants"),
			("HALF", "Halves"), ("FULL", "Full"),
		]:
			if token in name:
				return "Blocks & Shading", label
		return "Blocks & Shading", "Fractions"
	if block == "Braille Patterns":
		return "Braille", "Patterns"
	if block in LETTER_BLOCKS:
		if category == "Sc":
			return "Typography", "Currency"
		if category.startswith("P"):
			return "Typography", "Punctuation"
		if category == "Sm":
			return "Mathematics", "Operators"
		if category[0] in "LN":
			return "Letters & Numbers", LETTER_BLOCKS[block]
		return "Symbols", "General symbols"
	if block == "CJK Symbols and Punctuation" and not category.startswith("P"):
		return None
	if block in DESIGN_BLOCKS:
		if "ARROW" in name or "HARPOON" in name:
			return "Arrows", "Standard" if block == "Arrows" else "Supplemental"
		return DESIGN_BLOCKS[block]
	return None


def generate(sources):
	blocks = list(ranges(sources["Blocks.txt"]))
	emojis = emoji_catalog(sources["emoji-test.txt"])
	ignored = set()
	for start, end, _ in ranges(sources["DerivedCoreProperties.txt"], "Default_Ignorable_Code_Point"):
		ignored.update(range(start, end + 1))

	aliases = {}
	for line in sources["NameAliases.txt"].splitlines():
		if line and not line.startswith("#"):
			code, name, _ = line.split(";")
			aliases.setdefault(int(code, 16), []).append(name)

	names = {
		cp: name.replace("*", f"{cp:04X}")
		for start, end, name in ranges(sources["DerivedName.txt"])
		for cp in range(start, end + 1)
	}

	entries = {}
	block_at = 0
	first = None
	for line in sources["UnicodeData.txt"].splitlines():
		fields = line.split(";")
		code, name, category = int(fields[0], 16), fields[1], fields[2]
		if name.endswith(", First>"):
			first = code
			continue
		start = first if name.endswith(", Last>") else code
		first = None
		if category[0] not in "LNPS":
			continue
		for cp in range(start, code + 1):
			if cp in ignored or cp == 0x2800 or 0x1F1E6 <= cp <= 0x1F1FF or 0x1F3FB <= cp <= 0x1F3FF:
				continue
			while blocks[block_at][1] < cp:
				block_at += 1
			block = blocks[block_at][2]
			label = names[cp]
			entry_aliases = list(aliases.get(cp, []))
			if cp in emojis:
				subgroup, emoji_name, _ = emojis[cp]
				group = "Emoji"
				entry_aliases.append(emoji_name)
			else:
				classified = classification(label, category, block)
				if classified is None:
					continue
				group, subgroup = classified
			entries[cp] = [group, subgroup, label, "|".join(entry_aliases)]

	for alias, item in sorted(json.loads(sources["glyphnames.json"]).items()):
		if alias == "METADATA":
			continue
		cp = int(item["code"], 16)
		prefix = alias.split("-", 1)[0]
		family = ICON_FAMILIES[prefix]
		if cp in entries:
			entries[cp][3] += "|" + alias
		else:
			entries[cp] = ["Nerd Icons", family, alias, ""]

	rows = []
	for cp, values in sorted(entries.items()):
		if values[0] == "Emoji":
			# Keep emoji together after ordinary scalars in global results, in CLDR order.
			order = 0x110000 + emojis[cp][2]
			values = [*values, str(order)]
		rows.append((f"{cp:X}\t" + "\t".join(values)).rstrip("\t") + "\n")
	return "".join(rows)


if __name__ == "__main__":
	manifest = json.loads(MANIFEST.read_text())
	sources = {name: fetch(source) for name, source in manifest["sources"].items()}
	OUTPUT.write_text(generate(sources), encoding="utf-8")
	print(f"Generated {OUTPUT.relative_to(ROOT)}")
