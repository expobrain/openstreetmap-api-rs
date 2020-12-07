#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod api;
mod errors;
mod types;

use errors::OpenstreetmapError;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;

use serde_xml_rs::from_reader;
use url::Url;

pub const DEFAULT_VERSION: &str = "0.6";

#[derive(Debug, Clone)]
pub struct Openstreetmap {
    pub host: String,
    api_version: String,
    credentials: types::Credentials,
    client: reqwest::Client,
}

impl Openstreetmap {
    pub fn new<T>(host: T, credentials: types::Credentials) -> Self
    where
        T: Into<String>,
    {
        Openstreetmap {
            host: host.into(),
            api_version: DEFAULT_VERSION.into(),
            credentials,
            client: reqwest::Client::new(),
        }
    }

    /// creates a new instance of a Openstreetmap client using a specified reqwest client
    pub fn from_client<H>(host: H, credentials: types::Credentials, client: reqwest::Client) -> Self
    where
        H: Into<String>,
    {
        Openstreetmap {
            host: host.into(),
            api_version: DEFAULT_VERSION.into(),
            credentials,
            client,
        }
    }

    pub async fn versions(&self) -> Result<Vec<String>, OpenstreetmapError> {
        Ok(api::versions::Versions::new(self).get().await?)
    }

    pub async fn capabilities(&self) -> Result<types::CapabilitiesAndPolicy, OpenstreetmapError> {
        Ok(api::capabilities::Capabilities::new(self).get().await?)
    }

    pub async fn map(&self, bbox: &types::BoundingBox) -> Result<types::Map, OpenstreetmapError> {
        Ok(api::map::Map::new(self).get(bbox).await?)
    }

    async fn request<D>(
        &self,
        method: reqwest::Method,
        version: Option<&str>,
        endpoint: &str,
        body: Option<Vec<u8>>,
    ) -> Result<D, OpenstreetmapError>
    where
        D: DeserializeOwned,
    {
        let mut url = Url::parse(&self.host)?.join("api/")?;

        if version.is_some() {
            let version_path = format!("{}/", version.unwrap().to_string());

            url = url.join(&version_path)?;
        }

        url = url.join(endpoint)?;
        debug!("url -> {:?}", url);

        let req = self.client.request(method, url);
        let mut builder = match self.credentials {
            types::Credentials::Basic(ref user, ref pass) => req.basic_auth(user, Some(pass)),
        };

        if let Some(payload) = body {
            builder = builder.body(payload)
        }

        let res = builder.send().await?;

        match res.status() {
            StatusCode::UNAUTHORIZED => Err(OpenstreetmapError::Unauthorized),
            StatusCode::METHOD_NOT_ALLOWED => Err(OpenstreetmapError::MethodNotAllowed),
            StatusCode::NOT_FOUND => Err(OpenstreetmapError::NotFound),
            client_err if client_err.is_client_error() => Err(OpenstreetmapError::Client {
                code: res.status(),
                error: res.text().await?,
            }),
            _ => Ok(from_reader(res.text().await?.as_bytes())?),
        }
    }
}
