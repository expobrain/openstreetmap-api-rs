use crate::types;
use crate::Openstreetmap;
use crate::OpenstreetmapError;

#[derive(Debug, Deserialize)]
struct Osm {
    pub api: Api,
    pub policy: types::Policy,
}

impl Into<types::CapabilitiesAndPolicy> for Osm {
    fn into(self) -> types::CapabilitiesAndPolicy {
        types::CapabilitiesAndPolicy {
            capabilities: types::Capabilities {
                versions: types::VersionRange {
                    minimum: self.api.version.minimum,
                    maximum: self.api.version.maximum,
                },
                maximum_area: self.api.area.maximum,
                maximum_note_area: self.api.note_area.maximum,
                tracepoints_per_page: self.api.tracepoints.per_page,
                maximum_waynodes: self.api.waynodes.maximum,
                maximum_changeset_elements: self.api.changesets.maximum_elements,
                timeout: self.api.timeout.seconds,
                status: types::Status {
                    database: self.api.status.database,
                    api: self.api.status.api,
                    gpx: self.api.status.gpx,
                },
            },
            policy: self.policy,
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
    pub minimum: String,
    pub maximum: String,
}

#[derive(Debug, Deserialize)]
struct Area {
    pub maximum: f64,
}

#[derive(Debug, Deserialize)]
struct NoteArea {
    pub maximum: f64,
}

#[derive(Debug, Deserialize)]
struct Tracepoints {
    pub per_page: u64,
}

#[derive(Debug, Deserialize)]
struct Waynodes {
    pub maximum: u64,
}

#[derive(Debug, Deserialize)]
struct Changesets {
    pub maximum_elements: u64,
}

#[derive(Debug, Deserialize)]
struct Timeout {
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
                None,
                "capabilities",
                types::RequestBody::None,
            )
            .await?
            .into();

        Ok(capabilities_and_policies)
    }
}
