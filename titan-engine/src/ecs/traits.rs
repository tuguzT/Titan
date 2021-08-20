//! General traits for game engine ECS.

use crate::app::DeltaTime;
use crate::error::Result;

use super::EntityID;

/// Objects of this trait represent **component** of ECS.
pub trait Component: 'static {
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
    fn call<T>(&mut self, state: SystemState, components: T) -> Result<()>
    where
        T: Iterator<Item = Self::Type>;
}

/// Enum that represents lifecycle of the system **inside of ECS**.
#[derive(Copy, Clone)]
pub enum SystemState {
    /// Emitted once during addition into the ECS.
    Initialize,

    /// Emitted contiguously before each frame.
    Update(DeltaTime),

    /// Emitted once during removing from the ECS.
    Destroy,
}
