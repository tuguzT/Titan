use std::sync::RwLock;

use slotmap::SlotMap;

pub trait SlotMappable: Sized + Send + Sync + 'static {
    type Key: slotmap::Key;

    fn key(&self) -> Self::Key;

    fn slotmap() -> &'static RwLock<SlotMap<Self::Key, Self>>;
}

pub trait HasParent<Parent>
where
    Self: SlotMappable,
    Parent: SlotMappable,
{
    fn parent_key(&self) -> Parent::Key;
}
