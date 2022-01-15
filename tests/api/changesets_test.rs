use openstreetmap_api::types;
use openstreetmap_api::Openstreetmap;
use pretty_assertions::assert_eq;
use rstest::*;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::utils::no_credentials;

#[fixture]
fn response_str() -> String {
    r#"
    <osm version="0.6" generator="OpenStreetMap server" copyright="OpenStreetMap and contributors" attribution="http://www.openstreetmap.org/copyright" license="http://opendatacommons.org/licenses/odbl/1-0/">
        <changeset id="188725" created_at="2020-12-09T22:51:17Z" open="false" comments_count="0" changes_count="3" closed_at="2020-12-09T22:51:18Z" min_lat="57.1444672" min_lon="-2.0845198" max_lat="57.1447233" max_lon="-2.0814377" uid="10723" user="expobrain">
            <tag k="comment" v="aaa"/>
        </changeset>
    </osm>
    "#.into()
}

#[rstest(expected,
    case(
        vec![types::Changeset {
            id: 188725,
            user: "expobrain".into(),
            uid: 10723,
            created_at: "2020-12-09T22:51:17Z".into(),
            closed_at: Some("2020-12-09T22:51:18Z".into()),
            open: false,
            min_lon: Some(-2.0845198),
            min_lat: Some(57.1444672),
            max_lon: Some(-2.0814377),
            max_lat: Some(57.1447233),
            discussion: None,
            tags: vec![types::Tag {
                k: "comment".into(),
                v: "aaa".into(),
            }],
        }]
    )
)]
#[actix_rt::test]
async fn test_get(
    no_credentials: types::Credentials,
    response_str: String,
    expected: Vec<types::Changeset>,
) {
    /*
    GIVEN an OSM client
    WHEN calling the get() function
    THEN returns the changeset
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/0.6/changesets"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), no_credentials);

    // WHEN
    let query = types::ChangesetQueryParams::default();
    let actual = client.changesets(query).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest]
#[actix_rt::test]
async fn test_get_with_query(no_credentials: types::Credentials, response_str: String) {
    /*
    GIVEN an OSM client
    WHEN calling the get() function with a non-default query
    THEN calls the enpoint with the rendered query
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/0.6/changesets"))
        .and(query_param("user", "123"))
        // .and(query_param("bbox", "1,2,3,4"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), no_credentials);

    // WHEN
    let query = types::ChangesetQueryParams {
        bbox: Some(types::BoundingBox {
            left: 1.0,
            bottom: 2.0,
            right: 3.0,
            top: 4.0,
        }),
        user_id: Some(123),
        display_name: None,
        closed_after: None,
        created_before: None,
        open: None,
        closed: None,
        changeset_ids: None,
    };

    let actual = client.changesets(query).await.unwrap();

    // THEN
    assert_eq!(actual.is_empty(), false);
}
