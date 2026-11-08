//! Versioned .tuxd.json codec. File access is owned by persistence.
use crate::libraries::ansi::{Document, DocumentData, Styles};
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileDocument {
	version: u32,
	text: String,
	styles: Styles,
}

pub fn encode(document: &Document) -> io::Result<Vec<u8>> {
	let data = document.to_data();
	let file = FileDocument {
		version: 1,
		text: data.text,
		styles: data.styles,
	};
	let mut bytes = Vec::new();
	let formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");
	file.serialize(&mut serde_json::Serializer::with_formatter(
		&mut bytes, formatter,
	))
	.map_err(io::Error::other)?;
	bytes.push(b'\n');
	Ok(bytes)
}

pub fn decode(bytes: &[u8]) -> io::Result<Document> {
	let file: FileDocument = serde_json::from_slice(bytes).map_err(io::Error::other)?;
	if file.version != 1 {
		return Err(io::Error::new(
			io::ErrorKind::InvalidData,
			"Unsupported document version",
		));
	}
	Document::try_from(DocumentData {
		text: file.text,
		styles: file.styles,
	})
	.map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::libraries::ansi::{Ink, Position, TextStyle};

	#[test]
	fn native_bytes_keep_the_existing_versioned_envelope() {
		let document = Document::from_text("x", TextStyle::default()).unwrap();
		let expected = concat!(
			"{\n\t\"version\": 1,\n\t\"text\": \"x\",\n\t\"styles\": {\n\t\t\"runs\": [\n",
			"\t\t\t{\n\t\t\t\t\"start\": 0,\n\t\t\t\t\"end\": 1,\n\t\t\t\t\"style\": {\n",
			"\t\t\t\t\t\"bold\": false,\n\t\t\t\t\t\"italic\": false,\n",
			"\t\t\t\t\t\"underline\": false,\n\t\t\t\t\t\"dim\": false,\n",
			"\t\t\t\t\t\"strike\": false,\n\t\t\t\t\t\"foreground\": \"Default\",\n",
			"\t\t\t\t\t\"background\": \"Default\"\n\t\t\t\t}\n\t\t\t}\n\t\t]\n\t}\n}\n",
		);
		assert_eq!(encode(&document).unwrap(), expected.as_bytes());
		assert_eq!(decode(expected.as_bytes()).unwrap(), document);
	}

	#[test]
	fn included_example_is_a_valid_styled_document() {
		let document = decode(include_bytes!("../../examples/welcome.tuxd.json")).unwrap();
		assert!(document.style_runs().iter().any(|run| run.style.bold));
		assert_eq!(decode(&encode(&document).unwrap()).unwrap(), document);
	}

	#[test]
	fn roundtrip_preserves_unicode_and_all_color_kinds() {
		for ink in [
			Ink::Default,
			Ink::Ansi(15),
			Ink::Indexed(255),
			Ink::Rgb(0, 128, 255),
		] {
			let mut document = Document::default();
			document
				.insert(Position::default(), "e\u{301}\t界\n👩‍💻\n")
				.unwrap();
			document
				.set_style(
					Position::default()..document.end(),
					TextStyle {
						bold: true,
						dim: true,
						italic: true,
						underline: true,
						strike: true,
						foreground: ink,
						background: ink,
					},
				)
				.unwrap();
			assert_eq!(decode(&encode(&document).unwrap()).unwrap(), document);
		}
	}
	#[test]
	fn rejects_invalid_version_ranges_controls_and_colors() {
		let mut doc = Document::default();
		doc.insert(Position::default(), "e\u{301}").unwrap();
		let original: serde_json::Value = serde_json::from_slice(&encode(&doc).unwrap()).unwrap();
		for (pointer, value) in [
			("/version", serde_json::json!(2)),
			("/styles/runs/0/end", serde_json::json!(1)),
			("/text", serde_json::json!("\u{1b}")),
			(
				"/styles/runs/0/style/foreground",
				serde_json::json!({"Ansi":16}),
			),
		] {
			let mut file = original.clone();
			*file.pointer_mut(pointer).unwrap() = value;
			assert!(decode(&serde_json::to_vec(&file).unwrap()).is_err());
		}
	}
}
