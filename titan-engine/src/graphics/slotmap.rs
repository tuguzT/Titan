use super::{
    command, device,
    ext::{debug_utils, swapchain},
    framebuffer, image, instance, pipeline, shader, surface,
    sync::{fence, semaphore},
};

pub fn clear() {
    fence::slotmap::clear();
    semaphore::slotmap::clear();
    shader::slotmap::clear();
    command::buffer::slotmap::clear();
    command::pool::slotmap::clear();
    framebuffer::slotmap::clear();
    pipeline::slotmap::clear();
    pipeline::layout::slotmap::clear();
    pipeline::render_pass::slotmap::clear();
    image::view::slotmap::clear();
    image::slotmap::clear();
    swapchain::slotmap::clear();
    device::queue::slotmap::clear();
    device::logical::slotmap::clear();
    device::physical::slotmap::clear();
    surface::slotmap::clear();
    debug_utils::slotmap::clear();
    instance::slotmap::clear();
}

#[doc(hidden)]
#[macro_export]
macro_rules! slotmap_helper {
    ($T:ty) => {
        use std::sync::{LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};
        type SlotMap = slotmap::SlotMap<Key, $T>;
        slotmap::new_key_type! {
            pub struct Key;
        }
        lazy_static::lazy_static! {
            static ref SLOTMAP: RwLock<SlotMap> = RwLock::new(SlotMap::with_key());
        }
        pub fn read() -> LockResult<RwLockReadGuard<'static, SlotMap>> {
            SLOTMAP.read()
        }
        pub fn write() -> LockResult<RwLockWriteGuard<'static, SlotMap>> {
            SLOTMAP.write()
        }
        pub fn clear() {
            if let Ok(mut slotmap) = self::write() {
                slotmap.clear()
            }
        }
    };
}
