use openstreetmap_api::types;
use openstreetmap_api::Openstreetmap;
use pretty_assertions::assert_eq;
use rstest::*;
use wiremock::matchers::{method, path, query_param, QueryParamExactMatcher};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::utils::credentials;

#[rstest(user_id, response_str, expected,
    case(
        12023,
        r#"
        <osm version="0.6" generator="OpenStreetMap server">
            <user id="12023" display_name="jbpbis" account_created="2007-08-16T01:35:56Z">
                <description />
                <contributor-terms agreed="false"/>
                <img href="http://www.gravatar.com/avatar/c8c86cd15f60ecca66ce2b10cb6b9a00.jpg"/>
                <roles />
                <changesets count="1"/>
                <traces count="0"/>
                <blocks>
                    <received count="0" active="0"/>
                </blocks>
            </user>
        </osm>
        "#,
        types::User {
            id: 12023,
            display_name: "jbpbis".into(),
            account_created: "2007-08-16T01:35:56Z".into(),
            description: Some("".into()),
            image: Some(types::Image {
                url: "http://www.gravatar.com/avatar/c8c86cd15f60ecca66ce2b10cb6b9a00.jpg".into()
            }),
            changesets: types::UserChangesets { count: 1 },
            blocks: vec![types::Block::default()],
            ..Default::default()
        }
    )
)]
#[actix_rt::test]
async fn test_get(
    credentials: types::Credentials,
    user_id: u64,
    response_str: &str,
    expected: types::User,
) {
    /*
    GIVEN an OSM client
    WHEN calling the get() function
    THEN returns user's data
    */
    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/0.6/user/{}", user_id)))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.user().get(user_id).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest(user_ids, request_qs, response_str, expected,
    case(
        vec![12023],
        query_param("users", "12023"),
        r#"
        <osm version="0.6" generator="OpenStreetMap server">
            <user id="12023" display_name="jbpbis" account_created="2007-08-16T01:35:56Z">
                <description />
                <contributor-terms agreed="false"/>
                <img href="http://www.gravatar.com/avatar/c8c86cd15f60ecca66ce2b10cb6b9a00.jpg"/>
                <roles />
                <changesets count="1"/>
                <traces count="0"/>
                <blocks>
                    <received count="0" active="0"/>
                </blocks>
            </user>
        </osm>
        "#,
        vec![types::User {
            id: 12023,
            display_name: "jbpbis".into(),
            account_created: "2007-08-16T01:35:56Z".into(),
            description: Some("".into()),
            image: Some(types::Image {
                url: "http://www.gravatar.com/avatar/c8c86cd15f60ecca66ce2b10cb6b9a00.jpg".into()
            }),
            changesets: types::UserChangesets { count: 1 },
            blocks: vec![types::Block::default()],
            ..Default::default()
        }]
    )
)]
#[actix_rt::test]
async fn test_users(
    credentials: types::Credentials,
    user_ids: Vec<u64>,
    request_qs: QueryParamExactMatcher,
    response_str: &str,
    expected: Vec<types::User>,
) {
    /*
    GIVEN an OSM client
    WHEN calling the users() function
    THEN returns mutliple user's data
    */
    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/0.6/users"))
        .and(request_qs)
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.user().users(&user_ids).await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest(response_str, expected,
    case(
        r#"
        <osm version="0.6" generator="OpenStreetMap server">
            <user display_name="Max Muster" account_created="2006-07-21T19:28:26Z" id="1234">
            <contributor-terms agreed="true" pd="true"/>
            <img href="https://www.openstreetmap.org/attachments/users/images/000/000/1234/original/someLongURLOrOther.JPG"/>
            <roles></roles>
            <changesets count="4182"/>
            <traces count="513"/>
            <blocks>
                <received count="0" active="0"/>
            </blocks>
            <home lat="49.4733718952806" lon="8.89285988577866" zoom="3"/>
            <description>The description of your profile</description>
            <languages>
                <lang>de-DE</lang>
                <lang>de</lang>
                <lang>en-US</lang>
                <lang>en</lang>
            </languages>
            <messages>
                <received count="1" unread="0"/>
                <sent count="0"/>
            </messages>
            </user>
        </osm>
        "#,
        types::User {
            id: 1234,
            display_name: "Max Muster".into(),
            account_created: "2006-07-21T19:28:26Z".into(),
            contributor_terms: types::ContributorTerms {
                agreed: true,
                public_domain: true
            },
            image: Some(types::Image {
                url: "https://www.openstreetmap.org/attachments/users/images/000/000/1234/original/someLongURLOrOther.JPG".into()
            }),
            changesets: types::UserChangesets { count: 4182 },
            traces: types::Traces{ count: 513 },
            blocks: vec![types::Block::default()],
            home: Some(types::CoordsView {
                lat: 49.4733718952806,
                lon: 8.89285988577866,
                zoom: 3
            }),
            description: Some("The description of your profile".into()),
            languages: vec![
                "de-DE".into(),
                "de".into(),
                "en-US".into(),
                "en".into(),
            ],
            messages: types::Messages {
                received: 1,
                ..Default::default()
            },
            ..Default::default()
        }
    )
)]
#[actix_rt::test]
async fn test_details(credentials: types::Credentials, response_str: &str, expected: types::User) {
    /*
    GIVEN an OSM client
    WHEN calling the details() function
    THEN returns the current user's data
    */
    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/0.6/user/details"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.user().details().await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest(response_str, expected,
    case(
        r#"
        <osm version="0.6" generator="OpenStreetMap server">
            <preferences>
                <preference k="somekey" v="somevalue" />
            </preferences>
        </osm>
        "#,
        [("somekey".to_string(), "somevalue".to_string())]
            .iter()
            .cloned()
            .collect::<types::UserPreferences>()
    )
)]
#[actix_rt::test]
async fn test_preferences(
    credentials: types::Credentials,
    response_str: &str,
    expected: types::UserPreferences,
) {
    /*
    GIVEN an OSM client
    WHEN calling the preferences() function
    THEN returns the current user's preferences
    */
    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/0.6/user/preferences"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.user().preferences().await.unwrap();

    // THEN
    assert_eq!(actual, expected);
}

#[rstest(preferences,
    case(
        [("somekey".to_string(), "somevalue".to_string())]
            .iter()
            .cloned()
            .collect::<types::UserPreferences>()
    )
)]
#[actix_rt::test]
async fn test_preferences_update(
    credentials: types::Credentials,
    preferences: types::UserPreferences,
) {
    /*
    GIVEN an OSM client
    WHEN calling the preferences_update() function
    THEN updates the user preferences
    */
    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/0.6/user/preferences"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    client
        .user()
        .preferences_update(&preferences)
        .await
        .unwrap();
}
