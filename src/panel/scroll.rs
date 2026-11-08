#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
	Tools,
	Indexed,
	Characters,
}

impl crate::components::scrollbar::Orientation for Target {
	fn horizontal(self) -> bool {
		false
	}
}
