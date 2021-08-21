//! General traits for game engine ECS.

use std::any::Any;

use crate::error::Result;

use super::{EntityID, World};

/// Objects of this trait represent **component** of ECS.
pub trait Component: Any {
    /// Get ID of entity that owns current component.
    fn entity(&self) -> EntityID;
}

/// Objects of this trait represent **system** of ECS.
pub trait System {
    /// Type of component which will be handled by this system.
    type Type: Component;

    /// Handle state of the current system.
    ///
    /// Do something useful with given components.
    ///
    fn handle(&mut self, world: &mut World) -> Result<()>;
}
