use std::error::Error;

use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use crate::config::Config;
use winit::event::{Event, WindowEvent};

pub struct Window {
    window: winit::window::Window,
    event_loop: EventLoop<()>,
}

impl Window {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(format!(
                "{} {}",
                config.engine_name(),
                config.engine_version()
            ))
            .with_visible(false)
            .build(&event_loop)?;
        Ok(Self { window, event_loop })
    }

    pub fn run(self) -> ! {
        self.window.set_visible(true);
        let window = self.window;
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window.id() => *control_flow = ControlFlow::Exit,
                _ => (),
            }
        })
    }
}
