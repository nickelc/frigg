use generic_array::{typenum::U16, GenericArray};
use md5::{Digest, Md5};

use crate::auth::calc_logic_check;
use crate::xml::{self, XmlExt};
use crate::Error;

#[derive(Debug)]
pub struct BinaryInfo {
    pub display_name: String,
    pub os_version: String,
    pub model_path: String,
    pub binary_name: String,
    pub binary_size: u64,
    pub version: String,
    pub decrypt_key: DecryptKey,
}

#[derive(Debug)]
pub enum DecryptKey {
    V2(GenericArray<u8, U16>),
    V4(GenericArray<u8, U16>),
    Unknown,
}

pub fn from_xml(model: &str, region: &str, xml: &str) -> Result<BinaryInfo, Error> {
    let doc = xml::parse(xml)?;

    let fields = doc
        .get_elem(&["FUSMsg", "FUSBody", "Put"])
        .ok_or("Missing element FUSMsg/FUSBody/Put")?;

    let binary_name = fields
        .get_elem_text(&["BINARY_NAME", "Data"])
        .ok_or("Missing element FUSMsg/FUSBody/Put/BINARY_NAME/Data")?
        .to_owned();
    let binary_size = fields
        .get_elem_text(&["BINARY_BYTE_SIZE", "Data"])
        .ok_or("Missing element FUSMsg/FUSBody/Put/BINARY_NAME/Data")?
        .parse()?;

    let version = doc
        .get_elem_text(&["FUSMsg", "FUSBody", "Results", "LATEST_FW_VERSION", "Data"])
        .ok_or("Missing element FUSMsg/FUSBody/Results/LATEST_FW_VERSION/Data")?
        .to_owned();

    let decrypt_key = match (
        binary_name.ends_with(".enc2"),
        binary_name.ends_with(".enc4"),
    ) {
        (true, _) => {
            let key = format!("{region}:{model}:{version}");
            let key = Md5::digest(key.as_bytes());
            DecryptKey::V2(key)
        }
        (_, true) => {
            let logic_value_factory = fields
                .get_elem_text(&["LOGIC_VALUE_FACTORY", "Data"])
                .ok_or("Missing element FUSMsg/FUSBody/Put/LOGIC_VALUE_FACTORY/Data")?;

            if logic_value_factory.is_empty() {
                tracing::warn!("logic value is empty");
            }
            let key = calc_logic_check(&version, logic_value_factory);
            let key = Md5::digest(key.as_bytes());
            DecryptKey::V4(key)
        }
        _ => DecryptKey::Unknown,
    };

    let display_name = fields
        .get_elem_text(&["DEVICE_MODEL_DISPLAYNAME", "Data"])
        .ok_or("Missing element FUSMsg/FUSBody/Put/DEVICE_MODEL_DISPLAYNAME/Data")?
        .to_owned();

    let os_version = fields
        .get_elem_text(&["CURRENT_OS_VERSION", "Data"])
        .ok_or("Missing element FUSMsg/FUSBody/Put/CURRENT_OS_VERSION/Data")?
        .to_owned();

    let model_path = fields
        .get_elem_text(&["MODEL_PATH", "Data"])
        .ok_or("Missing element FUSMsg/FUSBody/Put/MODEL_PATH/Data")?
        .to_owned();

    let info = BinaryInfo {
        display_name,
        os_version,
        model_path,
        binary_name,
        binary_size,
        version,
        decrypt_key,
    };
    Ok(info)
}
