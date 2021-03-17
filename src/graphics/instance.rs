use std::error::Error;

use ash::{Entry, vk};
use ash::version::{EntryV1_0, InstanceV1_0};

use crate::{config, version};

pub struct Instance {
    vk_instance: vk::Instance,
    vk_api_version: version::Version,
}

impl Instance {
    pub fn new(config: &config::Config) -> Result<Self, Box<dyn Error>> {
        let entry = unsafe {
            Entry::new()?
        };
        let application_version = vk::make_version(
            config.app_version.major as u32,
            config.app_version.minor as u32,
            config.app_version.patch as u32,
        );
        let engine_version = vk::make_version(
            config.engine_version.major as u32,
            config.engine_version.minor as u32,
            config.engine_version.patch as u32,
        );
        let application_info = vk::ApplicationInfo {
            p_application_name: config.app_name.as_ptr() as *const i8,
            application_version,
            p_engine_name: config.engine_name.as_ptr() as *const i8,
            engine_version,
            api_version: vk::API_VERSION_1_2,
            ..Default::default()
        };
        let instance_create_info = vk::InstanceCreateInfo {
            p_application_info: &application_info,
            ..Default::default()
        };
        let vk_instance = unsafe {
            entry.create_instance(&instance_create_info, None)?
        }.handle();

        println!("Instance was created!");
        Ok(Instance {
            vk_instance,
            vk_api_version: version::Version::default(),
        })
    }
}
