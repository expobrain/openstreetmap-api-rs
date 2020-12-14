use crate::types;
use crate::Openstreetmap;
use crate::OpenstreetmapError;

#[derive(Debug, Deserialize)]
struct Version {
    #[serde(rename = "$value")]
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
                None,
                "versions",
                types::RequestBody::None,
            )
            .await?
            .versions
            .into_iter()
            .map(|v| v.version)
            .collect::<Vec<String>>();

        Ok(versions)
    }
}
