//! Settings owns draft editing; fixed navigation remains independent of user bindings.
mod activation;
mod appearance;
mod bindings;
mod general;
mod input;
mod interaction;
mod mouse;
mod navigation;
mod sections;
mod view;

use appearance::Appearance;
use bindings::Shortcuts;
use general::General;
use interaction::Mode;
use sections::Section;

use crate::components::scrollbar::Bar;
use crate::shortcuts::registry::Subsection;
use crate::shortcuts::{ActionId, Config, InputScope, ShortcutKey};
use ratatui::layout::Rect;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Focus {
	Categories,
	Preset,
	ResetTheme,
	List,
	Binding(usize),
	Reset,
	Unset,
	Leader,
	Timeout,
	Footer(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
	Section(Subsection),
	Action(ActionId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hit {
	Category(Section),
	Selector,
	Preset(usize),
	PresetOpen,
	ResetTheme,
	Row(usize),
	Binding(usize),
	Reset,
	Unset,
	Leader,
	Timeout,
	Footer(usize),
	Discard,
	Keep,
	LeaderChoice(usize),
	ThemeRow(usize),
	ThemeReset(crate::theme::Role),
}

#[derive(Clone)]
struct Capture {
	slot: usize,
	action: ActionId,
	count: usize,
	keys: Vec<ShortcutKey>,
	received: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThemeRow {
	Section(crate::theme::Group),
	Color(crate::theme::Role),
}

pub struct Settings {
	general: General,
	shortcuts: Shortcuts,
	appearance: Appearance,
	mode: Mode,
	closed_groups: BTreeSet<Section>,
	category: Section,
	focus: Focus,
	top: usize,
	nav_top: usize,
	viewport: usize,
	narrow: bool,
	pending_g: bool,
	pub error: Option<String>,
	message: Option<String>,
	hits: Vec<(Rect, Hit)>,
	bar: Option<Bar>,
	nav_bar: Option<Bar>,
	drag: Option<(bool, usize)>,
	follow: bool,
	size: (u16, u16),
}

pub enum Outcome {
	Keep,
	Close,
	Save,
}

impl Settings {
	pub fn drafts(&self) -> crate::configuration::commit::Draft<'_> {
		crate::configuration::commit::Draft {
			shortcuts: &self.shortcuts.draft,
			theme: &self.appearance.draft,
			original_theme: &self.appearance.original,
		}
	}

	fn dirty(&self) -> bool {
		self.shortcuts.draft != self.shortcuts.original
			|| self.appearance.draft != self.appearance.original
	}

	pub fn new(config: &Config, category: impl Into<Section>) -> Self {
		let category = category.into();
		let mut state = Self {
			general: General::new(),
			shortcuts: Shortcuts::new(config),
			appearance: Appearance::new(),
			mode: Mode::Browse,
			closed_groups: BTreeSet::new(),
			category,
			focus: Focus::List,
			top: 0,
			nav_top: 0,
			viewport: 1,
			narrow: false,
			pending_g: false,
			error: None,
			message: None,
			hits: Vec::new(),
			bar: None,
			nav_bar: None,
			drag: None,
			follow: true,
			size: (0, 0),
		};
		state.switch_category(category);
		state.focus = Focus::List;
		state
	}

	pub fn for_scope(config: &Config, scope: InputScope) -> Self {
		let mut state = Self::new(config, scope.section());
		let action = match scope {
			InputScope::Location | InputScope::Filename => Some(ActionId::SelectAll),
			InputScope::Radio | InputScope::ToolTabs | InputScope::Grid | InputScope::Format => {
				Some(ActionId::FieldLeft)
			}
			InputScope::Channel => Some(ActionId::RangeIncrease),
			InputScope::Tools => Some(ActionId::Activate),
			_ => None,
		};
		if let Some(action) = action {
			state.select_action(action);
		}
		state
	}

	fn close(&mut self) -> Outcome {
		if !self.dirty() {
			Outcome::Close
		} else {
			self.mode = Mode::Discard(false);
			Outcome::Keep
		}
	}

	fn validate(&mut self) {
		self.error = self
			.shortcuts
			.draft
			.build()
			.err()
			.or_else(|| self.appearance.draft.resolve().err());
	}
}

#[cfg(test)]
mod tests;
