//! Utilities for engine initialization.

use egui::TextureId;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use egui_winit_platform::{Platform, PlatformDescriptor};
use image::RgbaImage;
use thiserror::Error;
use ultraviolet::{Mat4, Vec3};
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use crate::{
    config::Config,
    graphics::{camera::CameraUBO, error::ImageRegisterError, Renderer, RendererCreationError},
    window::{Event as MyEvent, Size},
};

pub type Result<T> = std::result::Result<T, AppCreationError>;

#[derive(Debug, Error)]
pub enum AppCreationError {
    #[error("cannot create more than one application instance")]
    Initialized,

    #[error("graphics initialization error: {0}")]
    Graphics(#[from] RendererCreationError),
}

/// Type which represents duration between two frames.
pub type DeltaTime = Duration;

/// General context of game engine.
///
/// Can be created using [`init`] function.
///
pub struct Application {
    _config: Config,
    renderer: Renderer,
    egui: Option<Platform>,
    event_loop: Option<EventLoop<()>>,
}

impl Application {
    fn new(config: Config) -> Result<Self> {
        let event_loop = EventLoop::with_user_event();
        let renderer = Renderer::new(&config, &event_loop)?;

        let window = renderer.window();
        let size = window.inner_size();
        let egui = Platform::new(PlatformDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor(),
            ..Default::default()
        });

        Ok(Self {
            renderer,
            egui: Some(egui),
            _config: config,
            event_loop: Some(event_loop),
        })
    }

    /// Returns underlying window of this application.
    pub fn window(&self) -> &Window {
        self.renderer.window()
    }

    pub fn register_ui_image(
        &mut self,
        image: &RgbaImage,
    ) -> std::result::Result<TextureId, ImageRegisterError> {
        self.renderer.register_ui_image(image)
    }

    /// Starts execution of game engine.
    pub fn run(mut self, mut callback: impl FnMut(MyEvent) + 'static) -> ! {
        let event_loop = self.event_loop.take().unwrap();

        let mut start_time = Instant::now();
        event_loop.run(move |event, _, control_flow| {
            // Have the closure take ownership of `self`.
            // `event_loop.run` never returns, therefore we must do this to ensure
            // the resources are properly cleaned up.
            let _ = &self;

            *control_flow = ControlFlow::Poll;

            // Take `Platform` object from `self` to workaround about borrow checker.
            let mut egui = self.egui.take().unwrap();

            // Have this closure to early return if needed (for example if error is occurred).
            // Closure is needed because `label_break_value` feature is unstable.
            let action = || {
                egui.handle_event(&event);
                egui.update_time(start_time.elapsed().as_secs_f64());

                let window = self.window();
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
                                if let Err(error) = self.renderer.resize() {
                                    log::error!("window resizing error: {}", error);
                                    *control_flow = ControlFlow::Exit;
                                    return;
                                }
                                let size = (size.width, size.height);
                                callback(MyEvent::Resized(size.into()));
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                let size = *new_inner_size;
                                if size.width == 0 || size.height == 0 {
                                    callback(MyEvent::Resized(Size::default()));
                                    return;
                                }
                                if let Err(error) = self.renderer.resize() {
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
                        let size = window.inner_size();
                        if size.width == 0 || size.height == 0 {
                            return;
                        }
                        window.request_redraw();
                    }
                    Event::RedrawRequested(window_id) if window_id == window.id() => {
                        let size = window.inner_size();
                        if size.width == 0 || size.height == 0 {
                            return;
                        }
                        let frame_start = Instant::now();

                        egui.begin_frame();
                        let context = egui.context();
                        callback(MyEvent::UI(context.clone()));
                        let (_output, shapes) = egui.end_frame(Some(window));
                        let meshes = context.tessellate(shapes);
                        let texture = context.texture();

                        if let Err(error) = self.renderer.render(Some((meshes, texture))) {
                            log::error!("rendering error: {}", error);
                            *control_flow = ControlFlow::Exit;
                            return;
                        }
                        let delta_time = Instant::now().duration_since(frame_start);
                        callback(MyEvent::Update(delta_time));

                        let ubo = {
                            let duration = Instant::now().duration_since(start_time);
                            let elapsed = duration.as_millis() as f32;

                            use ultraviolet::projection::perspective_vk as perspective;
                            let projection = perspective(
                                45f32.to_radians(),
                                (size.width as f32) / (size.height as f32),
                                1.0,
                                10.0,
                            );
                            let model = Mat4::from_rotation_z(elapsed * 0.1f32.to_radians());
                            let view = Mat4::look_at(
                                Vec3::new(2.0, 2.0, 2.0),
                                Vec3::zero(),
                                Vec3::unit_z(),
                            );
                            CameraUBO::new(projection, model, view)
                        };
                        self.renderer.set_camera_ubo(ubo);
                    }
                    Event::LoopDestroyed => {
                        callback(MyEvent::Destroyed);
                        log::info!("closing this application");
                    }
                    _ => (),
                }
            };
            action();

            // Assign `Platform` object back to `self`.
            self.egui = Some(egui);
        })
    }
}

/// Creates a unique [`Application`] instance.
/// If application instance was created earlier, function call will return an error.
///
/// # Errors
///
/// An error is returned if application instance have already been initialized.
///
/// # Panic
///
/// This function could panic if invoked **not on main thread**.
///
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

    if initialized {
        return Err(AppCreationError::Initialized);
    }
    Application::new(config)
}
