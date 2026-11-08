use super::*;
use std::{
	sync::Mutex,
	time::{Duration, Instant},
};

struct Controlled {
	results: Mutex<mpsc::Receiver<Result<WriteOutput, Error>>>,
	started: mpsc::Sender<String>,
}

impl Driver for Controlled {
	fn capabilities(&self) -> Capabilities {
		Capabilities {
			write: Some(Transport::Osc52),
			read: false,
		}
	}

	fn write(&self, text: &str) -> Result<WriteOutput, Error> {
		let _ = self.started.send(text.to_owned());
		self.results
			.lock()
			.unwrap()
			.recv()
			.unwrap_or(Err(Error::WorkerDisconnected))
	}

	fn read(&self) -> Result<String, Error> {
		Err(Error::DesktopRead)
	}
}

fn until<T>(mut poll: impl FnMut() -> Option<T>) -> T {
	let deadline = Instant::now() + Duration::from_secs(3);
	loop {
		if let Some(result) = poll() {
			return result;
		}
		assert!(Instant::now() < deadline, "completion timed out");
		std::thread::yield_now();
	}
}

fn service() -> (
	Service,
	mpsc::Sender<Result<WriteOutput, Error>>,
	mpsc::Receiver<String>,
) {
	let (sender, results) = mpsc::channel();
	let (started, receiver) = mpsc::channel();
	(
		Service::with_driver(Controlled {
			results: Mutex::new(results),
			started,
		}),
		sender,
		receiver,
	)
}

#[test]
fn terminal_output_is_finalized_once_by_owner_and_busy_requests_do_not_start() {
	let (service, sender, started) = service();
	assert_eq!(
		service.capabilities(),
		Capabilities {
			write: Some(Transport::Osc52),
			read: false
		}
	);
	let mut request = service.write("exact text".into()).unwrap();
	assert_eq!(
		started.recv_timeout(Duration::from_secs(1)).unwrap(),
		"exact text"
	);
	let id = request.id();
	assert!(service.clone().write("must not start".into()).is_none());
	let mut output = Vec::new();
	assert!(request.poll(&mut output).is_none());
	assert!(output.is_empty());
	sender
		.send(Ok(WriteOutput::Terminal(b"prepared OSC".to_vec())))
		.unwrap();
	let completed = until(|| request.poll(&mut output));
	assert_eq!(completed.id, id);
	assert_eq!(completed.result, Ok(Receipt::Osc52));
	assert_eq!(output, b"prepared OSC");
	assert!(request.poll(&mut output).is_none());
	assert!(service.write("next".into()).is_some());
}

#[test]
fn cancellation_retains_the_write_lane_until_worker_finishes_and_discards_terminal_output() {
	let (service, sender, started) = service();
	let request = service.write("old".into()).unwrap();
	let old_id = request.id();
	started.recv_timeout(Duration::from_secs(1)).unwrap();
	drop(request);
	assert!(service.write("blocked".into()).is_none());
	sender
		.send(Ok(WriteOutput::Terminal(b"abandoned".to_vec())))
		.unwrap();
	let mut next = until(|| service.write("new".into()));
	assert_ne!(old_id, next.id());
	sender
		.send(Ok(WriteOutput::Delivered(Receipt::Desktop)))
		.unwrap();
	let mut output = Vec::new();
	assert_eq!(
		until(|| next.poll(&mut output)).result,
		Ok(Receipt::Desktop)
	);
	assert!(output.is_empty());
}

#[test]
fn failures_and_reads_are_typed_and_do_not_replay() {
	let (service, sender, _) = service();
	let mut read = service.read();
	let id = read.id();
	let completion = until(|| read.poll());
	assert_eq!(completion.id, id);
	assert_eq!(completion.result, Err(Error::DesktopRead));
	assert!(read.poll().is_none());

	// Keep the driver's started receiver alive while exercising output failure.
	let (service, sender2, _started) = self::service();
	let mut request = service.write("x".into()).unwrap();
	sender2.send(Ok(WriteOutput::Terminal(vec![1]))).unwrap();
	struct Broken;
	impl Write for Broken {
		fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
			Err(std::io::Error::other("closed"))
		}

		fn flush(&mut self) -> std::io::Result<()> {
			Ok(())
		}
	}
	assert_eq!(
		until(|| request.poll(&mut Broken)).result,
		Err(Error::TerminalOutput("closed".into()))
	);
	assert!(request.poll(&mut Broken).is_none());
	drop(sender);
}

#[test]
fn disconnected_worker_is_reported_once() {
	let (sender, receiver) = mpsc::channel::<Result<String, Error>>();
	drop(sender);
	let mut pending = Pending {
		id: RequestId(1),
		receiver,
		completed: false,
	};
	assert_eq!(
		pending.poll().unwrap().result,
		Err(Error::WorkerDisconnected)
	);
	assert!(pending.poll().is_none());
}
