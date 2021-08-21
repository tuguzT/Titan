//! Utilities for window handling of game engine.

use crate::app::DeltaTime;

/// General event of game engine window.
pub enum Event {
    /// Called when game window was created.
    Created,

    /// Called when game window was resized.
    Resized(Size),

    /// Called when game window needs updating.
    Update(DeltaTime),

    /// Called when game window will be destroyed.
    Destroyed,
}

/// Size of game engine window.
#[derive(Default, Copy, Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    /// Creates new size of window.
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl From<(u32, u32)> for Size {
    fn from(tuple: (u32, u32)) -> Self {
        Size::new(tuple.0, tuple.1)
    }
}

impl From<Size> for (u32, u32) {
    fn from(size: Size) -> Self {
        (size.width, size.height)
    }
}
