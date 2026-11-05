use std::{
	io::Write,
	process::{Command, Stdio},
	thread,
	time::{Duration, Instant},
};

/// The desktop helper retains clipboard ownership after the editor exits.
pub fn copy(text: &str) -> Result<(), String> {
	let mut helpers: Vec<(&str, &[&str])> = Vec::new();
	if std::env::var_os("WAYLAND_DISPLAY").is_some() {
		helpers.push(("wl-copy", &["--type", "text/plain;charset=utf-8"]));
	}
	if std::env::var_os("DISPLAY").is_some() {
		helpers.push(("xclip", &["-selection", "clipboard", "-in"]));
		helpers.push(("xsel", &["--clipboard", "--input"]));
	}
	for (program, args) in helpers {
		if copy_with(program, args, text).is_ok() {
			return Ok(());
		}
	}
	Err(
		"Copy failed: use wl-copy on Wayland or xclip/xsel on X11; check your display connection."
			.into(),
	)
}

fn copy_with(program: &str, args: &[&str], text: &str) -> Result<(), ()> {
	let mut child = Command::new(program)
		.args(args)
		.stdin(Stdio::piped())
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.spawn()
		.map_err(|_| ())?;
	let mut stdin = child.stdin.take().ok_or(())?;
	let bytes = text.as_bytes().to_vec();
	let (sent, written) = std::sync::mpsc::channel();
	thread::spawn(move || {
		let _ = sent.send(stdin.write_all(&bytes));
	});
	let deadline = Instant::now() + Duration::from_secs(3);
	let result = loop {
		match child.try_wait() {
			Ok(Some(status)) => break if status.success() { Ok(()) } else { Err(()) },
			Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
			_ => {
				let _ = child.kill();
				let _ = child.wait();
				break Err(());
			}
		}
	};
	// On timeout, do not wait on a pipe that an ill-behaved descendant may still hold.
	result?;
	written
		.recv_timeout(deadline.saturating_duration_since(Instant::now()))
		.map_err(|_| ())?
		.map_err(|_| ())
}
