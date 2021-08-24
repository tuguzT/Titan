//! Entity Component System (ECS) utilities for game engine.

pub use component::Component;
pub use entity::Entity;
pub use system::System;
pub use world::World;

use component::ComponentManager;
use entity::EntityStorage;

mod component;
mod entity;
mod system;
mod world;
