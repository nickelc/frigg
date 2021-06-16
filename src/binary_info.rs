use std::borrow::Cow;

use generic_array::{typenum::U16, GenericArray};
use md5::{Digest, Md5};
use strong_xml::XmlRead;

use crate::auth::calc_logic_check;
use crate::Error;

macro_rules! data_struct {
    ($name:tt, $elem:expr) => {
        #[derive(Debug, XmlRead)]
        #[xml(tag = $elem)]
        struct $name<'a> {
            #[xml(flatten_text = "Data")]
            data: Cow<'a, str>,
        }

        impl<'a> std::ops::Deref for $name<'a> {
            type Target = Cow<'a, str>;

            fn deref(&self) -> &Self::Target {
                &self.data
            }
        }
    };
}

#[derive(Debug, XmlRead)]
#[xml(tag = "FUSMsg")]
struct Message<'a> {
    #[xml(child = "FUSBody")]
    body: Body<'a>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "FUSBody")]
struct Body<'a> {
    #[xml(child = "Results")]
    status: Status<'a>,
    #[xml(child = "Put")]
    put: Put<'a>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "Results")]
struct Status<'a> {
    #[xml(flatten_text = "Status")]
    value: Cow<'a, str>,
    #[xml(child = "LATEST_FW_VERSION")]
    version: LatestVersion<'a>,
}

data_struct!(LatestVersion, "LATEST_FW_VERSION");

#[derive(Debug, XmlRead)]
#[xml(tag = "Put")]
struct Put<'a> {
    #[xml(child = "DEVICE_MODEL_DISPLAYNAME")]
    display_name: DisplayName<'a>,
    #[xml(child = "CURRENT_OS_VERSION")]
    os_version: OsVersion<'a>,
    #[xml(child = "BINARY_NAME")]
    binary_name: BinaryName<'a>,
    #[xml(child = "BINARY_BYTE_SIZE")]
    binary_byte_size: BinaryByteSize<'a>,
    #[xml(child = "MODEL_PATH")]
    model_path: ModelPath<'a>,
    #[xml(child = "LOGIC_VALUE_FACTORY")]
    logic_value_factory: LogicValueFactory<'a>,
}

data_struct!(DisplayName, "DEVICE_MODEL_DISPLAYNAME");
data_struct!(OsVersion, "CURRENT_OS_VERSION");
data_struct!(BinaryName, "BINARY_NAME");
data_struct!(BinaryByteSize, "BINARY_BYTE_SIZE");
data_struct!(ModelPath, "MODEL_PATH");
data_struct!(LogicValueFactory, "LOGIC_VALUE_FACTORY");

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
    let msg = Message::from_str(xml)?;

    let binary_name = msg.body.put.binary_name.to_string();
    let binary_size = msg.body.put.binary_byte_size.parse()?;
    let version = msg.body.status.version.to_string();

    let decrypt_key = match (
        binary_name.ends_with(".enc2"),
        binary_name.ends_with(".enc4"),
    ) {
        (true, _) => {
            let key = format!("{}:{}:{}", region, model, version);
            let key = Md5::digest(key.as_bytes());
            DecryptKey::V2(key)
        }
        (_, true) => {
            if msg.body.put.logic_value_factory.is_empty() {
                tracing::warn!("logic value is empty");
            }
            let key = calc_logic_check(&version, &msg.body.put.logic_value_factory.data);
            let key = Md5::digest(key.as_bytes());
            DecryptKey::V4(key)
        }
        _ => DecryptKey::Unknown,
    };

    let info = BinaryInfo {
        display_name: msg.body.put.display_name.to_string(),
        os_version: msg.body.put.os_version.to_string(),
        model_path: msg.body.put.model_path.to_string(),
        binary_name,
        binary_size,
        version,
        decrypt_key,
    };
    Ok(info)
}
