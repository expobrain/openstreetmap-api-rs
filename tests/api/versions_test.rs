use openstreetmap_api::types;
use openstreetmap_api::Openstreetmap;
use rstest::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::utils::no_credentials;

#[rstest(response_str, expected,
    case(
        r#"
        <osm generator="OpenStreetMap server" copyright="OpenStreetMap and contributors" attribution="http://www.openstreetmap.org/copyright" license="http://opendatacommons.org/licenses/odbl/1-0/">
            <api>
                <version>0.6</version>
            </api>
        </osm>
        "#,
        vec!["0.6".to_string()]
    )
)]
#[actix_rt::test]
async fn test_get(no_credentials: types::Credentials, response_str: &str, expected: Vec<String>) {
    /*
    GIVEN an OSM client
    WHEN calling the versions() function
    THEN returns a list of supported versions
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/versions"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), no_credentials);

    // WHEN
    let actual = client.versions().await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest(response_str, expected,
    case(
        r#"
        <osm generator="OpenStreetMap server" copyright="OpenStreetMap and contributors" attribution="http://www.openstreetmap.org/copyright" license="http://opendatacommons.org/licenses/odbl/1-0/">
            <api>
                <version>0.7</version>
            </api>
        </osm>
        "#,
        vec!["0.7".to_string()]
    )
)]
#[actix_rt::test]
async fn test_get_returns_unknown_version(
    no_credentials: types::Credentials,
    response_str: &str,
    expected: Vec<String>,
) {
    /*
    GIVEN an OSM client
        AND an unknown version number
    WHEN calling the versions() function
    THEN returns a list of supported versions
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/versions"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), no_credentials);

    // WHEN
    let actual = client.versions().await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}
