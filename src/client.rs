use anyhow::anyhow;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, COOKIE, USER_AGENT};
use reqwest::Response;

use crate::auth::{calc_logic_check, Nonce, Session};
use crate::binary_info::{self, BinaryInfo};
use crate::requests;
use crate::Error;

pub struct Client {
    inner: reqwest::Client,
}

impl Client {
    pub fn new() -> Result<Self, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Kies2.0_FUS"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .build()?;

        Ok(Self { inner: client })
    }

    pub async fn fetch_version(&self, model: &str, region: &str) -> Result<String, Error> {
        let url =
            format!("https://fota-cloud-dn.ospserver.net/firmware/{region}/{model}/version.xml",);
        let resp = self.inner.get(url).send().await?;
        let xml = resp.error_for_status()?.text().await?;

        tracing::debug!(request = "fetch_version", "{xml}");

        Ok(crate::version::from_xml(&xml)?)
    }

    pub async fn begin_session(&self) -> Result<Session, Error> {
        let url = "http://cloud-neofussvr.samsungmobile.com/NF_DownloadGenerateNonce.do";
        let resp = self
            .inner
            .get(url)
            .header(AUTHORIZATION, r#"FUS newauth="1""#)
            .header(COOKIE, r#"JSESSIONID="#)
            .send()
            .await?;

        let nonce = resp
            .headers()
            .get("NONCE")
            .ok_or_else(|| anyhow!("missing nonce header"))
            .and_then(Nonce::try_from)?;
        let id = resp
            .cookies()
            .find(|c| c.name() == "JSESSIONID")
            .map(|c| c.value().to_owned())
            .ok_or_else::<Error, _>(|| anyhow!("missing JSESSIONID cookie"))?;

        Ok(Session { nonce, id })
    }

    pub async fn file_info(
        &self,
        model: &str,
        imei: &str,
        region: &str,
        version: &str,
        session: &mut Session,
    ) -> Result<BinaryInfo, Error> {
        let check = calc_logic_check(version, &session.nonce.value);

        let data = requests::file_info(model, imei, region, version, &check);
        let xml = self
            .request("NF_DownloadBinaryInform.do", data, session)
            .await?
            .error_for_status()?
            .text()
            .await?;

        tracing::debug!(request = "file_info", "{xml}");

        binary_info::from_xml(model, region, &xml)
    }

    pub async fn download(
        &self,
        info: &BinaryInfo,
        session: &mut Session,
    ) -> Result<Response, Error> {
        self.init_download(&info.binary_name, session)
            .await?
            .error_for_status()?;

        let url = format!(
            "http://cloud-neofussvr.samsungmobile.com/NF_DownloadBinaryInitForMass.do?file={}{}",
            info.model_path, info.binary_name
        );
        let auth = format!(
            r#"FUS nonce="{}", signature="{}", type="", nc="", realm="", newauth="1""#,
            session.nonce.encoded, session.nonce.signature
        );
        let resp = self
            .inner
            .get(url)
            .header(reqwest::header::AUTHORIZATION, auth)
            .send()
            .await?
            .error_for_status()?;
        Ok(resp)
    }

    async fn init_download(
        &self,
        filename: &str,
        session: &mut Session,
    ) -> Result<Response, Error> {
        let basename = filename
            .split_once('.')
            .map(|(s, _)| s)
            .unwrap_or_else(|| filename);

        let basename = &basename[basename.len() - 16..];
        let check = calc_logic_check(basename, &session.nonce.value);

        let data = requests::init_download(filename, &check);

        self.request("NF_DownloadBinaryInitForMass.do", data, session)
            .await
    }

    async fn request(
        &self,
        path: &str,
        data: String,
        session: &mut Session,
    ) -> Result<Response, Error> {
        let url = format!("http://cloud-neofussvr.samsungmobile.com/{path}");

        let auth = format!(
            r#"FUS nonce="", signature="{}", type="", nc="", realm="", newauth="1""#,
            session.nonce.signature
        );
        let cookie = format!("JSESSIONID={}", session.id);
        let resp = self
            .inner
            .post(url)
            .header(reqwest::header::AUTHORIZATION, auth)
            .header(reqwest::header::COOKIE, cookie)
            .body(data)
            .send()
            .await?;

        if let Some(nonce) = resp.headers().get("NONCE") {
            session.nonce = Nonce::try_from(nonce)?;
        }

        Ok(resp)
    }
}
