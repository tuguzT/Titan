use jni::errors::Error;
use jni::objects::JObject;
use jni::JNIEnv;

use crate::config::{Config, Version};

pub fn get_config(env: JNIEnv, config: JObject) -> Result<Config, Error> {
    let name_obj = env.get_field(config, "name", "Ljava/lang/String;")?.l()?;
    let name = env.get_string(name_obj.into())?.into();

    let version_obj = env
        .get_field(config, "version", "Lcom/tuguzT/Version;")?
        .l()?;
    let version = get_version(env, version_obj)?;

    Ok(Config::new(name, version))
}

pub fn get_version(env: JNIEnv, version: JObject) -> Result<Version, Error> {
    let major = env.get_field(version, "major", "I")?.i()? as u32;
    let minor = env.get_field(version, "minor", "I")?.i()? as u32;
    let patch = env.get_field(version, "patch", "I")?.i()? as u32;
    let postfix = env
        .get_field(version, "postfix", "Ljava/lang/String;")?
        .l()?;
    let postfix = env.get_string(postfix.into())?.into();
    Ok(Version::new(major, minor, patch, postfix))
}
