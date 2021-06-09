use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result};
use std::str::FromStr;

use regex::Regex;

const SEMVER_PATTERN: &'static str = concat!(
    r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)",
    r"(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))",
    r"?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$",
);
lazy_static::lazy_static! {
    static ref SEMVER_REGEX: Regex = Regex::new(SEMVER_PATTERN).unwrap();
    static ref INT_REGEX: Regex = Regex::new(r"\d+").unwrap();
}

#[derive(Debug)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub postfix: String,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32, postfix: String) -> Self {
        Self {
            major,
            minor,
            patch,
            postfix,
        }
    }
}

impl FromStr for Version {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if !SEMVER_REGEX.is_match(s) {
            return Err("Given string is not a semver".into());
        }
        let mut end: usize = 0;
        let mut numbers = INT_REGEX.find_iter(s).filter_map(|int| {
            end = int.end();
            int.as_str().parse().ok()
        });
        Ok(Version {
            major: numbers.next().unwrap(),
            minor: numbers.next().unwrap(),
            patch: numbers.next().unwrap(),
            postfix: s[end..].to_string(),
        })
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{}.{}.{}{}",
            self.major, self.minor, self.patch, self.postfix,
        )
    }
}

impl Default for Version {
    fn default() -> Self {
        Self {
            major: 0,
            minor: 0,
            patch: 0,
            postfix: String::new(),
        }
    }
}
