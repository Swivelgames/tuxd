//! Pure terminal-cell mapping; no widgets, themes, or application state.
pub mod display;
mod layout;
mod viewport;
pub use layout::Layout;
pub use viewport::Viewport;
