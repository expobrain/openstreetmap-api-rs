use crate::types;
use crate::Openstreetmap;
use crate::OpenstreetmapError;
use crate::RequestOptions;

#[derive(Debug, Deserialize)]
struct Osm {
    pub api: Api,
    pub policy: types::Policy,
}

impl From<Osm> for types::CapabilitiesAndPolicy {
    fn from(value: Osm) -> types::CapabilitiesAndPolicy {
        types::CapabilitiesAndPolicy {
            capabilities: types::Capabilities {
                versions: types::VersionRange {
                    minimum: value.api.version.minimum,
                    maximum: value.api.version.maximum,
                },
                maximum_area: value.api.area.maximum,
                maximum_note_area: value.api.note_area.maximum,
                tracepoints_per_page: value.api.tracepoints.per_page,
                maximum_waynodes: value.api.waynodes.maximum,
                maximum_changeset_elements: value.api.changesets.maximum_elements,
                timeout: value.api.timeout.seconds,
                status: types::Status {
                    database: value.api.status.database,
                    api: value.api.status.api,
                    gpx: value.api.status.gpx,
                },
            },
            policy: value.policy,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Api {
    pub version: Version,
    pub area: Area,
    pub note_area: NoteArea,
    pub tracepoints: Tracepoints,
    pub waynodes: Waynodes,
    pub changesets: Changesets,
    pub timeout: Timeout,
    pub status: types::Status,
}

#[derive(Debug, Deserialize)]
struct Version {
    #[serde(rename = "@minimum")]
    pub minimum: String,
    #[serde(rename = "@maximum")]
    pub maximum: String,
}

#[derive(Debug, Deserialize)]
struct Area {
    #[serde(rename = "@maximum")]
    pub maximum: f64,
}

#[derive(Debug, Deserialize)]
struct NoteArea {
    #[serde(rename = "@maximum")]
    pub maximum: f64,
}

#[derive(Debug, Deserialize)]
struct Tracepoints {
    #[serde(rename = "@per_page")]
    pub per_page: u64,
}

#[derive(Debug, Deserialize)]
struct Waynodes {
    #[serde(rename = "@maximum")]
    pub maximum: u64,
}

#[derive(Debug, Deserialize)]
struct Changesets {
    #[serde(rename = "@maximum_elements")]
    pub maximum_elements: u64,
}

#[derive(Debug, Deserialize)]
struct Timeout {
    #[serde(rename = "@seconds")]
    pub seconds: u64,
}

pub struct Capabilities {
    client: Openstreetmap,
}

impl Capabilities {
    pub fn new(client: &Openstreetmap) -> Self {
        Capabilities {
            client: client.clone(),
        }
    }

    pub async fn get(&self) -> Result<types::CapabilitiesAndPolicy, OpenstreetmapError> {
        let capabilities_and_policies: types::CapabilitiesAndPolicy = self
            .client
            .request::<(), Osm>(
                reqwest::Method::GET,
                "capabilities",
                types::RequestBody::None,
                RequestOptions::new(),
            )
            .await?
            .into();

        Ok(capabilities_and_policies)
    }
}
