use std::error::Error;
use std::mem::ManuallyDrop;

use winit::dpi::LogicalSize;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use super::config::Config;
use super::graphics::Renderer;
use super::window::Callback;

pub struct Window {
    window: winit::window::Window,
    event_loop: EventLoop<()>,
}

impl Window {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(format!("{} version {}", config.name(), config.version()))
            .with_min_inner_size(LogicalSize::new(250, 100))
            .with_visible(false)
            .build(&event_loop)?;
        Ok(Self { window, event_loop })
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn run<T>(self, renderer: Renderer) -> !
    where
        T: Callback<T> + 'static,
    {
        self.window.set_visible(true);
        let window = self.window;
        let event_handler = T::new();
        let mut renderer = ManuallyDrop::new(renderer);
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::NewEvents(cause) => match cause {
                    StartCause::Init => event_handler.created(),
                    _ => (),
                },
                Event::WindowEvent { event, window_id } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(size) => {
                            event_handler.resized(size.width, size.height)
                        }
                        _ => (),
                    }
                }
                Event::MainEventsCleared => renderer.render(),
                Event::LoopDestroyed => {
                    event_handler.destroyed();
                    unsafe { ManuallyDrop::drop(&mut renderer) };
                    log::info!("Closing this application");
                }
                _ => (),
            }
        })
    }
}
