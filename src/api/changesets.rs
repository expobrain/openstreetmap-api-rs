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
    let serialisable_value = vector.as_ref().map(|value| {
        value
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<String>>()
            .join(",")
    });

    match serialisable_value {
        None => serialiser.serialize_none(),
        _ => serialiser.serialize_some(&serialisable_value),
    }
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

impl From<types::ChangesetQueryParams> for RawQuery {
    fn from(query: types::ChangesetQueryParams) -> Self {
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

    pub async fn get(
        &self,
        query: types::ChangesetQueryParams,
    ) -> Result<Vec<types::Changeset>, OpenstreetmapError> {
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

    #[test]
    fn test_query_raw_from() {
        /*
        GIVEN a Query with all attributes set
        WHEN building
        THEN a Query is returned
            AND all attribues are set
        */
        // GIVEN
        let query = types::ChangesetQueryParams {
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
}
