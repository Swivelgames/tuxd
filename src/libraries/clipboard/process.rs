//! Bounded helper processes; no application or terminal rendering policy.
use std::{
	io::Write,
	process::{Child, Command, Stdio},
	thread,
	time::{Duration, Instant},
};
pub(super) fn copy_with(program: &str, args: &[&str], text: &str) -> Result<(), ()> {
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
	// On timeout, do not wait on a pipe that an ill-behaved descendant may still hold.
	wait_for_success(&mut child, deadline)?;
	written
		.recv_timeout(deadline.saturating_duration_since(Instant::now()))
		.map_err(|_| ())?
		.map_err(|_| ())
}

pub(super) fn paste_with(program: &str, args: &[&str]) -> Result<String, ()> {
	use std::io::Read;
	const LIMIT: usize = 64 * 1024;
	let mut child = Command::new(program)
		.args(args)
		.stdin(Stdio::null())
		.stdout(Stdio::piped())
		.stderr(Stdio::null())
		.spawn()
		.map_err(|_| ())?;
	let stdout = child.stdout.take().ok_or(())?;
	let (sender, receiver) = std::sync::mpsc::channel();
	thread::spawn(move || {
		let mut bytes = Vec::new();
		let result = stdout
			.take((LIMIT + 1) as u64)
			.read_to_end(&mut bytes)
			.map_err(|_| ())
			.and_then(|_| {
				if bytes.len() > LIMIT {
					Err(())
				} else {
					String::from_utf8(bytes).map_err(|_| ())
				}
			});
		let _ = sender.send(result);
	});
	let deadline = Instant::now() + Duration::from_secs(3);
	wait_for_success(&mut child, deadline)?;
	receiver
		.recv_timeout(deadline.saturating_duration_since(Instant::now()))
		.map_err(|_| ())?
}

/// Reap failed or timed-out helpers before returning; pipe workers share this deadline.
fn wait_for_success(child: &mut Child, deadline: Instant) -> Result<(), ()> {
	loop {
		match child.try_wait() {
			Ok(Some(status)) => return if status.success() { Ok(()) } else { Err(()) },
			Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
			_ => {
				let _ = child.kill();
				let _ = child.wait();
				return Err(());
			}
		}
	}
}
