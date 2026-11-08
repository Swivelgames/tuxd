//! Offline, application-independent character discovery. IDs are Unicode scalars.
use std::sync::LazyLock;
use unicode_width::UnicodeWidthChar;

pub struct Character {
	pub glyph: char,
	pub group: &'static str,
	pub subgroup: &'static str,
	pub name: &'static str,
	pub aliases: &'static str,
	search: String,
	browse_order: u32,
}

pub struct Group {
	pub name: &'static str,
	pub subgroups: Vec<&'static str>,
}

pub struct Catalog {
	entries: Vec<Character>,
	pub groups: Vec<Group>,
}

pub static CATALOG: LazyLock<Catalog> = LazyLock::new(Catalog::bundled);

/// Catalog admission also checks the renderer's conservative display policy.
/// Metadata generation excludes controls, marks, separators, unassigned scalars,
/// default-ignorables and emoji components. Font coverage is not detectable here.
pub fn display_supported(glyph: char) -> bool {
	let text = glyph.to_string();
	matches!(glyph.width(), Some(1 | 2)) && super::text::display::display_grapheme(&text) == text
}

fn normalize(text: &str) -> String {
	text.to_lowercase().replace(['_', '-', '|'], " ")
}

impl Catalog {
	fn bundled() -> Self {
		let entries: Vec<_> = include_str!("data/catalog.tsv")
			.lines()
			.filter_map(|line| {
				let mut fields = line.split('\t');
				let glyph = char::from_u32(u32::from_str_radix(fields.next()?, 16).ok()?)?;
				let group = fields.next()?;
				let subgroup = fields.next()?;
				let name = fields.next()?;
				let aliases = fields.next().unwrap_or("");
				let browse_order = fields
					.next()
					.map(|value| value.parse().expect("generated browse order"))
					.unwrap_or(glyph as u32);
				display_supported(glyph).then(|| Character {
					glyph,
					group,
					subgroup,
					name,
					aliases,
					browse_order,
					search: normalize(&format!("{name} {aliases} {group} {subgroup}")),
				})
			})
			.collect();
		let groups = [
			"Box Drawing",
			"Blocks & Shading",
			"Arrows",
			"Shapes",
			"Symbols",
			"Mathematics",
			"Typography",
			"Letters & Numbers",
			"Emoji",
			"Braille",
			"Nerd Icons",
		]
		.into_iter()
		.map(|name| {
			let mut subgroups: Vec<_> = entries
				.iter()
				.filter(|e| e.group == name)
				.map(|e| e.subgroup)
				.collect();
			subgroups.sort_unstable();
			subgroups.dedup();
			if name == "Emoji" {
				subgroups.sort_by_key(|subgroup| {
					entries
						.iter()
						.filter(|entry| entry.group == name && entry.subgroup == *subgroup)
						.map(|entry| entry.browse_order)
						.min()
						.unwrap()
				});
			}
			Group { name, subgroups }
		})
		.collect();
		Self { entries, groups }
	}

	pub fn get(&self, glyph: char) -> Option<&Character> {
		self.entries
			.binary_search_by_key(&glyph, |e| e.glyph)
			.ok()
			.map(|i| &self.entries[i])
	}

	/// Empty queries browse a group or optional subgroup. Nonempty queries search globally,
	/// or within an explicitly supplied group. Exact glyph/code point wins.
	pub fn search(&self, query: &str, group: Option<&str>, subgroup: Option<&str>) -> Vec<char> {
		let query = query.trim();
		let normalized = normalize(query);
		let terms: Vec<_> = normalized.split_whitespace().collect();
		let hex = query
			.strip_prefix("U+")
			.or_else(|| query.strip_prefix("u+"))
			.or_else(|| query.strip_prefix("0x"));
		let exact = hex
			.and_then(|hex| u32::from_str_radix(hex, 16).ok())
			.and_then(char::from_u32)
			.or_else(|| {
				(query.chars().count() == 1)
					.then(|| query.chars().next())
					.flatten()
			});
		let mut matches = Vec::new();
		for entry in &self.entries {
			if group.is_some_and(|g| entry.group != g)
				|| subgroup.is_some_and(|s| entry.subgroup != s)
			{
				continue;
			}
			let rank = if exact == Some(entry.glyph) {
				0
			} else if hex.is_some() {
				continue;
			} else if terms.iter().all(|term| entry.search.contains(term)) {
				1
			} else {
				continue;
			};
			matches.push((rank, entry.browse_order, entry.glyph));
		}
		matches.sort_unstable();
		matches.into_iter().map(|(_, _, glyph)| glyph).collect()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn catalog_is_grouped_unique_and_safe() {
		assert!((10_000..25_000).contains(&CATALOG.entries.len()));
		assert!(CATALOG.entries.windows(2).all(|e| e[0].glyph < e[1].glyph));
		assert!(CATALOG.groups.iter().all(|g| !g.subgroups.is_empty()));
		for glyph in [
			'\0',
			'\n',
			'\u{301}',
			'\u{200d}',
			'\u{202e}',
			'\u{1f1fa}',
			'\u{1f3fd}',
			'\u{378}',
		] {
			assert!(CATALOG.get(glyph).is_none(), "{glyph:?}");
		}
		assert_eq!(
			CATALOG
				.entries
				.iter()
				.filter(|e| e.group == "Box Drawing")
				.count(),
			128
		);
	}
	#[test]
	fn discovery_is_curated_for_design_and_emoji_are_standalone() {
		for glyph in [
			'─',
			'█',
			'→',
			'✓',
			'⌘',
			'♜',
			'∑',
			'α',
			'é',
			'①',
			'€',
			'⠿',
			'\u{1fb00}',
		] {
			assert!(CATALOG.get(glyph).is_some(), "missing useful glyph {glyph}");
		}
		for glyph in [
			'界',
			'\u{13000}',
			'\u{12000}',
			'\u{2800}',
			'\u{1f3fd}',
			'\u{1f9b0}',
		] {
			assert!(
				CATALOG.get(glyph).is_none(),
				"unexpected discovery entry {glyph}"
			);
		}
		for glyph in ['😀', '🚀', '📁', '✅'] {
			assert_eq!(CATALOG.get(glyph).unwrap().group, "Emoji");
		}
		assert!(
			CATALOG
				.search("grinning face", Some("Emoji"), None)
				.contains(&'😀')
		);
		let emoji = CATALOG
			.groups
			.iter()
			.find(|group| group.name == "Emoji")
			.unwrap();
		assert!(emoji.subgroups.contains(&"Smileys & Emotion"));
		assert!(emoji.subgroups.contains(&"Objects"));
		assert!(
			CATALOG
				.groups
				.iter()
				.all(|group| group.subgroups.len() < 25)
		);
	}

	#[test]
	fn emoji_follow_picker_order_while_exact_search_stays_first() {
		let glyphs = CATALOG.search("", Some("Emoji"), None);
		assert_eq!(&glyphs[..6], &['😀', '😃', '😄', '😁', '😆', '😅']);
		let mut categories = Vec::new();
		for glyph in glyphs {
			let category = CATALOG.get(glyph).unwrap().subgroup;
			if categories.last() != Some(&category) {
				categories.push(category);
			}
		}
		assert_eq!(
			categories,
			vec![
				"Smileys & Emotion",
				"People & Body",
				"Animals & Nature",
				"Food & Drink",
				"Travel & Places",
				"Activities",
				"Objects",
				"Symbols",
				"Flags"
			]
		);
		assert_eq!(
			CATALOG
				.groups
				.iter()
				.find(|group| group.name == "Emoji")
				.unwrap()
				.subgroups,
			categories
		);
		assert_eq!(CATALOG.search("😀", Some("Emoji"), None)[0], '😀');
		assert_eq!(CATALOG.search("U+1F680", None, None), vec!['🚀']);
		let faces = CATALOG.search("face", Some("Emoji"), None);
		assert_eq!(&faces[..3], &['😀', '😃', '😄']);
	}

	#[test]
	fn search_resolves_names_aliases_glyphs_and_codepoints() {
		for query in ["─", "U+2500", "0x2500", "box light horizontal"] {
			assert!(CATALOG.search(query, None, None).contains(&'─'));
		}
		assert_eq!(CATALOG.search("U+2500", None, None), vec!['─']);
		assert!(CATALOG.search("nf-fa-home", None, None).is_empty());
		assert!(
			!CATALOG
				.search("fa home", Some("Nerd Icons"), None)
				.is_empty()
		);
		assert!(CATALOG.search("U+2500", Some("Braille"), None).is_empty());
	}
}
