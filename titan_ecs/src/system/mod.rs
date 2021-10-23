//! Utilities for *systems* in ECS.

use signature::Signature;

mod signature;

/// Objects of this trait represent *system* of ECS.
pub trait System {
    /// Component types which will be handled by this system.
    type Type: Signature;

    /// Handles state of the current system with provided components.
    fn handle(&mut self, components: impl Iterator<Item = Self::Type>);
}
