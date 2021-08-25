//! Simple game engine based on Rust and Vulkan API

#![windows_subsystem = "windows"]

use std::error::Error;

use egui::TopBottomPanel;

use titan_core::{app::DeltaTime, config::Config, window::Event};

mod logger;

const APP_NAME: &str = env!("CARGO_CRATE_NAME", "library must be compiled by Cargo");
const APP_VERSION_STR: &str = env!("CARGO_PKG_VERSION", "library must be compiled by Cargo");

/// Entry point of `titan-rs` game engine
#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let _handle = logger::init().unwrap();
    log::info!("logger initialized successfully");

    let version = APP_VERSION_STR.parse().unwrap();
    let enable_validation = cfg!(debug_assertions);
    let config = Config::new(APP_NAME.to_string(), version, enable_validation);

    let mut duration = DeltaTime::ZERO;
    let mut fps = 0;

    let application = titan_core::init(config)?;
    application.run(move |event| match event {
        Event::Created => {
            log::debug!("created");
        }
        Event::Resized(size) => {
            let size: (u32, u32) = size.into();
            log::debug!("resized with {:?}", size);
        }
        Event::Update(delta_time) => {
            duration += delta_time;
        }
        Event::UI(ctx) => {
            const ID: &str = "top_panel";

            TopBottomPanel::top(ID).show(&ctx, |ui| {
                if duration.as_secs() > 0 {
                    fps = 0;
                    duration = DeltaTime::ZERO;
                } else {
                    fps += 1;
                }
                let text = format!("FPS: {}", fps);
                ui.label(text);
            });
        }
        Event::Destroyed => {
            log::debug!("destroyed");
        }
    })
}
