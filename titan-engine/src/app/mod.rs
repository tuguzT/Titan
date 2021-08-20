use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use ultraviolet::{Mat4, Vec3};
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::{
    config::Config,
    error::{Error, Result},
    graphics::{camera::CameraUBO, Renderer},
    window::{Event as MyEvent, Size},
};

pub type DeltaTime = Duration;

pub struct Application {
    _config: Config,
    renderer: Renderer,
    event_loop: Option<EventLoop<()>>,
}

impl Application {
    fn new(config: Config) -> Result<Self> {
        let event_loop = EventLoop::with_user_event();
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
        let mut start_time = Instant::now();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            let window = me.renderer.window();
            let size = window.inner_size();
            match event {
                Event::NewEvents(StartCause::Init) => {
                    start_time = Instant::now();
                    callback(MyEvent::Created);
                    window.set_visible(true);
                }
                Event::WindowEvent { event, window_id } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(size) => {
                            if size.width == 0 || size.height == 0 {
                                callback(MyEvent::Resized(Size::default()));
                                return;
                            }
                            if let Err(error) = me.renderer.resize() {
                                log::error!("window resizing error: {}", error);
                                *control_flow = ControlFlow::Exit;
                                return;
                            }
                            let size = (size.width, size.height);
                            callback(MyEvent::Resized(size.into()));
                        }
                        _ => (),
                    }
                }
                Event::MainEventsCleared => {
                    if size.width == 0 || size.height == 0 {
                        return;
                    }

                    let frame_start = Instant::now();
                    if let Err(error) = me.renderer.render() {
                        log::error!("rendering error: {}", error);
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                    let frame_end = Instant::now();
                    let delta_time = frame_end.duration_since(frame_start);
                    callback(MyEvent::Update(delta_time));

                    let ubo = {
                        let duration = Instant::now().duration_since(start_time);
                        let elapsed = duration.as_millis();

                        use ultraviolet::projection::perspective_vk as perspective;
                        let projection = perspective(
                            45f32.to_radians(),
                            (size.width as f32) / (size.height as f32),
                            1.0,
                            10.0,
                        );
                        let model = Mat4::from_rotation_z((elapsed as f32) * 0.1f32.to_radians());
                        let view =
                            Mat4::look_at(Vec3::new(2.0, 2.0, 2.0), Vec3::zero(), Vec3::unit_z());
                        CameraUBO::new(projection, model, view)
                    };
                    me.renderer.set_camera_ubo(ubo);
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
    const UNINITIALIZED: bool = false;
    const INITIALIZED: bool = true;

    let initialized = FLAG
        .compare_exchange(
            UNINITIALIZED,
            INITIALIZED,
            Ordering::SeqCst,
            Ordering::SeqCst,
        )
        .unwrap();

    if !initialized {
        Application::new(config)
    } else {
        Err(Error::from(
            "cannot create more than one application instance",
        ))
    }
}
