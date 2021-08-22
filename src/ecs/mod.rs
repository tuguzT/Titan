//! Entity Component System (ECS) utilities for game engine.

pub use component::{Component, ComponentStorage};
pub use entity::Entity;
pub use system::System;
pub use world::World;

use entity::EntityStorage;

mod component;
mod entity;
mod system;
mod world;
