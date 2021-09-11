//! Utilities for window handling of game engine.

use egui::CtxRef;

use crate::app::DeltaTime;

/// General event of game engine window.
pub enum Event {
    /// Called when game window was created.
    Created,

    /// Called when game window was resized.
    Resized(Size),

    /// Called when game window needs updating.
    Update(DeltaTime),

    /// Called when game UI needs updating.
    UI(CtxRef),

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

impl From<[u32; 2]> for Size {
    fn from(array: [u32; 2]) -> Self {
        Self::new(array[0], array[1])
    }
}

impl From<Size> for [u32; 2] {
    fn from(size: Size) -> Self {
        [size.width, size.height]
    }
}

impl From<(u32, u32)> for Size {
    fn from(tuple: (u32, u32)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

impl From<Size> for (u32, u32) {
    fn from(size: Size) -> Self {
        (size.width, size.height)
    }
}
