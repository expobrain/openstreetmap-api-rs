use openstreetmap_api::types;
use openstreetmap_api::Openstreetmap;
use rstest::*;
use urlencoding::encode;
use wiremock::matchers::{method, path, query_param, QueryParamExactMatcher};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::utils::credentials;
use super::utils::no_credentials;

#[fixture]
fn note() -> types::Note {
    types::Note {
        id: 16659,
        lat: 51.0000000,
        lon: 0.1000000,
        url: "https://master.apis.dev.openstreetmap.org/api/0.6/notes/16659".into(),
        comment_url: "https://master.apis.dev.openstreetmap.org/api/0.6/notes/16659/comment".into(),
        close_url: "https://master.apis.dev.openstreetmap.org/api/0.6/notes/16659/close".into(),
        created_at: "2019-06-15 08:26:04 UTC".into(),
        status: "open".into(),
        comments: vec![types::Comment {
            id: 1234,
            date: "2019-06-15 08:26:04 UTC".into(),
            user: "userName".into(),
            user_url: "https://master.apis.dev.openstreetmap.org/user/userName".into(),
            action: "opened".into(),
            text: "ThisIsANote".into(),
            html: "<p>ThisIsANote</p>".into(),
        }],
    }
}

#[fixture]
fn notes(note: types::Note) -> Vec<types::Note> {
    vec![note]
}

#[fixture]
fn note_response() -> &'static str {
    r#"
    <osm version="0.6" generator="OpenStreetMap server" copyright="OpenStreetMap and contributors" attribution="http://www.openstreetmap.org/copyright" license="http://opendatacommons.org/licenses/odbl/1-0/">
        <note lon="0.1000000" lat="51.0000000">
            <id>16659</id>
            <url>https://master.apis.dev.openstreetmap.org/api/0.6/notes/16659</url>
            <comment_url>https://master.apis.dev.openstreetmap.org/api/0.6/notes/16659/comment</comment_url>
            <close_url>https://master.apis.dev.openstreetmap.org/api/0.6/notes/16659/close</close_url>
            <date_created>2019-06-15 08:26:04 UTC</date_created>
            <status>open</status>
            <comments>
                <comment>
                    <date>2019-06-15 08:26:04 UTC</date>
                    <uid>1234</uid>
                    <user>userName</user>
                    <user_url>https://master.apis.dev.openstreetmap.org/user/userName</user_url>
                    <action>opened</action>
                    <text>ThisIsANote</text>
                    <html>&lt;p&gt;ThisIsANote&lt;/p&gt;</html>
                </comment>
            </comments>
        </note>
    </osm>
    "#
}

#[rstest(bbox, limit, closed, request_params,
    case(
        types::BoundingBox {
            left: 1.0,
            bottom: 2.0,
            right: 3.0,
            top: 4.0,
        },
        None,
        None,
        vec!(query_param("bbox", "1,2,3,4")),
    ),
    case(
        types::BoundingBox {
            left: 1.0,
            bottom: 2.0,
            right: 3.0,
            top: 4.0,
        },
        Some(100),
        None,
        vec!(query_param("bbox", "1,2,3,4"), query_param("limit", "100")),
    ),
    case(
        types::BoundingBox {
            left: 1.0,
            bottom: 2.0,
            right: 3.0,
            top: 4.0,
        },
        None,
        Some(10),
        vec!(query_param("bbox", "1,2,3,4"), query_param("closed", "10")),
    ),
    case(
        types::BoundingBox {
            left: 1.0,
            bottom: 2.0,
            right: 3.0,
            top: 4.0,
        },
        None,
        Some(-1),
        vec!(query_param("bbox", "1,2,3,4"), query_param("closed", "-1")),
    )
)]
#[actix_rt::test]
async fn test_get_by_bounding_box(
    no_credentials: types::Credentials,
    bbox: types::BoundingBox,
    limit: Option<u16>,
    closed: Option<i64>,
    request_params: Vec<QueryParamExactMatcher>,
    note_response: &str,
    notes: Vec<types::Note>,
) {
    /*
    GIVEN an OSM client
    WHEN calling the get_by_bounding_box() function
    THEN returns a list of notes
    */
    // GIVEN
    let mock_server = MockServer::start().await;
    let mut mock = Mock::given(method("GET"));

    for request_param in request_params {
        mock = mock.and(request_param);
    }

    mock.and(path("/api/0.6/notes"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(note_response, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), no_credentials);

    // WHEN
    let actual = client
        .notes()
        .get_by_bounding_box(&bbox, limit, closed)
        .await
        .unwrap();

    // THEN
    assert_eq!(actual, notes);
}

#[rstest]
#[actix_rt::test]
async fn test_get(no_credentials: types::Credentials, note_response: &str, note: types::Note) {
    /*
    GIVEN an OSM client
    WHEN calling the get() function
    THEN returns a list of notes
    */
    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/0.6/notes/{}", note.id)))
        .respond_with(ResponseTemplate::new(200).set_body_raw(note_response, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), no_credentials);

    // WHEN
    let actual = client.notes().get(note.id).await.unwrap();

    // THEN
    assert_eq!(actual, note);
}

#[fixture]
fn note_content() -> types::NoteContent {
    types::NoteContent {
        lat: 51.0000000,
        lon: 0.1000000,
        text: "ThisIsANote".into(),
    }
}

#[rstest]
#[actix_rt::test]
async fn test_create(
    credentials: types::Credentials,
    note_content: types::NoteContent,
    note_response: &str,
    note: types::Note,
) {
    /*
    GIVEN an OSM client
    WHEN calling the create() function with a NoteContent instance
    THEN returns a note
    */
    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/0.6/notes"))
        .and(query_param("lat", note_content.lat.to_string()))
        .and(query_param("lon", note_content.lon.to_string()))
        .and(query_param("text", encode(&note_content.text)))
        .respond_with(ResponseTemplate::new(200).set_body_raw(note_response, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.notes().create(note_content).await.unwrap();

    // THEN
    assert_eq!(actual, note);
}

#[rstest(
    text,
    request_param,
    case(
        "ThisIsANoteComment",
        query_param("text", encode("ThisIsANoteComment")),
    )
)]
#[rstest]
#[actix_rt::test]
async fn test_create_comment(
    credentials: types::Credentials,
    text: &str,
    request_param: QueryParamExactMatcher,
    note_response: &str,
    note: types::Note,
) {
    /*
    GIVEN an OSM client
    WHEN calling the create_comment() function with a text as comment
    THEN returns a note
    */
    // GIVEN
    let note_id = note.id;
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(format!("/api/0.6/notes/{note_id}/comment")))
        .and(request_param)
        .respond_with(ResponseTemplate::new(200).set_body_raw(note_response, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.notes().create_comment(note.id, text).await.unwrap();

    // THEN
    assert_eq!(actual, note);
}

#[rstest(
    text,
    request_param,
    case(
        "ThisIsANoteComment",
        query_param("text", encode("ThisIsANoteComment")),
    )
)]
#[rstest]
#[actix_rt::test]
async fn test_close(
    credentials: types::Credentials,
    text: &str,
    request_param: QueryParamExactMatcher,
    note_response: &str,
    note: types::Note,
) {
    /*
    GIVEN an OSM client
    WHEN calling the close() function with a text as comment
    THEN returns a note
    */
    // GIVEN
    let note_id = note.id;
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(format!("/api/0.6/notes/{note_id}/close")))
        .and(request_param)
        .respond_with(ResponseTemplate::new(200).set_body_raw(note_response, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.notes().close(note.id, text).await.unwrap();

    // THEN
    assert_eq!(actual, note);
}

#[rstest(
    text,
    request_param,
    case(
        "ThisIsANoteComment",
        query_param("text", encode("ThisIsANoteComment")),
    )
)]
#[rstest]
#[actix_rt::test]
async fn test_reopen(
    credentials: types::Credentials,
    text: &str,
    request_param: QueryParamExactMatcher,
    note_response: &str,
    note: types::Note,
) {
    /*
    GIVEN an OSM client
    WHEN calling the reopen() function with a text as comment
    THEN returns a note
    */
    // GIVEN
    let note_id = note.id;
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(format!("/api/0.6/notes/{note_id}/reopen")))
        .and(request_param)
        .respond_with(ResponseTemplate::new(200).set_body_raw(note_response, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.notes().reopen(note.id, text).await.unwrap();

    // THEN
    assert_eq!(actual, note);
}

#[rstest(search_options, request_params,
    case(
        types::NoteSearchOptions {
            q: "SearchTerm".into(), ..Default::default()
        },
        vec!(query_param("q", "SearchTerm")),
    ),
    case(
        types::NoteSearchOptions {
            q: "SearchTerm".into(), limit: Some(10), ..Default::default()
        },
        vec!(
            query_param("q", "SearchTerm"),
            query_param("limit", "10")
        ),
    ),
    case(
        types::NoteSearchOptions {
            q: "SearchTerm".into(), closed: Some(7), ..Default::default()
        },
        vec!(
            query_param("q", "SearchTerm"),
            query_param("closed", "7")
        ),
    ),
    case(
        types::NoteSearchOptions {
            q: "SearchTerm".into(), closed: Some(-1), ..Default::default()
        },
        vec!(
            query_param("q", "SearchTerm"),
            query_param("closed", "-1")
        ),
    ),
    case(
        types::NoteSearchOptions {
            q: "SearchTerm".into(),
            display_name: Some("User_name".into()),
            ..Default::default()
        },
        vec!(
            query_param("q", "SearchTerm"),
            query_param("display_name", "User_name")
        ),
    ),
    case(
        types::NoteSearchOptions {
            q: "SearchTerm".into(), user: Some(12), ..Default::default()
        },
        vec!(
            query_param("q", "SearchTerm"),
            query_param("user", "12")
        ),
    ),
    case(
        types::NoteSearchOptions {
            q: "SearchTerm".into(),
            from: Some("2020-12-09T22:51:17Z".into()),
            ..Default::default()
        },
        vec!(
            query_param("q", "SearchTerm"),
            query_param("from", "2020-12-09T22:51:17Z")
        ),
    ),
    case(
        types::NoteSearchOptions {
            q: "SearchTerm".into(),
            to: Some("2020-12-09T22:51:17Z".into()),
            ..Default::default()
        },
        vec!(
            query_param("q", "SearchTerm"),
            query_param("to", "2020-12-09T22:51:17Z")
        ),
    ),
    case(
        types::NoteSearchOptions {
            q: "SearchTerm".into(),
            sort: Some(types::NoteSearchSortOption::CreatedAt),
            ..Default::default()
        },
        vec!(
            query_param("q", "SearchTerm"),
            query_param("sort", "created_at")
        ),
    ),
    case(
        types::NoteSearchOptions {
            q: "SearchTerm".into(),
            order: Some(types::NoteSearchOrderOption::Oldest),
            ..Default::default()
        },
        vec!(
            query_param("q", "SearchTerm"),
            query_param("order", "oldest")
        ),
    ),
)]
#[actix_rt::test]
async fn test_search(
    no_credentials: types::Credentials,
    search_options: types::NoteSearchOptions,
    request_params: Vec<QueryParamExactMatcher>,
    note_response: &str,
    notes: Vec<types::Note>,
) {
    /*
    GIVEN an OSM client
    WHEN calling the search() function with options
    THEN returns a list of notes
    */
    // GIVEN
    let mock_server = MockServer::start().await;
    let mut mock = Mock::given(method("GET"));

    for request_param in request_params {
        mock = mock.and(request_param);
    }

    mock.and(path("/api/0.6/notes/search"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(note_response, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), no_credentials);

    // WHEN
    let actual = client.notes().search(&search_options).await.unwrap();

    // THEN
    assert_eq!(actual, notes);
}

#[rstest(bbox, request_param,
    case(
        types::BoundingBox {
            left: 1.0,
            bottom: 2.0,
            right: 3.0,
            top: 4.0,
        },
        query_param("bbox", "1,2,3,4"),
    )
)]
#[actix_rt::test]
async fn test_feed_by_boundng_box(
    no_credentials: types::Credentials,
    bbox: types::BoundingBox,
    request_param: QueryParamExactMatcher,
    note_response: &str,
    notes: Vec<types::Note>,
) {
    /*
    GIVEN an OSM client
    WHEN calling the feed_by_bounding_box() function with bounding box
    THEN returns a list of notes
    */
    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/0.6/notes/feed"))
        .and(request_param)
        .respond_with(ResponseTemplate::new(200).set_body_raw(note_response, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), no_credentials);

    // WHEN
    let actual = client.notes().feed_by_bounding_box(&bbox).await.unwrap();

    // THEN
    assert_eq!(actual, notes);
}
