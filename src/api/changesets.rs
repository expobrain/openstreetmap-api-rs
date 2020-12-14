use crate::types;
use crate::Openstreetmap;
use crate::OpenstreetmapError;

use serde::ser::Serializer;
use std::fmt::Display;

fn vec_to_string<T, S>(vector: &Option<Vec<T>>, serialiser: S) -> Result<S::Ok, S::Error>
where
    T: Display,
    S: Serializer,
{
    let serialisable_value = match vector {
        Some(value) => Some(
            value
                .iter()
                .map(|v| format!("{}", v))
                .collect::<Vec<String>>()
                .join(","),
        ),
        _ => None,
    };

    match serialisable_value {
        None => serialiser.serialize_none(),
        _ => serialiser.serialize_some(&serialisable_value),
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Query {
    pub bbox: Option<types::BoundingBox>,
    pub user_id: Option<u64>,
    pub display_name: Option<String>,
    pub closed_after: Option<String>,
    pub created_before: Option<String>,
    pub open: Option<bool>,
    pub closed: Option<bool>,
    pub changeset_ids: Option<Vec<u64>>,
}

#[derive(Debug, Serialize, PartialEq)]
struct RawQuery {
    #[serde(serialize_with = "vec_to_string")]
    pub bbox: Option<Vec<f64>>,
    pub user: Option<u64>,
    pub display_name: Option<String>,
    #[serde(serialize_with = "vec_to_string")]
    pub time: Option<Vec<String>>,
    pub open: Option<bool>,
    pub closed: Option<bool>,
    #[serde(serialize_with = "vec_to_string")]
    pub changesets: Option<Vec<u64>>,
}

impl From<Query> for RawQuery {
    fn from(query: Query) -> Self {
        RawQuery {
            bbox: query
                .bbox
                .map(|bbox| vec![bbox.left, bbox.bottom, bbox.right, bbox.top]),
            user: query.user_id,
            display_name: query.display_name.clone(),
            time: match (query.closed_after, query.created_before) {
                (Some(t1), None) => Some(vec![t1]),
                (Some(t1), Some(t2)) => Some(vec![t1, t2]),
                _ => None,
            },
            open: query.open,
            closed: query.closed,
            changesets: query.changeset_ids,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename = "osm")]
struct Osm {
    #[serde(rename = "changeset", default)]
    pub changesets: Vec<types::Changeset>,
}

pub struct Changesets {
    client: Openstreetmap,
}

impl Changesets {
    pub fn new(client: &Openstreetmap) -> Self {
        Changesets {
            client: client.clone(),
        }
    }

    pub async fn get(&self, query: Query) -> Result<Vec<types::Changeset>, OpenstreetmapError> {
        let raw_query: RawQuery = query.into();
        let qs = serde_urlencoded::to_string(raw_query)?;

        let mut url = "changesets".to_string();

        if !qs.is_empty() {
            url.push('?');
            url.push_str(&qs);
        }

        let changesets = self
            .client
            .request_including_version::<(), Osm>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
            )
            .await?
            .changesets;

        Ok(changesets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[fixture]
    fn credentials() -> types::Credentials {
        types::Credentials::Basic("user".into(), "password".into())
    }

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

    #[test]
    fn test_query_raw_from() {
        /*
        GIVEN a Query with all attributes set
        WHEN building
        THEN a Query is returned
            AND all attribues are set
        */
        // GIVEN
        let query = Query {
            bbox: Some(types::BoundingBox {
                left: 1.0,
                bottom: 2.0,
                right: 3.0,
                top: 4.0,
            }),
            user_id: Some(123),
            display_name: Some("user".into()),
            closed_after: Some("2020-12-09T22:51:17Z".into()),
            created_before: Some("2020-11-09T22:51:17Z".into()),
            open: Some(true),
            closed: Some(false),
            changeset_ids: Some(vec![1, 2, 3]),
            ..Default::default()
        };

        // WHEN
        let raw_query: RawQuery = query.into();

        // THEN
        let expected = RawQuery {
            bbox: Some(vec![1.0, 2.0, 3.0, 4.0]),
            user: Some(123),
            display_name: Some("user".into()),
            time: Some(vec![
                "2020-12-09T22:51:17Z".into(),
                "2020-11-09T22:51:17Z".into(),
            ]),
            open: Some(true),
            closed: Some(false),
            changesets: Some(vec![1, 2, 3]),
        };

        assert_eq!(raw_query, expected);
    }

    #[test]
    fn test_raw_query_url_encode() {
        /*
        GIVEN a RawQuery
        WHEN serialising it into a query param
        THEN matches the expectations
        */
        // GIVEN
        let raw_query = RawQuery {
            bbox: Some(vec![1.0, 2.0, 3.0, 4.0]),
            user: Some(123),
            display_name: Some("user".into()),
            time: Some(vec![
                "2020-12-09T22:51:17Z".into(),
                "2020-11-09T22:51:17Z".into(),
            ]),
            open: Some(true),
            closed: Some(false),
            changesets: Some(vec![1, 2, 3]),
        };

        // WHEN
        let actual = serde_urlencoded::to_string(&raw_query).unwrap();

        // THEN
        let expected = vec![
            "bbox=1%2C2%2C3%2C4",
            "user=123",
            "display_name=user",
            "time=2020-12-09T22%3A51%3A17Z%2C2020-11-09T22%3A51%3A17Z",
            "open=true",
            "closed=false",
            "changesets=1%2C2%2C3",
        ]
        .join("&");

        assert_eq!(actual, expected);
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
        credentials: types::Credentials,
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

        let client = Openstreetmap::new(mock_server.uri(), credentials);

        // WHEN
        let query = Query::default();
        let actual = client.changesets(query).await.unwrap();

        // THEN
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[actix_rt::test]
    async fn test_get_with_query(credentials: types::Credentials, response_str: String) {
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

        let client = Openstreetmap::new(mock_server.uri(), credentials);

        // WHEN
        let query = Query {
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
}
