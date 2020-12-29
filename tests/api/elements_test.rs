use openstreetmap_api::types;
use openstreetmap_api::Openstreetmap;
use pretty_assertions::assert_eq;
use rstest::*;
use wiremock::matchers::{method, path, query_param, QueryParamExactMatcher};
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
        vec![types::Node {
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
        }],
    )
)]
#[actix_rt::test]
async fn test_history(
    credentials: types::Credentials,
    element_id: u64,
    response_str: &str,
    expected: Vec<types::Node>,
) {
    /*
    GIVEN an OSM client
    WHEN calling the get() function
    THEN returns the node
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/0.6/node/{}/history", element_id)))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.nodes().history(element_id).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest(element_id, version_id, response_str, expected,
    case(
        1234,
        1,
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
async fn test_version(
    credentials: types::Credentials,
    element_id: u64,
    version_id: u64,
    response_str: &str,
    expected: types::Node,
) {
    /*
    GIVEN an OSM client
    WHEN calling the version() function
    THEN returns the node at specified version
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/0.6/node/{}/{}", element_id, version_id)))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client
        .nodes()
        .version(element_id, version_id)
        .await
        .unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest(element_id_params, request_qs, response_str, expected,
    case(
        vec![types::ElementIdParam::new(1234, None)],
        query_param("nodes", "1234"),
        r#"
        <osm>
            <node id="1234" changeset="42" version="2" lat="12.1234567" lon="-8.7654321" timestamp="2009-12-09T08:19:00Z" uid="1" user="user" visible="true">
                <tag k="amenity" v="school"/>
            </node>
        </osm>
        "#,
        vec![types::Node {
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
        }],
    ),
    case(
        vec![types::ElementIdParam::new(1234, Some(2))],
        query_param("nodes", "1234v2"),
        r#"
        <osm>
            <node id="1234" changeset="42" version="2" lat="12.1234567" lon="-8.7654321" timestamp="2009-12-09T08:19:00Z" uid="1" user="user" visible="true">
                <tag k="amenity" v="school"/>
            </node>
        </osm>
        "#,
        vec![types::Node {
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
        }],
    ),
    case(
        vec![
            types::ElementIdParam::new(1234, None),
            types::ElementIdParam::new(2000, None)
        ],
        query_param("nodes", "1234,2000"),
        r#"
        <osm>
            <node id="1234" changeset="42" version="2" lat="12.1234567" lon="-8.7654321" timestamp="2009-12-09T08:19:00Z" uid="1" user="user" visible="true">
                <tag k="amenity" v="school"/>
            </node>
            <node id="2000" changeset="42" version="2" lat="12.1234567" lon="-8.7654321" timestamp="2009-12-09T08:19:00Z" uid="1" user="user" visible="true" />
        </osm>
        "#,
        vec![
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
            types::Node {
                id: 2000,
                changeset: 42,
                version: 2,
                uid: 1,
                timestamp: "2009-12-09T08:19:00Z".into(),
                user: "user".into(),
                visible: true,
                lat: 12.1234567,
                lon: -8.7654321,
                tags: vec![],
            },
        ],
    )
)]
#[actix_rt::test]
async fn test_multi_get(
    credentials: types::Credentials,
    element_id_params: Vec<types::ElementIdParam>,
    request_qs: QueryParamExactMatcher,
    response_str: &str,
    expected: Vec<types::Node>,
) {
    /*
    GIVEN an OSM client
    WHEN calling the multi_get() function
    THEN returns the list of nodes
    */
    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/0.6/nodes/")))
        .and(request_qs)
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.nodes().multi_get(element_id_params).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest(element_id, response_str, expected,
    case(
        4507,
        r#"
        <osm>
            <relation id="4507" visible="true" version="1" changeset="3198" timestamp="2010-02-25T19:52:18Z" user="rus" uid="96">
                <member type="way" ref="80976" role="outer"/>
            </relation>
        </osm>
        "#,
        vec![types::Relation {
            id: 4507,
            visible: true,
            version: 1,
            changeset: 3198,
            timestamp: "2010-02-25T19:52:18Z".into(),
            user: "rus".into(),
            uid: 96,
            tags: vec![],
            members: vec![types::Member {
                member_type: "way".into(),
                node_id: 80976,
                role: "outer".into(),
            }],
        }],
    )
)]
#[actix_rt::test]
async fn test_relations(
    credentials: types::Credentials,
    element_id: u64,
    response_str: &str,
    expected: Vec<types::Relation>,
) {
    /*
    GIVEN an OSM client
    WHEN calling the relations() function
    THEN returns the element's relations
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/0.6/node/{}/relations", element_id)))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.nodes().relations(element_id).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest(node_id, response_str, expected,
    case(
        49780,
        r#"
        <osm>
            <way id="49780" visible="true" version="1" changeset="2308" timestamp="2009-12-09T08:51:50Z" user="guggis" uid="1">
                <nd ref="1150401"/>
            </way>
        </osm>
        "#,
        vec![types::Way {
            id: 49780,
            visible: true,
            version: 1,
            changeset: 2308,
            timestamp: "2009-12-09T08:51:50Z".into(),
            user: "guggis".into(),
            uid: 1,
            node_refs: vec![types::NodeRef { node_id: 1150401 }],
            tags: vec![],
        }],
    )
)]
#[actix_rt::test]
async fn test_ways(
    credentials: types::Credentials,
    node_id: u64,
    response_str: &str,
    expected: Vec<types::Way>,
) {
    /*
    GIVEN an OSM client
    WHEN calling the ways() function
    THEN returns the ways for a given node
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/0.6/node/{}/ways", node_id)))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.nodes().ways(node_id).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}
