use crate::types;
use crate::Openstreetmap;
use crate::OpenstreetmapError;
use crate::RequestOptions;

#[derive(Debug, Deserialize)]
struct Version {
    pub version: String,
}

#[derive(Debug, Deserialize)]
struct Osm {
    #[serde(rename = "api", default)]
    pub versions: Vec<Version>,
}

pub struct Versions {
    client: Openstreetmap,
}

impl Versions {
    pub fn new(client: &Openstreetmap) -> Self {
        Versions {
            client: client.clone(),
        }
    }

    pub async fn get(&self) -> Result<Vec<String>, OpenstreetmapError> {
        let versions = self
            .client
            .request::<(), Osm>(
                reqwest::Method::GET,
                "versions",
                types::RequestBody::None,
                RequestOptions::new(),
            )
            .await?
            .versions
            .into_iter()
            .map(|v| v.version)
            .collect::<Vec<String>>();

        Ok(versions)
    }
}
