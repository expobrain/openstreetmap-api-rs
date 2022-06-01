#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod api;
pub mod errors;
pub mod types;

use errors::OpenstreetmapError;
use quick_xml::de::from_reader;
use quick_xml::se::to_string;
use reqwest::header::CONTENT_TYPE;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use url::Url;

pub const DEFAULT_VERSION: &str = "0.6";

#[derive(Debug, Clone)]
pub struct Openstreetmap {
    pub host: String,
    api_version: String,
    credentials: types::Credentials,
    client: reqwest::Client,
}

#[derive(Debug, Clone)]
struct RequestOptions {
    pub use_version: bool,
    pub use_auth: bool,
}

impl RequestOptions {
    pub fn new() -> Self {
        Self {
            use_version: false,
            use_auth: false,
        }
    }
    pub fn with_version(mut self) -> Self {
        self.use_version = true;
        self
    }
    pub fn with_auth(mut self) -> Self {
        self.use_auth = true;
        self
    }
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

    #[inline]
    pub async fn versions(&self) -> Result<Vec<String>, OpenstreetmapError> {
        api::versions::Versions::new(self).get().await
    }

    #[inline]
    pub async fn capabilities(&self) -> Result<types::CapabilitiesAndPolicy, OpenstreetmapError> {
        api::capabilities::Capabilities::new(self).get().await
    }

    #[inline]
    pub async fn map(&self, bbox: &types::BoundingBox) -> Result<types::Map, OpenstreetmapError> {
        api::map::Map::new(self).get(bbox).await
    }

    #[inline]
    pub async fn permissions(&self) -> Result<Vec<types::Permission>, OpenstreetmapError> {
        api::permissions::Permissions::new(self).get().await
    }

    #[inline]
    pub fn changeset(&self) -> api::changeset::Changeset {
        api::changeset::Changeset::new(self)
    }

    #[inline]
    pub fn nodes(&self) -> api::elements::Elements<types::Node> {
        api::elements::Elements::new(self)
    }

    #[inline]
    pub fn ways(&self) -> api::elements::Elements<types::Way> {
        api::elements::Elements::new(self)
    }

    #[inline]
    pub fn relations(&self) -> api::elements::Elements<types::Relation> {
        api::elements::Elements::new(self)
    }

    #[inline]
    pub fn user(&self) -> api::user::User {
        api::user::User::new(self)
    }

    #[inline]
    pub fn notes(&self) -> api::notes::Notes {
        api::notes::Notes::new(self)
    }

    #[inline]
    pub async fn changesets(
        &self,
        query: types::ChangesetQueryParams,
    ) -> Result<Vec<types::Changeset>, OpenstreetmapError> {
        api::changesets::Changesets::new(self).get(query).await
    }

    async fn request<S, D>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: types::RequestBody<S>,
        options: RequestOptions,
    ) -> Result<D, OpenstreetmapError>
    where
        S: Serialize,
        D: DeserializeOwned,
    {
        let mut url = Url::parse(&self.host)?.join("api/")?;

        if options.use_version {
            let version_path = format!("{}/", self.api_version);

            url = url.join(&version_path)?;
        }

        url = url.join(endpoint)?;
        debug!("url -> {:?}", url);

        let mut builder = self.client.request(method, url);

        if options.use_auth {
            builder = match self.credentials {
                types::Credentials::Basic(ref user, ref pass) => {
                    builder.basic_auth(user, Some(pass))
                }
                types::Credentials::None => return Err(OpenstreetmapError::CredentialsNeeded),
            };
        }

        builder = match body {
            types::RequestBody::Xml(payload) => builder
                .body(to_string(&payload)?.into_bytes())
                .header(CONTENT_TYPE, "text/xml"),
            types::RequestBody::Form(payload) => builder.form(&payload),
            types::RequestBody::RawForm(payload) => builder
                .body(payload)
                .header(CONTENT_TYPE, "application/x-www-form-urlencoded"),
            types::RequestBody::None => builder,
        };

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
