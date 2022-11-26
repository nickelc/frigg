use std::fmt;

pub fn msg<F>(f: &mut fmt::Formatter<'_>, body: F) -> fmt::Result
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    format_xml::write!(f,
        <FUSMsg>
            <FUSHdr><ProtoVer>1.0</ProtoVer></FUSHdr>
            <FUSBody>
                <Put>
                |f| body(f)?;
                </Put>
            </FUSBody>
        </FUSMsg>
    )
}

pub fn file_info(model: &str, region: &str, version: &str, check: &str) -> String {
    let body = |f: &mut fmt::Formatter<'_>| {
        format_xml::write!(f,
            <ACCESS_MODE><Data>"2"</Data></ACCESS_MODE>
            <BINARY_NATURE><Data>"1"</Data></BINARY_NATURE>
            <CLIENT_PRODUCT><Data>"Smart Switch"</Data></CLIENT_PRODUCT>
            <DEVICE_FW_VERSION><Data>{ version }</Data></DEVICE_FW_VERSION>
            <DEVICE_LOCAL_CODE><Data>{ region }</Data></DEVICE_LOCAL_CODE>
            <DEVICE_MODEL_NAME><Data>{ model }</Data></DEVICE_MODEL_NAME>
            <LOGIC_CHECK><Data>{ check }</Data></LOGIC_CHECK>
        )
    };

    format_xml::fmt(|f| msg(f, body)).to_string()
}

pub fn init_download(file: &str, check: &str) -> String {
    let body = |f: &mut fmt::Formatter<'_>| -> fmt::Result {
        format_xml::write!(f,
            <BINARY_FILE_NAME><Data>{ file }</Data></BINARY_FILE_NAME>
            <LOGIC_CHECK><Data>{ check }</Data></LOGIC_CHECK>
        )
    };
    format_xml::fmt(|f| msg(f, body)).to_string()
}
