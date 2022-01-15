use openstreetmap_api::types;
use openstreetmap_api::Openstreetmap;
use pretty_assertions::assert_eq;
use rstest::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::utils::no_credentials;

#[rstest(response_str, expected,
    case(
        r#"
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
        "#,
        types::CapabilitiesAndPolicy {
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
        }
    )
)]
#[actix_rt::test]
async fn test_get(
    no_credentials: types::Credentials,
    response_str: &str,
    expected: types::CapabilitiesAndPolicy,
) {
    /*
    GIVEN an OSM client
    WHEN calling the capabilities() function
    THEN returns the sets of capablities and policies
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/capabilities"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), no_credentials);

    // WHEN
    let actual = client.capabilities().await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}
