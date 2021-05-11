use std::error::Error;
use std::mem::ManuallyDrop;

use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use crate::config::Config;
use crate::graphics::Renderer;

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
                config.engine_version(),
            ))
            .with_min_inner_size(LogicalSize::new(250, 100))
            .with_visible(false)
            .build(&event_loop)?;
        Ok(Self { window, event_loop })
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn run(self, renderer: Renderer) -> ! {
        self.window.set_visible(true);
        let window = self.window;
        let mut renderer = ManuallyDrop::new(renderer);
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window.id() => *control_flow = ControlFlow::Exit,
                Event::MainEventsCleared => renderer.render(),
                Event::LoopDestroyed => {
                    unsafe { ManuallyDrop::drop(&mut renderer) };
                    log::info!("Renderer object was destroyed");
                    log::info!("Closing this application");
                }
                _ => (),
            }
        })
    }
}
