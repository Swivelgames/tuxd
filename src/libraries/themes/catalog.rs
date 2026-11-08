//! Stable role identities and presentation metadata share one definition.
use super::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Group {
	Interface,
	Controls,
	EditorFeedback,
	CodeHighlighting,
	RgbControls,
}

impl Group {
	pub const ALL: &'static [Self] = &[
		Self::Interface,
		Self::Controls,
		Self::EditorFeedback,
		Self::CodeHighlighting,
		Self::RgbControls,
	];
	pub fn label(self) -> &'static str {
		match self {
			Self::Interface => "Interface",
			Self::Controls => "Controls",
			Self::EditorFeedback => "Editor & Feedback",
			Self::CodeHighlighting => "Code Highlighting",
			Self::RgbControls => "RGB Controls",
		}
	}
}

macro_rules! roles {
	($( $variant:ident, $field:ident, $label:literal, $group:ident; )*) => {
		#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
		pub enum Role {
			$( $variant, )*
		}

		impl Role {
			pub const ALL: &'static [Self] = &[$(Self::$variant,)*];
			pub fn id(self) -> &'static str {
				match self { $(Self::$variant => stringify!($field),)* }
			}

			pub fn label(self) -> &'static str {
				match self { $(Self::$variant => $label,)* }
			}

			pub fn group(self) -> Group {
				match self { $(Self::$variant => Group::$group,)* }
			}

			pub fn from_id(id: &str) -> Result<Self, String> {
				match id {
					$(stringify!($field) => Ok(Self::$variant),)*
					_ => Err(format!("Unknown theme role {id}")),
				}
			}
		}

		#[derive(Clone, Debug, PartialEq, Eq)]
		pub struct Theme {
			$(pub $field: Color,)*
		}

		impl Theme {
			pub fn color(&self, role: Role) -> Color {
				match role { $(Role::$variant => self.$field,)* }
			}

			pub(super) fn set(&mut self, role: Role, color: Color) {
				match role { $(Role::$variant => self.$field = color,)* }
			}
		}
	};
}
roles! {
	Accent, accent, "Accent", Interface;
	Text, text, "Text", Interface;
	Muted, muted, "Muted text", Interface;
	Surface, surface, "Background", Interface;
	Border, border, "Border", Interface;
	Field, field, "Field background", Controls;
	Selected, selected, "Selected background", Controls;
	SelectedText, selected_text, "Selected text", Controls;
	Focus, focus, "Focused background", Controls;
	InactiveIndicator, inactive_indicator, "Inactive indicator", Controls;
	SelectionOverlay, selection_overlay, "Selection overlay", EditorFeedback;
	Error, error, "Error", EditorFeedback;
	SyntaxSequence, syntax_sequence, "Sequences", CodeHighlighting;
	SyntaxParameter, syntax_parameter, "Parameters", CodeHighlighting;
	SyntaxEscape, syntax_escape, "Escapes", CodeHighlighting;
	SyntaxSeparator, syntax_separator, "Separators", CodeHighlighting;
	SyntaxWhitespace, syntax_whitespace, "Whitespace", CodeHighlighting;
	RgbRed, rgb_red, "Red", RgbControls;
	RgbGreen, rgb_green, "Green", RgbControls;
	RgbBlue, rgb_blue, "Blue", RgbControls;
	RangeText, range_text, "Range text", RgbControls;
}
