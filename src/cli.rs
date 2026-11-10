//! Explicit startup sources; stdin is consumed before terminal event handling.
use crate::libraries::ansi::Mode;
use std::{
	ffi::OsString,
	io::{self, IsTerminal, Read},
	path::PathBuf,
};

pub const HELP: &str = concat!(
	"tuxd ",
	env!("CARGO_PKG_VERSION"),
	r#"
Design styled terminal text with live ANSI output.

Usage:
    tuxd [DOCUMENT | DIRECTORY]
    tuxd --raw INPUT
    tuxd --escaped INPUT
    tuxd --help
    tuxd --version

    With no DOCUMENT or DIRECTORY, open the current working directory.

Options:
        --raw INPUT    Import text containing actual ANSI escape bytes
        --escaped INPUT
                       Import text containing escaped ANSI notation (\x1b, \n, \t, \\)
    -h, --help         Print this help and exit
    -V, --version      Print the version and exit

Examples:
    tuxd
    tuxd greeting.tuxd.json
    tuxd ./my-project
    tuxd --raw output.ansi
    tuxd --escaped output.txt
    printf '\033[31mRed\033[0m\n' | tuxd --raw -
    printf '%s' '\x1b[31mRed\x1b[0m\n' | tuxd --escaped -

"#
);

pub enum Command {
	Edit(PathBuf),
	Import { mode: Mode, source: PathBuf },
	Help,
	Version,
	Diagnostic(bool),
}

pub fn parse(args: Vec<OsString>) -> io::Result<Command> {
	let invalid = || {
		io::Error::other(
			"Usage: tuxd [DOCUMENT | DIRECTORY] | --raw INPUT | --escaped INPUT (use - for stdin)",
		)
	};
	match args.as_slice() {
		[] => Ok(Command::Edit(std::env::current_dir()?)),
		[arg] if arg == "--version" || arg == "-V" => Ok(Command::Version),
		[arg] if arg == "--help" || arg == "-h" => Ok(Command::Help),
		[arg] if arg == "--diagnostic-error" => Ok(Command::Diagnostic(false)),
		[arg] if arg == "--diagnostic-panic" => Ok(Command::Diagnostic(true)),
		[arg] if !arg.as_encoded_bytes().starts_with(b"-") => Ok(Command::Edit(arg.into())),
		[mode, source] if mode == "--raw" || mode == "--escaped" => {
			if source != "-" && source.as_encoded_bytes().starts_with(b"-") {
				return Err(invalid());
			}
			Ok(Command::Import {
				mode: if mode == "--raw" {
					Mode::Raw
				} else {
					Mode::Escaped
				},
				source: source.into(),
			})
		}
		_ => Err(invalid()),
	}
}

pub fn read_stdin() -> io::Result<Vec<u8>> {
	use std::os::fd::AsRawFd;
	if io::stdin().is_terminal() {
		return Err(io::Error::other(
			"Expected piped or redirected stdin for '-'; supply a file path or pipe input",
		));
	}
	if !io::stdout().is_terminal() {
		return Err(io::Error::other(
			"Interactive output requires a terminal on stdout",
		));
	}
	let tty = std::fs::OpenOptions::new()
		.read(true)
		.write(true)
		.open("/dev/tty")
		.map_err(|e| {
			io::Error::other(format!("Stdin import requires a controlling terminal: {e}"))
		})?;
	let bytes = read_bounded(io::stdin().lock())?;
	// All stdin data is consumed and its lock dropped before fd 0 becomes the
	// controlling terminal. Crossterm then reads keys from the terminal, not EOF.
	if unsafe { libc::dup2(tty.as_raw_fd(), libc::STDIN_FILENO) } < 0 {
		return Err(io::Error::last_os_error());
	}
	Ok(bytes)
}

fn read_bounded(reader: impl Read) -> io::Result<Vec<u8>> {
	crate::libraries::storage::ReadPolicy {
		limit: 16 * 1024 * 1024,
		overflow: "Stdin import exceeds 16 MiB",
		special_file: "",
		error_kind: io::ErrorKind::Other,
	}
	.read(reader)
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn no_arguments_open_the_current_directory() {
		assert!(matches!(
			parse(vec![]).unwrap(),
			Command::Edit(path) if path == std::env::current_dir().unwrap()
		));
	}

	#[test]
	fn explicit_source_grammar() {
		for args in [
			vec!["-"],
			vec!["--raw"],
			vec!["--raw", "--escaped"],
			vec!["--raw", "-", "extra"],
		] {
			assert!(parse(args.into_iter().map(Into::into).collect()).is_err());
		}
		assert!(matches!(
			parse(vec!["--raw".into(), "-".into()]).unwrap(),
			Command::Import {
				mode: Mode::Raw,
				..
			}
		));
		assert!(matches!(
			parse(vec!["--escaped".into(), "sample.txt".into()]).unwrap(),
			Command::Import {
				mode: Mode::Escaped,
				..
			}
		));
	}
	#[test]
	fn informational_flags() {
		for flag in ["--version", "-V"] {
			assert!(matches!(
				parse(vec![flag.into()]).unwrap(),
				Command::Version
			));
		}
		assert!(parse(vec!["--version".into(), "extra".into()]).is_err());
	}

	#[test]
	fn input_limit_and_empty_eof() {
		assert!(read_bounded(&b""[..]).unwrap().is_empty());
		assert!(read_bounded(io::repeat(b'a').take(16 * 1024 * 1024 + 1)).is_err());
		assert_eq!(read_bounded(&b"hello"[..]).unwrap(), b"hello");
	}
}
