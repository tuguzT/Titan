//! Entity Component System (ECS) utilities for game engine.

use slotmap::{new_key_type, SlotMap};

pub use traits::*;

mod traits;

/// Zero-sized struct that represents **entity** in ECS.
pub struct Entity;

new_key_type! {
    /// Unique identifier of the **entity**.
    pub struct EntityID;
}

/// Container for entities, components and systems of ECS.
#[allow(dead_code)]
pub struct World {
    /// Storage for all entities.
    entities: SlotMap<EntityID, Entity>,
    // todo storage for components and systems
}
