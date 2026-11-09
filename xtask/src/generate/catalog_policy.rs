//! Curated catalog admission policy; deliberately excludes unrelated Unicode blocks.

pub const ICON_FAMILIES: &[(&str, &str)] = &[
	("cod", "Codicons"),
	("custom", "Custom"),
	("dev", "Devicons"),
	("extra", "Extra"),
	("fa", "Font Awesome"),
	("fae", "Font Awesome Extension"),
	("iec", "IEC Power"),
	("indent", "Indentation"),
	("indentation", "Indentation"),
	("linux", "Linux logos"),
	("md", "Material Design"),
	("oct", "Octicons"),
	("pl", "Powerline"),
	("ple", "Powerline Extra"),
	("pom", "Pomicons"),
	("seti", "Seti"),
	("weather", "Weather"),
];

pub const LETTER_BLOCKS: &[(&str, &str)] = &[
	("Basic Latin", "Basic Latin"),
	("Latin-1 Supplement", "Latin accents"),
	("Latin Extended-A", "Latin accents"),
	("Latin Extended-B", "Latin accents"),
	("Greek and Coptic", "Greek"),
	("Cyrillic", "Cyrillic"),
	("Cyrillic Supplement", "Cyrillic"),
	("Enclosed Alphanumerics", "Enclosed letters & numbers"),
	(
		"Enclosed Alphanumeric Supplement",
		"Enclosed letters & numbers",
	),
	("Number Forms", "Fractions & Roman numerals"),
];

pub const DESIGN_BLOCKS: &[(&str, (&str, &str))] = &[
	("Arrows", ("Arrows", "Standard")),
	("Supplemental Arrows-A", ("Arrows", "Supplemental")),
	("Supplemental Arrows-B", ("Arrows", "Supplemental")),
	("Supplemental Arrows-C", ("Arrows", "Supplemental")),
	(
		"Miscellaneous Symbols and Arrows",
		("Shapes", "Additional shapes"),
	),
	("Geometric Shapes", ("Shapes", "Geometric")),
	("Geometric Shapes Extended", ("Shapes", "Geometric")),
	("Mathematical Operators", ("Mathematics", "Operators")),
	(
		"Supplemental Mathematical Operators",
		("Mathematics", "Operators"),
	),
	(
		"Miscellaneous Mathematical Symbols-A",
		("Mathematics", "Notation"),
	),
	(
		"Miscellaneous Mathematical Symbols-B",
		("Mathematics", "Notation"),
	),
	(
		"Mathematical Alphanumeric Symbols",
		("Mathematics", "Styled alphabets"),
	),
	(
		"Superscripts and Subscripts",
		("Mathematics", "Superscripts & subscripts"),
	),
	("Letterlike Symbols", ("Mathematics", "Letterlike symbols")),
	("General Punctuation", ("Typography", "Punctuation")),
	("Supplemental Punctuation", ("Typography", "Punctuation")),
	("Currency Symbols", ("Typography", "Currency")),
	(
		"CJK Symbols and Punctuation",
		("Typography", "Wide punctuation"),
	),
	(
		"Miscellaneous Technical",
		("Symbols", "Technical & interface"),
	),
	("Control Pictures", ("Symbols", "Control pictures")),
	(
		"Optical Character Recognition",
		("Symbols", "Technical & interface"),
	),
	("Miscellaneous Symbols", ("Symbols", "General symbols")),
	(
		"Miscellaneous Symbols Supplement",
		("Symbols", "General symbols"),
	),
	("Dingbats", ("Symbols", "Dingbats")),
	("Ornamental Dingbats", ("Symbols", "Ornaments")),
	(
		"Miscellaneous Symbols and Pictographs",
		("Symbols", "Pictographs"),
	),
	("Transport and Map Symbols", ("Symbols", "Transport & maps")),
	("Musical Symbols", ("Symbols", "Music")),
	("Chess Symbols", ("Symbols", "Games")),
	("Playing Cards", ("Symbols", "Games")),
	("Domino Tiles", ("Symbols", "Games")),
	("Mahjong Tiles", ("Symbols", "Games")),
	(
		"Symbols for Legacy Computing",
		("Blocks & Shading", "Legacy computing"),
	),
	(
		"Symbols for Legacy Computing Supplement",
		("Blocks & Shading", "Legacy computing"),
	),
];
