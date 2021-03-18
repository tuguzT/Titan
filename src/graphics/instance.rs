use std::error::Error;
use std::ffi::CString;

use ash::{Entry, vk};
use ash::version::{EntryV1_0, InstanceV1_0};

use crate::config::Config;
use crate::graphics::utils;
use crate::version::Version;

pub struct Instance {
    vk_instance: vk::Instance,
    vk_version: Version,
    entry: Entry,
}

impl Instance {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let entry = unsafe {
            Entry::new()?
        };
        let application_version = to_vk_version(&config.app_version());
        let engine_version = to_vk_version(&config.engine_version());
        let vk_version = match entry.try_enumerate_instance_version() {
            Ok(version) => from_vk_version(version.unwrap_or(0)),
            Err(_) => Version::default()
        };
        let application_info = vk::ApplicationInfo {
            p_application_name: config.app_name_c().as_ptr(),
            application_version,
            p_engine_name: config.engine_name_c().as_ptr(),
            engine_version,
            api_version: vk::API_VERSION_1_2,
            ..Default::default()
        };
        let instance_create_info = vk::InstanceCreateInfo {
            p_application_info: &application_info,
            ..Default::default()
        };
        let vk_instance: vk::Instance = unsafe {
            entry.create_instance(&instance_create_info, None)?
        }.handle();

        println!("Instance was created! {:#?}", vk_version);
        Ok(Self {
            vk_instance,
            vk_version,
            entry,
        })
    }

    fn vk_version(&self) -> &Version {
        &self.vk_version
    }

    pub fn entry(&self) -> &Entry {
        &self.entry
    }

    pub fn vk_instance(&self) -> vk::Instance {
        self.vk_instance
    }
}

#[inline]
fn to_vk_version(version: &Version) -> u32 {
    vk::make_version(
        version.major,
        version.minor,
        version.patch,
    )
}

#[inline]
fn from_vk_version(version: u32) -> Version {
    Version::new(
        vk::version_major(version),
        vk::version_minor(version),
        vk::version_patch(version),
    )
}
