use xml::writer::{EmitterConfig, Error as XmlError, EventWriter, XmlEvent};

type XmlWriter<'a> = EventWriter<&'a mut Vec<u8>>;

fn fus_xml<F>(body: F) -> Result<String, crate::Error>
where
    F: Fn(&mut XmlWriter) -> Result<(), XmlError>,
{
    let mut buf = Vec::new();
    let mut w = EmitterConfig::new().create_writer(&mut buf);
    w.write(XmlEvent::start_element("FUSMsg"))?;
    w.write(XmlEvent::start_element("FUSHdr"))?;
    w.write(XmlEvent::start_element("ProtoVer"))?;
    w.write("1.0")?;
    w.write(XmlEvent::end_element())?; // ProtoVer
    w.write(XmlEvent::end_element())?; // FUSHdr

    w.write(XmlEvent::start_element("FUSBody"))?;
    w.write(XmlEvent::start_element("Put"))?;

    body(&mut w)?;

    w.write(XmlEvent::end_element())?; // Put
    w.write(XmlEvent::end_element())?; // FUSBody
    w.write(XmlEvent::end_element())?; // FUSMsg

    String::from_utf8(buf).map_err(Into::into)
}

fn fus_attr(w: &mut XmlWriter<'_>, name: &str, data: &str) -> Result<(), XmlError> {
    w.write(XmlEvent::start_element(name))?;
    w.write(XmlEvent::start_element("Data"))?;
    w.write(data)?;
    w.write(XmlEvent::end_element())?; // Data
    w.write(XmlEvent::end_element()) // $name
}

pub fn file_info(model: &str, imei: &str, region: &str, version: &str, check: &str) -> String {
    fus_xml(|w| {
        fus_attr(w, "ACCESS_MODE", "2")?;
        fus_attr(w, "BINARY_NATURE", "1")?;
        fus_attr(w, "CLIENT_PRODUCT", "Smart Switch")?;
        fus_attr(w, "DEVICE_IMEI_PUSH", imei)?;
        fus_attr(w, "DEVICE_FW_VERSION", version)?;
        fus_attr(w, "DEVICE_LOCAL_CODE", region)?;
        fus_attr(w, "DEVICE_MODEL_NAME", model)?;
        fus_attr(w, "LOGIC_CHECK", check)
    })
    .unwrap()
}

pub fn init_download(file: &str, check: &str) -> String {
    fus_xml(|w| {
        fus_attr(w, "BINARY_FILE_NAME", file)?;
        fus_attr(w, "LOGIC_CHECK", check)
    })
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    use xml::writer::EmitterConfig;

    #[test]
    fn empty_fus_xml() {
        let xml = fus_xml(|_| Ok(())).unwrap();
        assert_eq!(
            xml,
            r#"<?xml version="1.0" encoding="utf-8"?><FUSMsg><FUSHdr><ProtoVer>1.0</ProtoVer></FUSHdr><FUSBody><Put /></FUSBody></FUSMsg>"#
        );
    }

    #[test]
    fn test_fus_xml() {
        let xml = fus_xml(|w| fus_attr(w, "Foo", "Bar")).unwrap();
        assert_eq!(
            xml,
            r#"<?xml version="1.0" encoding="utf-8"?><FUSMsg><FUSHdr><ProtoVer>1.0</ProtoVer></FUSHdr><FUSBody><Put><Foo><Data>Bar</Data></Foo></Put></FUSBody></FUSMsg>"#
        );
    }

    #[test]
    fn test_fus_attr() {
        let mut buf = Vec::new();
        let mut w = EmitterConfig::new().create_writer(&mut buf);

        let _ = fus_attr(&mut w, "Foo", "Bar");

        let xml = String::from_utf8(buf).unwrap();
        assert_eq!(
            xml,
            r#"<?xml version="1.0" encoding="utf-8"?><Foo><Data>Bar</Data></Foo>"#
        );
    }
}
