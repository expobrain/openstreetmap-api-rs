use openstreetmap_api::types;
use openstreetmap_api::Openstreetmap;
use pretty_assertions::assert_eq;
use rstest::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::utils::credentials;

#[rstest(node, response_str, expected,
    case(
        types::Node {
            id: 1234,
            changeset: 42,
            version: 2,
            uid: 1,
            timestamp: "2009-12-09T08:19:00Z".into(),
            user: "user".into(),
            visible: true,
            lat: 12.1234567,
            lon: -8.7654321,
            tags: vec![types::Tag {
                k: "amenity".into(),
                v: "school".into(),
            }],
        },
        "10",
        10
    )
)]
#[actix_rt::test]
async fn test_create_node(
    credentials: types::Credentials,
    node: types::Node,
    response_str: &str,
    expected: u64,
) {
    /*
    GIVEN an OSM client
    WHEN calling the create() function
    THEN returns the list of nodes, ways and relations inside the bbox
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/0.6/node/create"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.nodes().create(node).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}
