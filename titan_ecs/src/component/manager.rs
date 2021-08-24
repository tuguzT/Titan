//! Utilities for managing component storages.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use super::{Component, ComponentStorage, super::Entity};

/// Manager of all components of ECS.
#[derive(Default)]
#[repr(transparent)]
pub struct ComponentManager {
    storages: HashMap<TypeId, Box<dyn Any>>,
}

impl ComponentManager {
    /// Creates new component manager.
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
        }
    }

    /// Inserts component of type `T` and attaches it to the entity.
    /// If component was already attached, it will be replaced by value.
    ///
    /// Returns previously attached component, if any.
    ///
    pub fn insert<T>(&mut self, entity: Entity, component: T) -> Option<T>
    where
        T: Component,
    {
        let storage = match self.get_storage_mut() {
            Some(storage) => storage,
            None => self.create_storage(),
        };
        storage.insert(entity, component)
    }

    /// Removes component of type `T` and detaches it from the entity.
    ///
    /// Returns component that was previously attached to the entity.
    ///
    pub fn remove<T>(&mut self, entity: Entity) -> Option<T>
    where
        T: Component,
    {
        let storage = self.get_storage_mut()?;
        storage.remove(entity)
    }

    /// Returns `true` if component of type `T` was already attached to the entity.
    pub fn attached<T>(&self, entity: Entity) -> bool
    where
        T: Component,
    {
        self.get_storage::<T>()
            .map(|storage| storage.attached(entity))
            .unwrap_or(false)
    }

    /// Retrieves an immutable reference to component of type `T` attached to the entity.
    pub fn get<T>(&self, entity: Entity) -> Option<&T>
    where
        T: Component,
    {
        let storage = self.get_storage::<T>()?;
        storage.get(entity)
    }

    /// Retrieves a mutable reference to component of type `T` attached to the entity.
    pub fn get_mut<T>(&mut self, entity: Entity) -> Option<&mut T>
    where
        T: Component,
    {
        let storage = self.get_storage_mut::<T>()?;
        storage.get_mut(entity)
    }

    fn get_storage<T>(&self) -> Option<&ComponentStorage<T>>
    where
        T: Component,
    {
        let typeid = TypeId::of::<T>();
        let boxed = self.storages.get(&typeid)?;
        Some(boxed.downcast_ref().expect("downcast error"))
    }

    fn get_storage_mut<T>(&mut self) -> Option<&mut ComponentStorage<T>>
    where
        T: Component,
    {
        let typeid = TypeId::of::<T>();
        let boxed = self.storages.get_mut(&typeid)?;
        Some(boxed.downcast_mut().expect("downcast error"))
    }

    fn create_storage<T>(&mut self) -> &mut ComponentStorage<T>
    where
        T: Component,
    {
        let typeid = TypeId::of::<T>();
        let boxed = Box::new(ComponentStorage::<T>::new());
        self.storages.insert(typeid, boxed);
        let boxed = self.storages.get_mut(&typeid).unwrap();
        boxed.downcast_mut().expect("downcast error")
    }
}
