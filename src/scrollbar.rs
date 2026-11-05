use ratatui::{
	Frame,
	layout::Rect,
	style::{Color, Style},
	widgets::Paragraph,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
	EditorVertical,
	EditorHorizontal,
	CodeVertical,
	CodeHorizontal,
	Tools,
	Indexed,
}

#[derive(Clone, Copy)]
pub struct Bar {
	pub target: Target,
	pub area: Rect,
	pub content: usize,
	pub viewport: usize,
	pub offset: usize,
}

impl Bar {
	pub fn horizontal(self) -> bool {
		matches!(
			self.target,
			Target::EditorHorizontal | Target::CodeHorizontal
		)
	}
	pub fn length(self) -> usize {
		if self.horizontal() {
			self.area.width as usize
		} else {
			self.area.height as usize
		}
	}
	pub fn thumb(self) -> (usize, usize) {
		let length = self.length();
		let size = (length * self.viewport / self.content.max(1))
			.max(1)
			.min(length);
		let max = self.content.saturating_sub(self.viewport);
		let start = self.offset.min(max) * (length - size) / max.max(1);
		(start, size)
	}
	pub fn coordinate(self, x: u16, y: u16) -> usize {
		if self.horizontal() {
			x.saturating_sub(self.area.x) as usize
		} else {
			y.saturating_sub(self.area.y) as usize
		}
	}
	pub fn offset_at(self, coordinate: usize, grab: usize) -> usize {
		let (_, size) = self.thumb();
		let travel = self.length().saturating_sub(size);
		coordinate.saturating_sub(grab).min(travel) * self.content.saturating_sub(self.viewport)
			/ travel.max(1)
	}
}

pub fn draw(frame: &mut Frame, bars: &mut Vec<Bar>, bar: Bar) {
	if bar.content <= bar.viewport || bar.viewport == 0 || bar.length() == 0 {
		return;
	}
	let (start, size) = bar.thumb();
	for cell in 0..bar.length() {
		let active = (start..start + size).contains(&cell);
		let symbol = match (bar.horizontal(), active) {
			(true, true) => "━",
			(true, false) => "─",
			(false, true) => "┃",
			(false, false) => "│",
		};
		let rect = if bar.horizontal() {
			Rect::new(bar.area.x + cell as u16, bar.area.y, 1, 1)
		} else {
			Rect::new(bar.area.x, bar.area.y + cell as u16, 1, 1)
		};
		frame.render_widget(
			Paragraph::new(symbol).style(Style::default().fg(if active {
				Color::Cyan
			} else {
				Color::Rgb(62, 78, 106)
			})),
			rect,
		);
	}
	bars.push(bar);
}
