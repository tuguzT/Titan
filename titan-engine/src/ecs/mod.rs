//! Entity Component System (ECS) utilities for game engine.

use slotmap::{new_key_type, SecondaryMap, SlotMap};

pub use traits::*;

mod traits;

/// Zero-sized struct that represents **entity** in ECS.
pub struct Entity;

new_key_type! {
    /// Unique identifier of the **entity**.
    pub struct EntityID;
}

/// Storage for all **entities** of ECS.
pub type EntityStorage = SlotMap<EntityID, Entity>;

/// Storage for all **components** of ECS.
pub type ComponentStorage = SecondaryMap<EntityID, Box<dyn Component>>;

// TODO: define type of storage for all **systems** of ECS
