//! Utilities for storage of ECS.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use super::EntityStorage;

/// Storage for entities, components and systems of ECS.
#[allow(dead_code)]
pub struct World {
    /// Storage for all entities.
    entities: EntityStorage,
    /// Map with typeid of components and their storages.
    component_storages: HashMap<TypeId, Box<dyn Any>>,
    // TODO: storage for systems and impl
}
