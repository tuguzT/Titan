//! Utilities for *components* in ECS.

use std::any::Any;

use slotmap::{new_key_type, SecondaryMap, SlotMap};

use super::Entity;

/// Objects of this trait represent *component* of ECS.
///
/// Components should be just POD (plain old data).
///
pub trait Component: Copy + Any {}

impl<T> Component for T where T: Copy + Any {}

new_key_type! {
    /// Unique identifier of the *component* of ECS.
    struct ComponentID;
}

/// Storage for statically typed components of ECS.
pub struct ComponentStorage<T>
where
    T: Component,
{
    /// Components are actually stored here.
    components: SlotMap<ComponentID, T>,
    entity_to_component: SecondaryMap<Entity, ComponentID>,
    component_to_entity: SecondaryMap<ComponentID, Entity>,
}

impl<T> ComponentStorage<T>
where
    T: Component,
{
    /// Creates an empty component storage.
    pub fn new() -> Self {
        Self {
            components: SlotMap::with_key(),
            entity_to_component: SecondaryMap::new(),
            component_to_entity: SecondaryMap::new(),
        }
    }

    /// Inserts component and attaches it with given entity.
    ///
    /// # Panic
    ///
    /// Panics if component was already attached to the entity.
    ///
    pub fn insert(&mut self, entity: Entity, component: T) {
        assert!(
            !self.attached(entity),
            "component was already attached to the entity",
        );
        let id = self.components.insert(component);
        self.component_to_entity.insert(id, entity);
        self.entity_to_component.insert(entity, id);
    }

    /// Replaces component attached to given entity by value.
    ///
    /// Returns previously attached component, if any.
    ///
    pub fn replace(&mut self, entity: Entity, component: T) -> Option<T> {
        let prev = self.remove(entity);
        self.insert(entity, component);
        prev
    }

    /// Removes component and detaches it from given entity.
    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        let id = *self.entity_to_component.get(entity)?;
        self.entity_to_component.remove(entity);
        self.component_to_entity.remove(id);
        self.components.remove(id)
    }

    /// Returns `true` if component was already attached to the entity.
    pub fn attached(&self, entity: Entity) -> bool {
        self.entity_to_component.get(entity).is_some()
    }

    /// Retrieves an immutable reference to component associated with given entity.
    pub fn get(&self, entity: Entity) -> Option<&T> {
        let id = *self.entity_to_component.get(entity)?;
        self.components.get(id)
    }

    /// Retrieves a mutable reference to component associated with given entity.
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let id = *self.entity_to_component.get(entity)?;
        self.components.get_mut(id)
    }
}

#[cfg(test)]
mod tests {
    use super::{super::EntityStorage, *};

    #[test]
    fn test_insertion() {
        let mut entities = EntityStorage::with_key();
        let entity = entities.insert(());

        let mut storage = ComponentStorage::new();
        let component = "foo";

        storage.insert(entity, component);
        assert!(storage.attached(entity));
        assert_eq!(storage.get(entity), Some(&"foo"));

        storage.remove(entity);
        assert!(!storage.attached(entity));
        assert_eq!(storage.get(entity), None);
    }

    #[test]
    #[should_panic]
    fn test_insertion_assert() {
        use std::time::Instant;

        let mut entities = EntityStorage::with_key();
        let entity1 = entities.insert(());
        let entity2 = entities.insert(());

        let mut storage = ComponentStorage::new();
        storage.insert(entity1, Instant::now());
        storage.insert(entity2, Instant::now());
        storage.insert(entity1, Instant::now());
    }

    #[test]
    fn test_replace() {
        let mut entities = EntityStorage::with_key();
        let entity = entities.insert(());

        let mut storage = ComponentStorage::new();
        assert_eq!(storage.replace(entity, 123), None);
        assert_eq!(storage.replace(entity, 456), Some(123));
        assert_eq!(storage.remove(entity), Some(456));
        assert_eq!(storage.remove(entity), None);
    }
}
