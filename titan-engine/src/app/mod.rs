use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicBool, Ordering};

use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::{
    config::Config,
    error::{Error, Result},
    graphics::Renderer,
    window::Event as MyEvent,
};

pub struct Application {
    _config: Config,
    renderer: Renderer,
    event_loop: Option<EventLoop<()>>,
}

impl Application {
    fn new(config: Config) -> Result<Self> {
        let event_loop = EventLoop::new();
        let renderer = Renderer::new(&config, &event_loop)?;
        Ok(Self {
            renderer,
            _config: config,
            event_loop: Some(event_loop),
        })
    }

    pub fn run(mut self, mut callback: impl FnMut(MyEvent) + 'static) -> ! {
        let event_loop = self.event_loop.take().unwrap();
        let mut me = ManuallyDrop::new(self);

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            let window = me.renderer.window();
            match event {
                Event::NewEvents(cause) => match cause {
                    StartCause::Init => {
                        callback(MyEvent::Created);
                        window.set_visible(true);
                    }
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
                    let size = window.inner_size();
                    if size.width == 0 || size.height == 0 {
                        return;
                    }
                    if let Err(error) = me.renderer.render() {
                        log::error!("{}", error);
                        log::info!("exit event sent due to previous error");
                        *control_flow = ControlFlow::Exit;
                    }
                }
                Event::LoopDestroyed => {
                    callback(MyEvent::Destroyed);
                    unsafe { ManuallyDrop::drop(&mut me) }
                    log::info!("closing this application");
                }
                _ => (),
            }
        })
    }
}

/// Creates a unique application instance.
/// If application instance was created earlier, it will return an error.
///
///     Panic
/// This function could panic if invoked **not on main thread**.
pub fn init(config: Config) -> Result<Application> {
    static FLAG: AtomicBool = AtomicBool::new(false);

    if !FLAG
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .unwrap()
    {
        Application::new(config)
    } else {
        Err(Error::from(
            "cannot create more than one application instance",
        ))
    }
}
