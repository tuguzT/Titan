//! Utilities for different types of storages for **components** of ECS.

use std::ops::{Index, IndexMut};

use slotmap::{HopSlotMap, SecondaryMap};

use super::{super::Entity, Component, ComponentID};

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
    /// If component was already attached, it will be replaced by value.
    ///
    /// Returns previously attached component, if any.
    ///
    pub fn insert(&mut self, entity: Entity, component: T) -> Option<T> {
        if self.attached(entity) {
            let prev = *self.get(entity)?;
            *self.get_mut(entity)? = component;
            return Some(prev);
        }
        let id = self.components.insert(component);
        self.component_to_entity.insert(id, entity);
        self.entity_to_component.insert(entity, id);
        None
    }

    /// Removes component and detaches it from the entity.
    ///
    /// Returns component that was previously attached to the entity.
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

    /// Returns iterator over all entities which have component of this type.
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entity_to_component.keys()
    }

    /// Returns immutable iterator over all components.
    pub fn components(&self) -> impl Iterator<Item = &T> {
        self.components.values()
    }

    /// Returns mutable iterator over all components.
    pub fn components_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.components.values_mut()
    }
}

pub struct IntoIter<T>
where
    T: Component,
{
    component_to_entity: SecondaryMap<ComponentID, Entity>,
    components: HopSlotMap<ComponentID, T>,
    index: usize,
}

impl<T> Iterator for IntoIter<T>
where
    T: Component,
{
    type Item = (Entity, T);

    fn next(&mut self) -> Option<Self::Item> {
        let (id, component) = self.components.iter().nth(self.index)?;
        self.index += 1;
        let entity = self.component_to_entity.get(id)?;
        Some((*entity, *component))
    }
}

impl<T> IntoIterator for ComponentStorage<T>
where
    T: Component,
{
    type Item = (Entity, T);
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            component_to_entity: self.component_to_entity,
            components: self.components,
            index: 0,
        }
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
