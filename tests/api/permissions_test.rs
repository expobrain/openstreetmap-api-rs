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
        <osm version="0.6" generator="OpenStreetMap server">
            <permissions>
                <permission name="allow_read_prefs"/>
                <permission name="allow_read_gpx"/>
                <permission name="allow_write_gpx"/>
            </permissions>
        </osm>
        "#,
        vec![
            types::Permission {
                name: "allow_read_prefs".into(),
            },
            types::Permission {
                name: "allow_read_gpx".into(),
            },
            types::Permission {
                name: "allow_write_gpx".into(),
            },
        ]
    )
)]
#[actix_rt::test]
async fn test_get(
    no_credentials: types::Credentials,
    response_str: &str,
    expected: Vec<types::Permission>,
) {
    /*
    GIVEN an OSM client
    WHEN calling the permissions() function
    THEN returns a list of permissions for the current user
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/0.6/permissions"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), no_credentials);

    // WHEN
    let actual = client.permissions().await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}
