use titan_engine::window::Callback;

pub struct EventHandler {}

impl Callback<EventHandler> for EventHandler {
    fn new() -> EventHandler {
        Self {}
    }

    fn on_create(&self) {
        log::debug!("on_create called")
    }

    fn on_destroy(&self) {
        log::debug!("on_destroy called")
    }
}
