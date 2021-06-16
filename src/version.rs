use std::borrow::Cow;
use std::fmt;
use strong_xml::{XmlError, XmlRead};

#[derive(Debug, XmlRead)]
#[xml(tag = "versioninfo")]
pub(super) struct VersionInfo<'a> {
    #[xml(child = "firmware")]
    firmware: Firmware<'a>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "firmware")]
struct Firmware<'a> {
    #[xml(child = "version")]
    version: Version<'a>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "version")]
struct Version<'a> {
    #[xml(flatten_text = "latest")]
    latest: Cow<'a, str>,
}

#[derive(Debug)]
pub enum Error {
    XmlError(XmlError),
    InvalidVersion,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::XmlError(_) => f.write_str("xml deserialization error"),
            Self::InvalidVersion => f.write_str("invalid version string \"\""),
        }
    }
}

pub fn from_xml(xml: &str) -> Result<String, Error> {
    let info = VersionInfo::from_str(xml).map_err(Error::XmlError)?;
    let version = info.firmware.version.latest;
    if version.is_empty() {
        return Err(Error::InvalidVersion);
    }
    let mut ver = version.split('/').collect::<Vec<_>>();
    if ver.len() == 3 {
        ver.push(ver[0]);
    }
    if ver[2].is_empty() {
        ver[2] = ver[0];
    }
    Ok(ver.join("/"))
}
