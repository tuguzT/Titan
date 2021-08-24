//! Utilities for *systems* in ECS.

use std::error::Error;

use super::{Component, World};

/// Objects of this trait represent *system* of ECS.
pub trait System {
    /// Type of component which will be handled by this system.
    type Type: Component;

    /// Handle state of the current system.
    ///
    /// Do something useful with given components.
    ///
    fn handle(&mut self, world: &mut World) -> Result<(), Box<dyn Error + Send + Sync + 'static>>;
}
