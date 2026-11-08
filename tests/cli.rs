//! Informational commands must work without an interactive terminal.
use std::process::{Command, Stdio};

#[test]
fn version_is_stable_and_does_not_enter_terminal_mode() {
	for flag in ["--version", "-V"] {
		let output = Command::new(env!("CARGO_BIN_EXE_tuxd"))
			.arg(flag)
			.stdin(Stdio::null())
			.env("TERM", "dumb")
			.output()
			.unwrap();

		assert!(output.status.success());
		assert_eq!(
			output.stdout,
			format!("tuxd {}\n", env!("CARGO_PKG_VERSION")).as_bytes()
		);
		assert!(output.stderr.is_empty());
	}
}

#[test]
fn help_is_versioned_and_works_without_a_terminal() {
	for flag in ["--help", "-h"] {
		let output = Command::new(env!("CARGO_BIN_EXE_tuxd"))
			.arg(flag)
			.stdin(Stdio::null())
			.env("TERM", "dumb")
			.output()
			.unwrap();

		assert!(output.status.success());
		assert!(output.stderr.is_empty());
		let help = String::from_utf8(output.stdout).unwrap();
		assert!(help.starts_with(&format!("tuxd {}\n", env!("CARGO_PKG_VERSION"))));
		for option in ["--raw", "--escaped", "--help", "--version"] {
			assert!(help.contains(option), "missing {option}");
		}
		assert!(help.contains("tuxd [DOCUMENT]"));
		assert!(help.contains(r"\x1b"));
		assert!(!help.contains('\u{1b}'));
	}
}
