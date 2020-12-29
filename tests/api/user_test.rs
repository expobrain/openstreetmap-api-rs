use openstreetmap_api::types;
use openstreetmap_api::Openstreetmap;
use pretty_assertions::assert_eq;
use rstest::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::utils::credentials;

#[rstest(user_id, response_str, expected,
    case(
        12023,
        r#"
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
        "#,
        types::User {
            id: 12023,
            display_name: "jbpbis".into(),
            account_created: "2007-08-16T01:35:56Z".into(),
            description: Some("".into()),
            contributor_terms: types::ContributorTerms::default(),
            image: types::Image {
                url: "http://www.gravatar.com/avatar/c8c86cd15f60ecca66ce2b10cb6b9a00.jpg".into()
            },
            changesets: types::UserChangesets { count: 1 },
            traces: types::Traces::default(),
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
