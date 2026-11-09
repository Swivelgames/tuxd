use super::catalog_policy::{DESIGN_BLOCKS, ICON_FAMILIES, LETTER_BLOCKS};
use crate::{archive, process::Runner};
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use std::{
	collections::{BTreeMap, BTreeSet},
	fs,
	time::Duration,
};

#[derive(Deserialize)]
struct Source {
	url: String,
	sha256: String,
}
#[derive(Deserialize)]
struct Manifest {
	sources: BTreeMap<String, Source>,
}
struct Range {
	start: u32,
	end: u32,
	name: String,
}
struct Emoji {
	group: String,
	name: String,
	order: usize,
}
struct Entry {
	group: String,
	kind: String,
	name: String,
	aliases: String,
}

fn ranges(text: &str, property: Option<&str>) -> Result<Vec<Range>> {
	let mut result = Vec::new();
	for line in text.lines() {
		let line = line.split('#').next().unwrap().trim();
		if line.is_empty() {
			continue;
		}
		let (span, name) = line.split_once(';').context("Invalid Unicode range")?;
		let name = name.split(';').next().unwrap().trim();
		if property.is_some_and(|p| p != name) {
			continue;
		}
		let ends: Vec<_> = span.trim().split("..").collect();
		result.push(Range {
			start: u32::from_str_radix(ends[0], 16)?,
			end: u32::from_str_radix(ends[ends.len() - 1], 16)?,
			name: name.into(),
		});
	}
	Ok(result)
}

fn emojis(text: &str) -> Result<BTreeMap<u32, Emoji>> {
	let mut entries = BTreeMap::new();
	let mut group = String::new();
	for line in text.lines() {
		if let Some(name) = line.strip_prefix("# group: ") {
			group = name.into();
		}
		if line.is_empty() || line.starts_with('#') {
			continue;
		}
		let (data, description) = line.split_once('#').context("Emoji description missing")?;
		let (codes, status) = data.split_once(';').context("Emoji status missing")?;
		if status.trim() != "fully-qualified" || codes.split_whitespace().count() != 1 {
			continue;
		}
		let name = description
			.trim()
			.splitn(3, ' ')
			.nth(2)
			.context("Emoji name missing")?;
		entries.insert(
			u32::from_str_radix(codes.trim(), 16)?,
			Emoji {
				group: group.clone(),
				name: name.into(),
				order: entries.len(),
			},
		);
	}
	Ok(entries)
}

fn classify(name: &str, category: &str, block: &str) -> Option<(&'static str, &'static str)> {
	if block == "Box Drawing" {
		if (name.contains("LIGHT") && name.contains("HEAVY"))
			|| (name.contains("SINGLE") && name.contains("DOUBLE"))
		{
			return Some(("Box Drawing", "Mixed junctions"));
		}
		for (token, label) in [
			("ARC", "Rounded"),
			("DASH", "Dashed"),
			("DIAGONAL", "Diagonal"),
			("DOUBLE", "Double"),
			("HEAVY", "Heavy"),
			("LIGHT", "Light"),
		] {
			if name.contains(token) {
				return Some(("Box Drawing", label));
			}
		}
	}
	if block == "Block Elements" {
		for (token, label) in [
			("SHADE", "Shading"),
			("QUADRANT", "Quadrants"),
			("HALF", "Halves"),
			("FULL", "Full"),
		] {
			if name.contains(token) {
				return Some(("Blocks & Shading", label));
			}
		}
		return Some(("Blocks & Shading", "Fractions"));
	}
	if block == "Braille Patterns" {
		return Some(("Braille", "Patterns"));
	}
	if let Some((_, kind)) = LETTER_BLOCKS.iter().find(|(name, _)| *name == block) {
		return Some(match category {
			"Sc" => ("Typography", "Currency"),
			"Sm" => ("Mathematics", "Operators"),
			_ if category.starts_with('P') => ("Typography", "Punctuation"),
			_ if category.starts_with(['L', 'N']) => ("Letters & Numbers", kind),
			_ => ("Symbols", "General symbols"),
		});
	}
	if block == "CJK Symbols and Punctuation" && !category.starts_with('P') {
		return None;
	}
	DESIGN_BLOCKS
		.iter()
		.find(|(name, _)| *name == block)
		.map(|(_, pair)| {
			if name.contains("ARROW") || name.contains("HARPOON") {
				(
					"Arrows",
					if block == "Arrows" {
						"Standard"
					} else {
						"Supplemental"
					},
				)
			} else {
				*pair
			}
		})
}

fn generate(sources: &BTreeMap<String, String>) -> Result<String> {
	let blocks = ranges(&sources["Blocks.txt"], None)?;
	let emoji = emojis(&sources["emoji-test.txt"])?;
	let ignored: BTreeSet<_> = ranges(
		&sources["DerivedCoreProperties.txt"],
		Some("Default_Ignorable_Code_Point"),
	)?
	.into_iter()
	.flat_map(|r| r.start..=r.end)
	.collect();
	let mut aliases: BTreeMap<u32, Vec<String>> = BTreeMap::new();
	for line in sources["NameAliases.txt"]
		.lines()
		.filter(|l| !l.is_empty() && !l.starts_with('#'))
	{
		let fields: Vec<_> = line.split(';').collect();
		aliases
			.entry(u32::from_str_radix(fields[0], 16)?)
			.or_default()
			.push(fields[1].into());
	}
	let mut names = BTreeMap::new();
	for range in ranges(&sources["DerivedName.txt"], None)? {
		for cp in range.start..=range.end {
			names.insert(cp, range.name.replace('*', &format!("{cp:04X}")));
		}
	}
	let mut entries = BTreeMap::new();
	let mut block_at = 0;
	let mut first = None;
	for line in sources["UnicodeData.txt"].lines() {
		let fields: Vec<_> = line.split(';').collect();
		let code = u32::from_str_radix(fields[0], 16)?;
		let name = fields[1];
		let category = fields[2];
		if name.ends_with(", First>") {
			first = Some(code);
			continue;
		}
		let start = if name.ends_with(", Last>") {
			first.context("Unicode range end without start")?
		} else {
			code
		};
		first = None;
		if !category.starts_with(['L', 'N', 'P', 'S']) {
			continue;
		}
		for cp in start..=code {
			if ignored.contains(&cp)
				|| cp == 0x2800
				|| (0x1f1e6..=0x1f1ff).contains(&cp)
				|| (0x1f3fb..=0x1f3ff).contains(&cp)
			{
				continue;
			}
			while blocks[block_at].end < cp {
				block_at += 1;
			}
			let label = &names[&cp];
			let mut entry_aliases = aliases.get(&cp).cloned().unwrap_or_default();
			let (group, kind) = if let Some(emoji) = emoji.get(&cp) {
				entry_aliases.push(emoji.name.clone());
				("Emoji", emoji.group.as_str())
			} else if let Some(pair) = classify(label, category, &blocks[block_at].name) {
				pair
			} else {
				continue;
			};
			entries.insert(
				cp,
				Entry {
					group: group.into(),
					kind: kind.into(),
					name: label.clone(),
					aliases: entry_aliases.join("|"),
				},
			);
		}
	}
	let glyphs: BTreeMap<String, serde_json::Value> =
		serde_json::from_str(&sources["glyphnames.json"])?;
	for (alias, item) in glyphs {
		if alias == "METADATA" {
			continue;
		}
		let cp = u32::from_str_radix(item["code"].as_str().context("Missing icon code")?, 16)?;
		let prefix = alias.split('-').next().unwrap();
		let family = ICON_FAMILIES
			.iter()
			.find(|(name, _)| *name == prefix)
			.context("Unknown icon family")?
			.1;
		if let Some(entry) = entries.get_mut(&cp) {
			entry.aliases += &format!("|{alias}");
		} else {
			entries.insert(
				cp,
				Entry {
					group: "Nerd Icons".into(),
					kind: family.into(),
					name: alias,
					aliases: String::new(),
				},
			);
		}
	}
	let mut output = String::new();
	for (cp, entry) in entries {
		let mut row = format!(
			"{cp:X}\t{}\t{}\t{}\t{}",
			entry.group, entry.kind, entry.name, entry.aliases
		);
		if entry.group == "Emoji" {
			row += &format!("\t{}", 0x110000 + emoji[&cp].order);
		}
		output += row.trim_end_matches('\t');
		output.push('\n');
	}
	Ok(output)
}

pub fn run(runner: &Runner) -> Result<()> {
	let manifest: Manifest = serde_json::from_slice(&fs::read(
		runner.root.join("licenses/characters/sources.json"),
	)?)?;
	let client = reqwest::blocking::Client::builder()
		.timeout(Duration::from_secs(60))
		.build()?;
	let mut sources = BTreeMap::new();
	for (name, source) in manifest.sources {
		let bytes = client
			.get(&source.url)
			.send()?
			.error_for_status()?
			.bytes()?;
		ensure!(
			archive::digest_bytes(&bytes) == source.sha256,
			"Checksum mismatch: {}",
			source.url
		);
		sources.insert(name, String::from_utf8(bytes.to_vec())?);
	}
	let result = generate(&sources)?;
	fs::write(
		runner
			.root
			.join("src/libraries/characters/data/catalog.tsv"),
		result,
	)?;
	println!("Generated character catalog from pinned sources");
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn only_standalone_fully_qualified_emoji_are_admitted() -> Result<()> {
		let entries = emojis(
			"# group: Smileys & Emotion\n1F600 ; fully-qualified # 😀 E1.0 grinning face\n1F3FB ; component # 🏻 E1.0 light skin tone\n1F469 200D 1F4BB ; fully-qualified # 👩‍💻 E1.0 woman technologist\n1F601 ; unqualified # 😁 E1.0 excluded\n# group: Travel & Places\n1F680 ; fully-qualified # 🚀 E1.0 rocket\n",
		)?;
		assert_eq!(entries.len(), 2);
		assert_eq!(entries[&0x1f600].order, 0);
		assert_eq!(entries[&0x1f680].order, 1);
		assert_eq!(entries[&0x1f680].group, "Travel & Places");
		assert_eq!(entries[&0x1f680].name, "rocket");
		Ok(())
	}

	#[test]
	fn admission_classifies_design_families_without_admitting_every_script() {
		assert_eq!(
			classify("BOX DRAWINGS LIGHT AND HEAVY", "So", "Box Drawing"),
			Some(("Box Drawing", "Mixed junctions"))
		);
		assert_eq!(
			classify("UPWARDS ARROW", "So", "Miscellaneous Symbols and Arrows"),
			Some(("Arrows", "Supplemental"))
		);
		assert_eq!(
			classify("CJK LETTER", "Lo", "CJK Symbols and Punctuation"),
			None
		);
		assert_eq!(
			classify("CJK IDEOGRAPH", "Lo", "CJK Unified Ideographs"),
			None
		);
		assert_eq!(
			classify("QUARTER BLOCK", "So", "Block Elements"),
			Some(("Blocks & Shading", "Fractions"))
		);
	}

	#[test]
	fn property_ranges_preserve_endpoints_and_ignore_comments() -> Result<()> {
		let values = ranges(
			"0001..0003 ; Wanted # comment\n0004 ; Other\n# ignored\n0005 ; Wanted",
			Some("Wanted"),
		)?;
		assert_eq!(values.len(), 2);
		assert_eq!((values[0].start, values[0].end), (1, 3));
		assert_eq!((values[1].start, values[1].end), (5, 5));
		Ok(())
	}
}
