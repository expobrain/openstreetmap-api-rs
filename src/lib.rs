#[macro_use]
extern crate log;

mod errors;

use errors::Error;
use reqwest::header::CONTENT_TYPE;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use std::fmt;

#[derive(Debug)]
enum ApiVersion {
    V6,
}

impl fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::V6 => "0.6",
        };

        write!(f, "{}", value)
    }
}
#[derive(Debug)]
pub enum Credentials {
    Basic(String, String), // Username, password
}

#[derive(Debug)]
pub struct Openstreetmap {
    pub host: String,
    api_version: ApiVersion,
    credentials: Credentials,
    client: reqwest::Client,
}

impl Openstreetmap {
    pub fn new<T>(host: T, credentials: Credentials) -> Self
    where
        T: Into<String>,
    {
        Openstreetmap {
            host: host.into(),
            api_version: ApiVersion::V6,
            credentials,
            client: reqwest::Client::new(),
        }
    }

    /// creates a new instance of a Openstreetmap client using a specified reqwest client
    pub fn from_client<H>(host: H, credentials: Credentials, client: reqwest::Client) -> Self
    where
        H: Into<String>,
    {
        Openstreetmap {
            host: host.into(),
            api_version: ApiVersion::V6,
            credentials,
            client,
        }
    }

    async fn request<D>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<Vec<u8>>,
    ) -> Result<D, Error>
    where
        D: DeserializeOwned,
    {
        let url = format!("{}/api/{}/{}", self.host, self.api_version, endpoint);
        debug!("url -> {:?}", url);

        let req = self.client.request(method, &url);
        let mut builder = match self.credentials {
            Credentials::Basic(ref user, ref pass) => req
                .basic_auth(user, Some(pass))
                .header(CONTENT_TYPE, "application/json"),
        };

        if let Some(payload) = body {
            builder = builder.body(payload)
        }

        let res = builder.send().await.map_err(|e| Error::Http(e))?;

        match res.status() {
            StatusCode::UNAUTHORIZED => Err(Error::Unauthorized),
            StatusCode::METHOD_NOT_ALLOWED => Err(Error::MethodNotAllowed),
            StatusCode::NOT_FOUND => Err(Error::NotFound),
            client_err if client_err.is_client_error() => Err(Error::Client {
                code: res.status(),
                error: res.text().await.map_err(|e| Error::Http(e))?,
            }),
            _ => Ok(res.json::<D>().await.map_err(|e| Error::Http(e))?),
        }
    }
}
