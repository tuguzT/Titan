//! Utilities for *entities* in ECS.

use slotmap::{new_key_type, SlotMap};

new_key_type! {
    /// Unique identifier of the *entity* of ECS.
    pub struct Entity;
}

/// Storage for all entities of ECS.
pub type EntityStorage = SlotMap<Entity, ()>;
