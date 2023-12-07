use std::fmt;

use crate::xml::{self, XmlExt};

#[derive(Debug)]
pub enum Error {
    XmlError(roxmltree::Error),
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
    let doc = xml::parse(xml).map_err(Error::XmlError)?;

    let version = doc
        .get_elem_text(&["versioninfo", "firmware", "version", "latest"])
        .filter(|v| !v.is_empty())
        .ok_or(Error::InvalidVersion)?;

    let mut ver = version.split('/').collect::<Vec<_>>();
    if ver.len() == 3 {
        ver.push(ver[0]);
    }
    if ver[2].is_empty() {
        ver[2] = ver[0];
    }
    Ok(ver.join("/"))
}
