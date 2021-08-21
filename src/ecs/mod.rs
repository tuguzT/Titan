//! Entity Component System (ECS) utilities for game engine.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use entity::EntityStorage;

pub use component::{Component, ComponentStorage};
pub use entity::Entity;
pub use system::System;

mod component;
mod entity;
mod system;

/// Storage for entities, components and systems of ECS.
#[allow(dead_code)]
pub struct World {
    /// Storage for all entities.
    entities: EntityStorage,
    /// Map with typeid of components and their storages.
    component_storages: HashMap<TypeId, Box<dyn Any>>,
    // TODO: storage for systems and impl
}
