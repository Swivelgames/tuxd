//! Text transport and owned requests. No ANSI, selection, focus, or UI messages.
mod process;
mod system;

use std::{
	io::Write,
	sync::{
		Arc,
		atomic::{AtomicBool, AtomicU64, Ordering},
		mpsc,
	},
};
pub use system::System;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transport {
	Desktop,
	Tmux,
	Osc52,
}

/// Availability is an environment hint, not a guarantee that a helper succeeds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capabilities {
	pub write: Option<Transport>,
	pub read: bool,
}

/// Only Desktop acknowledges helper completion. Forwarded delivery is unconfirmed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Receipt {
	Desktop,
	Tmux,
	Osc52,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
	UnsupportedTerminal,
	PayloadTooLarge,
	TmuxWrite,
	DesktopWrite,
	DesktopRead,
	WorkerDisconnected,
	TerminalOutput(String),
}

pub enum WriteOutput {
	Delivered(Receipt),
	Terminal(Vec<u8>),
}

/// Drivers run on workers. Terminal output must be returned for caller-thread emission.
pub trait Driver: Send + Sync + 'static {
	fn capabilities(&self) -> Capabilities;
	fn write(&self, text: &str) -> Result<WriteOutput, Error>;
	fn read(&self) -> Result<String, Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestId(u64);

pub struct Completion<T> {
	pub id: RequestId,
	pub result: Result<T, Error>,
}

struct Pending<T> {
	id: RequestId,
	receiver: mpsc::Receiver<Result<T, Error>>,
	completed: bool,
}

impl<T: Send + 'static> Pending<T> {
	fn spawn(work: impl FnOnce() -> Result<T, Error> + Send + 'static) -> Self {
		static NEXT: AtomicU64 = AtomicU64::new(1);
		let id = RequestId(NEXT.fetch_add(1, Ordering::Relaxed));
		let (sender, receiver) = mpsc::channel();
		std::thread::spawn(move || {
			let _ = sender.send(work());
		});
		Self {
			id,
			receiver,
			completed: false,
		}
	}

	fn poll(&mut self) -> Option<Completion<T>> {
		if self.completed {
			return None;
		}
		let result = match self.receiver.try_recv() {
			Ok(result) => result,
			Err(mpsc::TryRecvError::Empty) => return None,
			Err(mpsc::TryRecvError::Disconnected) => Err(Error::WorkerDisconnected),
		};
		self.completed = true;
		Some(Completion {
			id: self.id,
			result,
		})
	}
}

/// Dropping a request abandons its result; it cannot undo an external write.
/// The worker and owner jointly hold the permit so cancellation cannot overlap writes.
struct WritePermit(Arc<AtomicBool>);
impl Drop for WritePermit {
	fn drop(&mut self) {
		self.0.store(false, Ordering::Release);
	}
}

pub struct WriteRequest {
	pending: Pending<WriteOutput>,
	permit: Option<Arc<WritePermit>>,
}

impl WriteRequest {
	pub fn id(&self) -> RequestId {
		self.pending.id
	}

	/// Call between frames on the terminal-owning thread. Finalization is one-shot,
	/// including failed output; no worker can interleave OSC bytes with a frame.
	pub fn poll(&mut self, output: &mut impl Write) -> Option<Completion<Receipt>> {
		let completion = self.pending.poll()?;
		let result = completion.result.and_then(|prepared| match prepared {
			WriteOutput::Delivered(receipt) => Ok(receipt),
			WriteOutput::Terminal(bytes) => {
				output
					.write_all(&bytes)
					.and_then(|_| output.flush())
					.map_err(|error| Error::TerminalOutput(error.to_string()))?;
				Ok(Receipt::Osc52)
			}
		});
		self.permit = None;
		Some(Completion {
			id: completion.id,
			result,
		})
	}
}

pub struct ReadRequest(Pending<String>);
impl ReadRequest {
	pub fn id(&self) -> RequestId {
		self.0.id
	}

	pub fn poll(&mut self) -> Option<Completion<String>> {
		self.0.poll()
	}
}

/// Clones share a write lane, preserving the existing one-copy-at-a-time behavior.
/// Each consumer retains its request; completion never routes by current focus.
#[derive(Clone)]
pub struct Service {
	driver: Arc<dyn Driver>,
	writing: Arc<AtomicBool>,
}

impl Default for Service {
	fn default() -> Self {
		Self::with_driver(System)
	}
}

impl Service {
	pub fn with_driver(driver: impl Driver) -> Self {
		Self {
			driver: Arc::new(driver),
			writing: Arc::new(AtomicBool::new(false)),
		}
	}

	pub fn capabilities(&self) -> Capabilities {
		self.driver.capabilities()
	}

	/// None means the shared write lane is already occupied; no work was started.
	pub fn write(&self, text: String) -> Option<WriteRequest> {
		self.writing
			.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
			.ok()?;
		let permit = Arc::new(WritePermit(self.writing.clone()));
		let worker_permit = permit.clone();
		let driver = self.driver.clone();
		let pending = Pending::spawn(move || {
			let _permit = worker_permit;
			driver.write(&text)
		});
		Some(WriteRequest {
			pending,
			permit: Some(permit),
		})
	}

	pub fn read(&self) -> ReadRequest {
		let driver = self.driver.clone();
		ReadRequest(Pending::spawn(move || driver.read()))
	}
}

#[cfg(test)]
mod tests;
