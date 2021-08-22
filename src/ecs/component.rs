//! Utilities for *components* in ECS.

use std::any::Any;
use std::ops::{Index, IndexMut};
use std::vec::IntoIter;

use slotmap::{new_key_type, HopSlotMap, SecondaryMap};

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
#[derive(Default)]
pub struct ComponentStorage<T>
where
    T: Component,
{
    /// Components are actually stored here.
    components: HopSlotMap<ComponentID, T>,
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
            components: HopSlotMap::with_key(),
            entity_to_component: SecondaryMap::new(),
            component_to_entity: SecondaryMap::new(),
        }
    }

    /// Inserts component and attaches it to the entity.
    ///
    /// # Panics
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

    /// Replaces component attached to the entity by value.
    ///
    /// Returns previously attached component, if any.
    ///
    pub fn replace(&mut self, entity: Entity, component: T) -> Option<T> {
        let prev = self.remove(entity);
        self.insert(entity, component);
        prev
    }

    /// Removes component and detaches it from the entity.
    ///
    /// Returns component that was attached to the entity.
    ///
    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        let id = *self.entity_to_component.get(entity)?;
        self.entity_to_component.remove(entity);
        self.component_to_entity.remove(id);
        self.components.remove(id)
    }

    /// Returns `true` if component was already attached to the entity.
    pub fn attached(&self, entity: Entity) -> bool {
        self.entity_to_component.contains_key(entity)
    }

    /// Retrieves an immutable reference to component attached to the entity.
    pub fn get(&self, entity: Entity) -> Option<&T> {
        let id = *self.entity_to_component.get(entity)?;
        self.components.get(id)
    }

    /// Retrieves a mutable reference to component attached to the entity.
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let id = *self.entity_to_component.get(entity)?;
        self.components.get_mut(id)
    }

    /// Returns immutable iterator over all components with their entities.
    pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
        let component_to_entity = &self.component_to_entity;
        self.components
            .iter()
            .map(move |(id, component)| (component_to_entity[id], component))
    }

    /// Returns mutable iterator over all components with their entities.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
        let component_to_entity = &self.component_to_entity;
        self.components
            .iter_mut()
            .map(move |(id, component)| (component_to_entity[id], component))
    }
}

impl<T> IntoIterator for ComponentStorage<T>
where
    T: Component,
{
    type Item = (Entity, T);
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(mut self) -> Self::IntoIter {
        let component_to_entity = self.component_to_entity;
        let drained = self.components.drain();
        let vec: Vec<_> = drained
            .map(|(id, component)| (component_to_entity[id], component))
            .collect();
        vec.into_iter()
    }
}

impl<T> Index<Entity> for ComponentStorage<T>
where
    T: Component,
{
    type Output = T;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity)
            .expect("there is no component attached to the entity")
    }
}

impl<T> IndexMut<Entity> for ComponentStorage<T>
where
    T: Component,
{
    fn index_mut(&mut self, entity: Entity) -> &mut Self::Output {
        self.get_mut(entity)
            .expect("there is no component attached to the entity")
    }
}

#[cfg(test)]
mod tests {
    use super::{super::EntityStorage, *};

    #[test]
    fn test_insertion() {
        let mut entities = EntityStorage::with_key();
        let mut storage = ComponentStorage::new();

        let entity = entities.insert(());
        let component = "foo";

        storage.insert(entity, component);
        assert!(storage.attached(entity));
        assert_eq!(storage[entity], "foo");

        storage.remove(entity);
        assert!(!storage.attached(entity));
        assert_eq!(storage.get(entity), None);
    }

    #[test]
    #[should_panic]
    fn test_insertion_assert() {
        use std::time::Instant;

        let mut entities = EntityStorage::with_key();
        let mut storage = ComponentStorage::new();

        let entity1 = entities.insert(());
        let entity2 = entities.insert(());

        storage.insert(entity1, Instant::now());
        storage.insert(entity2, Instant::now());
        storage.insert(entity1, Instant::now());
    }

    #[test]
    fn test_replace() {
        let mut entities = EntityStorage::with_key();
        let mut storage = ComponentStorage::new();

        let entity = entities.insert(());
        assert_eq!(storage.replace(entity, 123), None);
        assert_eq!(storage.replace(entity, 456), Some(123));
        assert_eq!(storage.remove(entity), Some(456));
        assert_eq!(storage.remove(entity), None);
    }

    #[test]
    #[should_panic]
    fn test_index() {
        let mut entities = EntityStorage::with_key();
        let mut storage = ComponentStorage::new();

        let entity = entities.insert(());
        storage[entity] = 0;
        assert_eq!(storage[entity], 0);

        let entity = entities.insert(());
        let _component = storage[entity];
    }

    #[test]
    fn test_iterator() {
        let mut entities = EntityStorage::with_key();
        let mut storage = ComponentStorage::new();

        let _entities: Vec<_> = (0..100)
            .map(|int| {
                let entity = entities.insert(());
                storage.insert(entity, int);
                entity
            })
            .collect();

        for (_, component) in storage.iter_mut() {
            *component += 10;
        }
        for ((_, component), value) in storage.iter().zip(10..110) {
            assert_eq!(*component, value);
        }
        let iterator = storage.into_iter();
        let range: Vec<_> = iterator.map(|tuple| tuple.1).collect();
        assert_eq!(range, (10..110).collect::<Vec<_>>());
    }
}
