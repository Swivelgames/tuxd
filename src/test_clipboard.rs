//! Controlled clipboard worker boundary for application tests; no external clipboard.
use crate::libraries::clipboard::*;
use std::sync::{
	Mutex,
	mpsc::{self, Receiver, Sender},
};

pub struct Controls {
	pub writes: Sender<Result<WriteOutput, Error>>,
	pub reads: Sender<Result<String, Error>>,
	pub payloads: Receiver<String>,
}
struct Controlled {
	writes: Mutex<Receiver<Result<WriteOutput, Error>>>,
	reads: Mutex<Receiver<Result<String, Error>>>,
	payloads: Sender<String>,
}
impl Driver for Controlled {
	fn capabilities(&self) -> Capabilities {
		Capabilities {
			write: Some(Transport::Desktop),
			read: true,
		}
	}
	fn write(&self, text: &str) -> Result<WriteOutput, Error> {
		let _ = self.payloads.send(text.into());
		self.writes
			.lock()
			.unwrap()
			.recv()
			.unwrap_or(Err(Error::WorkerDisconnected))
	}
	fn read(&self) -> Result<String, Error> {
		self.reads
			.lock()
			.unwrap()
			.recv()
			.unwrap_or(Err(Error::WorkerDisconnected))
	}
}
pub fn controlled() -> (Service, Controls) {
	let (write_tx, writes) = mpsc::channel();
	let (read_tx, reads) = mpsc::channel();
	let (payloads, payload_rx) = mpsc::channel();
	(
		Service::with_driver(Controlled {
			writes: Mutex::new(writes),
			reads: Mutex::new(reads),
			payloads,
		}),
		Controls {
			writes: write_tx,
			reads: read_tx,
			payloads: payload_rx,
		},
	)
}
pub fn until(mut complete: impl FnMut() -> bool) {
	let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
	while !complete() {
		assert!(
			std::time::Instant::now() < deadline,
			"Clipboard completion timed out"
		);
		std::thread::yield_now();
	}
}
