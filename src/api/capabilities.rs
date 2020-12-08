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
    pub per_page: u32,
}

#[derive(Debug, Deserialize)]
struct Waynodes {
    pub maximum: u32,
}

#[derive(Debug, Deserialize)]
struct Changesets {
    pub maximum_elements: u32,
}

#[derive(Debug, Deserialize)]
struct Timeout {
    pub seconds: u32,
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
            .request::<(), Osm>(reqwest::Method::GET, None, "capabilities", None)
            .await?
            .into();

        Ok(capabilities_and_policies)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::Credentials;
    use crate::Openstreetmap;

    use lazy_static::lazy_static;
    use pretty_assertions::assert_eq;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    const CAPABILITIES_STR: &str = r#"
        <osm version="0.6" generator="OpenStreetMap server" copyright="OpenStreetMap and contributors" attribution="http://www.openstreetmap.org/copyright" license="http://opendatacommons.org/licenses/odbl/1-0/">
        <api>
            <version minimum="0.6" maximum="0.6"/>
            <area maximum="0.25"/>
            <note_area maximum="25"/>
            <tracepoints per_page="5000"/>
            <waynodes maximum="2000"/>
            <changesets maximum_elements="10000"/>
            <timeout seconds="300"/>
            <status database="online" api="online" gpx="online"/>
        </api>
        <policy>
            <imagery>
                <blacklist regex=".*\.google(apis)?\..*/(vt|kh)[\?/].*([xyz]=.*){3}.*"/>
                <blacklist regex="http://xdworld\.vworld\.kr:8080/.*"/>
                <blacklist regex=".*\.here\.com[/:].*"/>
            </imagery>
        </policy>
        </osm>
    "#;

    lazy_static! {
        static ref CREDENTIALS: Credentials = Credentials::Basic("user".into(), "password".into());
    }

    #[actix_rt::test]
    async fn test_get() {
        /*
        GIVEN an OSM client
        WHEN calling the capabilities() function
        THEN returns the sets of capablities and policies
        */

        // GIVEN
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/capabilities"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(CAPABILITIES_STR, "application/xml"),
            )
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), CREDENTIALS.clone());

        // WHEN
        let actual = client.capabilities().await.unwrap();

        // THEN
        let expected = types::CapabilitiesAndPolicy {
            capabilities: types::Capabilities {
                versions: types::VersionRange {
                    minimum: "0.6".into(),
                    maximum: "0.6".into(),
                },
                maximum_area: 0.25,
                maximum_note_area: 25.0,
                tracepoints_per_page: 5000,
                maximum_waynodes: 2000,
                maximum_changeset_elements: 10000,
                timeout: 300,
                status: types::Status {
                    database: "online".into(),
                    api: "online".into(),
                    gpx: "online".into(),
                },
            },
            policy: types::Policy {
                imagery: types::Imagery {
                    blacklist: vec![
                        types::Blacklist {
                            regex: r".*\.google(apis)?\..*/(vt|kh)[\?/].*([xyz]=.*){3}.*".into(),
                        },
                        types::Blacklist {
                            regex: r"http://xdworld\.vworld\.kr:8080/.*".into(),
                        },
                        types::Blacklist {
                            regex: r".*\.here\.com[/:].*".into(),
                        },
                    ],
                },
            },
        };

        assert_eq!(actual, expected);
    }
}
