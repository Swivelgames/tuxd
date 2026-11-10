//! Collapse queued pointer positions within a bounded read window, preserving event boundaries.
use crossterm::event::{self, Event, MouseEventKind};
use std::{
	io,
	time::{Duration, Instant},
};

// Bound both queue work and time: continuous motion must still yield to drawing.
const MAX_DRAGS: usize = 1024;
const DRAIN_BUDGET: Duration = Duration::from_millis(2);

#[derive(Default)]
pub struct Input {
	pending: Option<Event>,
}

impl Input {
	pub fn poll(&self, timeout: Duration) -> io::Result<bool> {
		if self.pending.is_some() {
			Ok(true)
		} else {
			event::poll(timeout)
		}
	}

	pub fn read(&mut self) -> io::Result<Event> {
		let first = match self.pending.take() {
			Some(event) => event,
			None => event::read()?,
		};
		// Crossterm's poll-based Unix backend requires a positive timeout even
		// for buffered input. An empty queue costs at most this 1 ms lookahead.
		let deadline = Instant::now() + DRAIN_BUDGET;
		self.coalesce(first, || {
			if Instant::now() >= deadline || !event::poll(Duration::from_millis(1))? {
				return Ok(None);
			}
			event::read().map(Some)
		})
	}

	fn coalesce(
		&mut self,
		mut latest: Event,
		mut ready: impl FnMut() -> io::Result<Option<Event>>,
	) -> io::Result<Event> {
		let Event::Mouse(mouse) = &latest else {
			return Ok(latest);
		};
		if !matches!(mouse.kind, MouseEventKind::Drag(_)) {
			return Ok(latest);
		}
		let (kind, modifiers) = (mouse.kind, mouse.modifiers);

		for _ in 1..MAX_DRAGS {
			let Some(next) = ready()? else {
				break;
			};
			if matches!(&next, Event::Mouse(mouse) if mouse.kind == kind && mouse.modifiers == modifiers)
			{
				latest = next;
			} else {
				// Keep the boundary for the next turn. In particular, apply the final
				// position before its release, and redraw before using new geometry.
				self.pending = Some(next);
				break;
			}
		}
		Ok(latest)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent};
	use std::collections::VecDeque;

	fn drag(row: u16) -> Event {
		Event::Mouse(MouseEvent {
			kind: MouseEventKind::Drag(MouseButton::Left),
			column: 8,
			row,
			modifiers: KeyModifiers::NONE,
		})
	}

	#[test]
	fn a_burst_keeps_only_the_latest_position_and_preserves_release_and_next_key() {
		let mut input = Input::default();
		let release = Event::Mouse(MouseEvent {
			kind: MouseEventKind::Up(MouseButton::Left),
			column: 8,
			row: 999,
			modifiers: KeyModifiers::NONE,
		});
		let key = Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
		let mut queue: VecDeque<_> = (1..1000)
			.map(drag)
			.chain([release.clone(), key.clone()])
			.collect();
		assert_eq!(
			input.coalesce(drag(0), || Ok(queue.pop_front())).unwrap(),
			drag(999)
		);
		assert!(input.poll(Duration::ZERO).unwrap());
		assert_eq!(input.read().unwrap(), release);
		assert_eq!(queue.pop_front(), Some(key));
	}

	#[test]
	fn clicks_keys_wheels_resizes_and_changed_drag_buttons_or_modifiers_are_barriers() {
		let mouse = |kind, modifiers| {
			Event::Mouse(MouseEvent {
				kind,
				column: 2,
				row: 3,
				modifiers,
			})
		};
		let barriers = [
			mouse(MouseEventKind::Down(MouseButton::Left), KeyModifiers::NONE),
			mouse(MouseEventKind::ScrollUp, KeyModifiers::NONE),
			mouse(MouseEventKind::Drag(MouseButton::Right), KeyModifiers::NONE),
			mouse(MouseEventKind::Drag(MouseButton::Left), KeyModifiers::SHIFT),
			Event::Resize(40, 12),
			Event::Paste("text".into()),
			Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
		];
		for boundary in barriers {
			let mut input = Input::default();
			let mut queue = VecDeque::from([drag(4), boundary.clone(), drag(9)]);
			assert_eq!(
				input.coalesce(drag(1), || Ok(queue.pop_front())).unwrap(),
				drag(4)
			);
			assert_eq!(input.pending.take(), Some(boundary));
			assert_eq!(queue.pop_front(), Some(drag(9)));
		}
	}

	#[test]
	fn non_drags_never_read_ahead_and_continuous_motion_is_bounded() {
		let mut input = Input::default();
		let key = Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
		assert_eq!(
			input
				.coalesce(key.clone(), || panic!("must not read ahead"))
				.unwrap(),
			key
		);
		assert_eq!(input.coalesce(drag(7), || Ok(None)).unwrap(), drag(7));
		let mut reads = 0;
		let latest = input
			.coalesce(drag(0), || {
				reads += 1;
				Ok(Some(drag(reads as u16)))
			})
			.unwrap();
		assert_eq!(reads, MAX_DRAGS - 1);
		assert_eq!(latest, drag((MAX_DRAGS - 1) as u16));
	}
}
