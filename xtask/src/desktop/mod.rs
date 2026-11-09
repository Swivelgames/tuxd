//! Isolated X11 rendering and media capture; never uses the user's display.
mod display;
mod media;
mod rendering;
mod widths;

pub use media::capture;
pub use rendering::{Mode, render};
pub use widths::check;
#[cfg(unix)]
pub use widths::probe;
