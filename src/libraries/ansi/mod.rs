//! Styled text and ANSI conversions, independent of UI, storage, and input devices.
//!
//! ```
//! use terminal_ux_designer::libraries::ansi::{Action, Document, Position, TextStyle};
//!
//! let mut document = Document::from_text("Hello", TextStyle::default())?;
//! document.apply_style(Position::default()..document.end(), Action::Bold, true)?;
//! let fragment = document.extract(Position::default()..document.end())?;
//! assert_eq!(fragment, Document::try_from(document.to_data())?);
//! assert!(document.to_escaped_ansi().contains("Hello"));
//! # Ok::<(), terminal_ux_designer::libraries::ansi::Error>(())
//! ```
mod data;
mod document;
mod encode;
mod operations;
mod parse;
mod style;

pub use data::{DocumentData, Error};
pub use document::{Document, Position};
pub use encode::escape;
pub use parse::{Import, Issue, Mode, parse};
pub use style::{Action, Ink, Run, Styles, TextStyle};
