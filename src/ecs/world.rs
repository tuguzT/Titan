//! Utilities for storage of ECS.

use super::ComponentManager;
use super::EntityStorage;

/// Storage for entities, components and systems of ECS.
#[derive(Default)]
pub struct World {
    /// Storage for all entities.
    entities: EntityStorage,
    /// Map with typeid of components and their storages.
    component_manager: ComponentManager,
    // TODO: storage for systems and impl
}
