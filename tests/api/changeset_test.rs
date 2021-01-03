use openstreetmap_api::types;
use openstreetmap_api::Openstreetmap;
use pretty_assertions::assert_eq;
use rstest::*;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::utils::credentials;

#[rstest(body, response_str, expected,
    case(
        types::ChangesetCreate::new(
            "0.6",
            "iD",
            vec![
                types::Tag::new("comment", "aaa"),
                types::Tag::new("created_by", "iD 2.19.5"),
                types::Tag::new("host", "https://master.apis.dev.openstreetmap.org/edit"),
                types::Tag::new("locale", "en_GB"),
                types::Tag::new("imagery", "Bing aerial imagery"),
                types::Tag::new("changeset_count", "1"),
            ]
        ),
        "188664",
        188664
    )
)]
#[actix_rt::test]
async fn test_create(
    credentials: types::Credentials,
    body: types::ChangesetCreate,
    response_str: &str,
    expected: u64,
) {
    /*
    GIVEN an OSM client
        AND a changeset
    WHEN calling the create() function
    THEN returns the created changeset ID
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/0.6/changeset/create"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "text/plain"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.changeset().create(vec![body]).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest(response_str, expected,
    case(
        r#"
        <osm>
            <changeset id="10" user="fred" uid="123" created_at="2008-11-08T19:07:39+01:00" open="true" min_lon="7.0191821" min_lat="49.2785426" max_lon="7.0197485" max_lat="49.2793101">
                <tag k="created_by" v="JOSM 1.61"/>
            </changeset>
        </osm>
        "#,
        types::Changeset {
            id: 10,
            user: "fred".into(),
            uid: 123,
            created_at: "2008-11-08T19:07:39+01:00".into(),
            closed_at: None,
            open: true,
            min_lon: Some(7.0191821),
            min_lat: Some(49.2785426),
            max_lon: Some(7.0197485),
            max_lat: Some(49.2793101),
            discussion: None,
            tags: vec![types::Tag {
                k: "created_by".into(),
                v: "JOSM 1.61".into(),
            }],
        }
    )
)]
#[actix_rt::test]
async fn test_get(credentials: types::Credentials, response_str: &str, expected: types::Changeset) {
    /*
    GIVEN an OSM client
    WHEN calling the get() function with a changeset ID
    THEN returns the requested changeset
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/0.6/changeset/10"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.changeset().get(10).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest(response_str, expected,
    case(
        r#"
        <osm>
            <changeset id="10" user="fred" uid="123" created_at="2008-11-08T19:07:39+01:00" open="true" min_lon="7.0191821" min_lat="49.2785426" max_lon="7.0197485" max_lat="49.2793101">
                <tag k="created_by" v="JOSM 1.61"/>
                <discussion>
                    <comment date="2015-01-01T18:56:48Z" uid="1841" user="metaodi">
                        <text>Did you verify those street names?</text>
                    </comment>
                </discussion>
            </changeset>
        </osm>
        "#,
        types::Changeset {
            id: 10,
            user: "fred".into(),
            uid: 123,
            created_at: "2008-11-08T19:07:39+01:00".into(),
            closed_at: None,
            open: true,
            min_lon: Some(7.0191821),
            min_lat: Some(49.2785426),
            max_lon: Some(7.0197485),
            max_lat: Some(49.2793101),
            discussion: Some(types::Discussion {
                comments: vec![types::Comment {
                    date: "2015-01-01T18:56:48Z".into(),
                    uid: 1841,
                    user: "metaodi".into(),
                    text: "Did you verify those street names?".into(),
                }],
            }),
            tags: vec![types::Tag {
                k: "created_by".into(),
                v: "JOSM 1.61".into(),
            }],
        }
    )
)]
#[actix_rt::test]
async fn test_get_with_discussion(
    credentials: types::Credentials,
    response_str: &str,
    expected: types::Changeset,
) {
    /*
    GIVEN an OSM client
    WHEN calling the get() function with a changeset ID
        AND includes discussion
    THEN returns the requested changeset
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/0.6/changeset/10"))
        .and(query_param("include_discussion", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.changeset().get_with_discussion(10).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest(body, response_str, expected,
    case(
        vec![types::Tag::new("created_by", "JOSM 1.61")],
        r#"
        <osm>
            <changeset id="10" user="fred" uid="123" created_at="2008-11-08T19:07:39+01:00" open="true" min_lon="7.0191821" min_lat="49.2785426" max_lon="7.0197485" max_lat="49.2793101">
                <tag k="created_by" v="JOSM 1.61"/>
            </changeset>
        </osm>
        "#,
        types::Changeset {
            id: 10,
            user: "fred".into(),
            uid: 123,
            created_at: "2008-11-08T19:07:39+01:00".into(),
            closed_at: None,
            open: true,
            min_lon: Some(7.0191821),
            min_lat: Some(49.2785426),
            max_lon: Some(7.0197485),
            max_lat: Some(49.2793101),
            discussion: None,
            tags: vec![types::Tag {
                k: "created_by".into(),
                v: "JOSM 1.61".into(),
            }],
        }
    )
)]
#[actix_rt::test]
async fn test_update_tags_on_changeset(
    credentials: types::Credentials,
    body: Vec<types::Tag>,
    response_str: &str,
    expected: types::Changeset,
) {
    /*
    GIVEN an OSM client
    WHEN calling the update_tags_on_changeset() function with a changeset ID
        AND a list of tags
    THEN returns the updated changeset
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/0.6/changeset/10"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client
        .changeset()
        .update_tags_on_changeset(10, body.clone())
        .await
        .unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest]
#[actix_rt::test]
async fn test_close(credentials: types::Credentials) {
    /*
    GIVEN an OSM client
    WHEN calling the close() function with a changeset ID
    THEN returns nothing
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/0.6/changeset/10/close"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.changeset().close(10).await.unwrap();

    // THEN
    let expected = ();

    assert_eq!(actual, expected);
}

#[rstest(response_str, expected,
    case(
        r#"
        <osmChange version="0.6" generator="acme osm editor">
            <modify>
                <node id="1234" changeset="42" version="2" lat="12.1234567" lon="-8.7654321" timestamp="2009-12-09T08:19:00Z" uid="1" user="user" visible="true">
                    <tag k="amenity" v="school"/>
                </node>
            </modify>
        </osmChange>
        "#,
        types::ChangesetChanges {
            modifications: vec![types::Modification {
                nodes: vec![types::Node {
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
                ways: vec![],
                relations: vec![],
            }],
            creations: vec![],
            deletions: vec![],
        }
    )
)]
#[actix_rt::test]
async fn test_download(
    credentials: types::Credentials,
    response_str: &str,
    expected: types::ChangesetChanges,
) {
    /*
    GIVEN an OSM client
    WHEN calling the download() function with a changeset ID
        AND a list of tags
    THEN returns the updated changeset
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/0.6/changeset/10/download"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.changeset().download(10).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest( body, response_str, expected,
    case(
        types::ChangesetChanges {
            modifications: vec![types::Modification {
                nodes: vec![types::Node {
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
                ways: vec![],
                relations: vec![],
            }],
            creations: vec![],
            deletions: vec![],
        },
        r#"
        <diffResult generator="OpenStreetMap Server" version="0.6">
            <node old_id="1234" new_id="42" new_version="2" />
            <way old_id="1234" new_id="42" new_version="2" />
            <relation old_id="1234" new_id="42" new_version="2" />
        </diffResult>
        "#,
        types::DiffResult {
            nodes:vec![types::DiffNode { old_id:1234, new_id:42, new_version:2 }],
            ways:vec![types::DiffWay { old_id:1234, new_id:42, new_version:2 }],
            relations:vec![types::DiffRelation { old_id:1234, new_id:42, new_version:2 }],
        }
    ),
)]
#[actix_rt::test]
async fn test_upload(
    credentials: types::Credentials,
    body: types::ChangesetChanges,
    response_str: &str,
    expected: types::DiffResult,
) {
    /*
    GIVEN an OSM client
    WHEN calling the upload() function with a changeset ID
        AND a ChangesetChange
    THEN returns the list of diffs
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/0.6/changeset/10/upload"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.changeset().upload(10, body).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest(changeset_id, comment, case(10, "my_comment"))]
#[actix_rt::test]
async fn test_comment(credentials: types::Credentials, changeset_id: u64, comment: &str) {
    /*
    GIVEN an OSM client
    WHEN calling the comment() function with a changeset ID
    THEN returns nothing
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(format!("/api/0.6/changeset/{}/comment", changeset_id)))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client
        .changeset()
        .comment(changeset_id, comment)
        .await
        .unwrap();

    // THEN
    assert_eq!(actual, ());
}

#[rstest(changeset_id, response_str, expected,
    case(
        10,
        r#"
        <osm>
            <changeset id="10" user="fred" uid="123" created_at="2008-11-08T19:07:39+01:00" open="true" min_lon="7.0191821" min_lat="49.2785426" max_lon="7.0197485" max_lat="49.2793101">
                <tag k="created_by" v="JOSM 1.61"/>
            </changeset>
        </osm>
        "#,
        types::Changeset {
            id: 10,
            user: "fred".into(),
            uid: 123,
            created_at: "2008-11-08T19:07:39+01:00".into(),
            closed_at: None,
            open: true,
            min_lon: Some(7.0191821),
            min_lat: Some(49.2785426),
            max_lon: Some(7.0197485),
            max_lat: Some(49.2793101),
            discussion: None,
            tags: vec![types::Tag {
                k: "created_by".into(),
                v: "JOSM 1.61".into(),
            }],
        }
    )
)]
#[actix_rt::test]
async fn test_subscribe(
    credentials: types::Credentials,
    changeset_id: u64,
    response_str: &str,
    expected: types::Changeset,
) {
    /*
    GIVEN an OSM client
    WHEN calling the subscribe() function with a changeset ID
    THEN returns the subscribed changeset
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(format!(
            "/api/0.6/changeset/{}/subscribe",
            changeset_id
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.changeset().subscribe(changeset_id).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest(changeset_id, response_str, expected,
    case(
        10,
        r#"
        <osm>
            <changeset id="10" user="fred" uid="123" created_at="2008-11-08T19:07:39+01:00" open="true" min_lon="7.0191821" min_lat="49.2785426" max_lon="7.0197485" max_lat="49.2793101">
                <tag k="created_by" v="JOSM 1.61"/>
            </changeset>
        </osm>
        "#,
        types::Changeset {
            id: 10,
            user: "fred".into(),
            uid: 123,
            created_at: "2008-11-08T19:07:39+01:00".into(),
            closed_at: None,
            open: true,
            min_lon: Some(7.0191821),
            min_lat: Some(49.2785426),
            max_lon: Some(7.0197485),
            max_lat: Some(49.2793101),
            discussion: None,
            tags: vec![types::Tag {
                k: "created_by".into(),
                v: "JOSM 1.61".into(),
            }],
        }
    )
)]
#[actix_rt::test]
async fn test_unsubscribe(
    credentials: types::Credentials,
    changeset_id: u64,
    response_str: &str,
    expected: types::Changeset,
) {
    /*
    GIVEN an OSM client
    WHEN calling the unsubscribe() function with a changeset ID
    THEN returns the unsubscribed changeset
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(format!(
            "/api/0.6/changeset/{}/unsubscribe",
            changeset_id
        )))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.changeset().unsubscribe(changeset_id).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}
