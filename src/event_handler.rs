use titan_engine::window::Callback;

pub struct EventHandler;

impl Callback<EventHandler> for EventHandler {
    fn new() -> EventHandler {
        Self {}
    }

    fn created(&self) {
        log::debug!("created")
    }

    fn resized(&self, width: u32, height: u32) {
        log::debug!("resized with ({}, {})", width, height)
    }

    fn destroyed(&self) {
        log::debug!("destroyed")
    }
}
