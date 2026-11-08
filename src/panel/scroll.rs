#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
	Tools,
	Indexed,
}

impl crate::components::scrollbar::Orientation for Target {
	fn horizontal(self) -> bool {
		false
	}
}
