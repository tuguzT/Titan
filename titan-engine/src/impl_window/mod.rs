use std::error::Error;
use std::mem::ManuallyDrop;

use winit::dpi::LogicalSize;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use super::config::Config;
use super::graphics::Renderer;
use super::window::Event as MyEvent;

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

    pub fn run<T>(self, renderer: Renderer, mut callback: T) -> !
    where
        T: 'static + FnMut(MyEvent),
    {
        self.window.set_visible(true);
        let window = self.window;
        let mut renderer = ManuallyDrop::new(renderer);
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::NewEvents(cause) => match cause {
                    StartCause::Init => callback(MyEvent::Created),
                    _ => (),
                },
                Event::WindowEvent { event, window_id } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(size) => {
                            let size = (size.width, size.height);
                            callback(MyEvent::Resized(size.into()));
                        }
                        _ => (),
                    }
                }
                Event::MainEventsCleared => {
                    if let Err(error) = renderer.render() {
                        log::error!("{}", error);
                        *control_flow = ControlFlow::Exit;
                    }
                }
                Event::LoopDestroyed => {
                    callback(MyEvent::Destroyed);
                    if let Err(error) = renderer.wait() {
                        log::error!("{}", error);
                    }
                    unsafe { ManuallyDrop::drop(&mut renderer) };
                    log::info!(target: "titan_engine::window", "closing this application");
                }
                _ => (),
            }
        })
    }
}
