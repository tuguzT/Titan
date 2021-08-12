use std::mem::ManuallyDrop;

use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use config::Config;
use error::Result;
use graphics::Renderer;

pub mod config;
pub mod error;
pub mod window;

mod graphics;

type MyEvent = window::Event;

#[cfg(feature = "jni-export")]
mod jni;

pub fn run<T>(config: Config, mut callback: T) -> !
where
    T: 'static + FnMut(MyEvent),
{
    let event_loop = EventLoop::new();

    let renderer = get_or_panic(Renderer::new(&config, &event_loop));
    log::info!("renderer initialized successfully");
    let mut renderer = ManuallyDrop::new(renderer);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        let window = renderer.window();
        match event {
            Event::NewEvents(cause) => match cause {
                StartCause::Init => {
                    callback(MyEvent::Created);
                    window.set_visible(true);
                }
                _ => (),
            },
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    let size = (size.width, size.height);
                    callback(MyEvent::Resized(size.into()));
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                let size = window.inner_size();
                if size.width == 0 || size.height == 0 {
                    return;
                }
                if let Err(error) = renderer.render() {
                    log::error!("{}", error);
                    log::info!("exit event sent due to previous error");
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::LoopDestroyed => {
                callback(MyEvent::Destroyed);
                unsafe { ManuallyDrop::drop(&mut renderer) }
                log::info!("closing this application");
            }
            _ => (),
        }
    })
}

fn get_or_panic<T>(value: Result<T>) -> T {
    match value {
        Ok(value) => value,
        Err(error) => {
            let format = format!("initialization error: {}", error);
            log::error!("{}", format);
            panic!("{}", format);
        }
    }
}
