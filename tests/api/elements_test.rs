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
async fn test_create_element(
    credentials: types::Credentials,
    node: types::Node,
    response_str: &str,
    expected: u64,
) {
    /*
    GIVEN an OSM client
    WHEN calling the create() function
    THEN returns the ID of the created node
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

#[rstest(element_id, response_str, expected,
    case(
        1234,
        r#"
        <osm>
            <node id="1234" changeset="42" version="2" lat="12.1234567" lon="-8.7654321" timestamp="2009-12-09T08:19:00Z" uid="1" user="user" visible="true">
                <tag k="amenity" v="school"/>
            </node>
        </osm>
        "#,
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
    )
)]
#[actix_rt::test]
async fn test_get(
    credentials: types::Credentials,
    element_id: u64,
    response_str: &str,
    expected: types::Node,
) {
    /*
    GIVEN an OSM client
    WHEN calling the get() function
    THEN returns the node
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/0.6/node/{}", element_id)))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.nodes().get(element_id).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest(element, response_str, expected,
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
async fn test_update_element(
    credentials: types::Credentials,
    element: types::Node,
    response_str: &str,
    expected: u64,
) {
    /*
    GIVEN an OSM client
    WHEN calling the update() function
    THEN returns the ID of the updated node
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path(format!("/api/0.6/node/{}", element.id)))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "text/plain"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.nodes().update(element).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest(element, response_str, expected,
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
async fn test_delete_element(
    credentials: types::Credentials,
    element: types::Node,
    response_str: &str,
    expected: u64,
) {
    /*
    GIVEN an OSM client
    WHEN calling the delete() function
    THEN returns the ID of the updated node
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path(format!("/api/0.6/node/{}", element.id)))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "text/plain"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.nodes().delete(element).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}
