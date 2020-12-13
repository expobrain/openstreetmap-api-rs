use crate::types;
use crate::Openstreetmap;
use crate::OpenstreetmapError;

use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::marker::PhantomData;

pub trait OpenstreetmapNode {
    fn base_url(&self) -> String;
}

impl OpenstreetmapNode for types::Node {
    #[inline]
    fn base_url(&self) -> String {
        "node/".into()
    }
}

impl OpenstreetmapNode for types::Way {
    #[inline]
    fn base_url(&self) -> String {
        "way/".into()
    }
}

impl OpenstreetmapNode for types::Relation {
    #[inline]
    fn base_url(&self) -> String {
        "relation/".into()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "osm")]
struct Osm<E> {
    element: E,
}

impl<E: Serialize> Osm<E> {
    pub fn new(element: E) -> Self {
        Osm { element }
    }
}

pub struct Elements<E: OpenstreetmapNode + Serialize + DeserializeOwned> {
    client: Openstreetmap,

    element_type: PhantomData<E>,
}

impl<E: OpenstreetmapNode + Serialize + DeserializeOwned> Elements<E> {
    pub fn new(client: &Openstreetmap) -> Self {
        Elements {
            client: client.clone(),
            element_type: PhantomData,
        }
    }

    pub async fn create(&self, element: E) -> Result<u64, OpenstreetmapError> {
        let url = format!("{}create", element.base_url());
        let body = types::RequestBody::Xml(Osm::new(element));

        let element_id = self
            .client
            .request_including_version::<Osm<E>, u64>(reqwest::Method::PUT, &url, body)
            .await?;

        Ok(element_id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[fixture]
    fn credentials() -> types::Credentials {
        types::Credentials::Basic("user".into(), "password".into())
    }

    #[rstest(element, expected,
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
            "node/"
        ),
        case(
            types::Way {
                id: 49780,
                visible: true,
                version: 1,
                changeset: 2308,
                timestamp: "2009-12-09T08:51:50Z".into(),
                user: "guggis".into(),
                uid: 1,
                node_refs: vec![
                    types::NodeRef { node_id: 1150401 },
                    types::NodeRef { node_id: 1150400 },
                ],
                tags: vec![types::Tag {
                    k: "random-key.1".into(),
                    v: "random-value.1".into(),
                }],
            },
            "way/"
        ),
        case(
            types::Relation {
                id: 4507,
                visible: true,
                version: 1,
                changeset: 3198,
                timestamp: "2010-02-25T19:52:18Z".into(),
                user: "rus".into(),
                uid: 96,
                members: vec![
                    types::Member {
                        member_type: "way".into(),
                        node_id: 80976,
                        role: "outer".into(),
                    },
                    types::Member {
                        member_type: "way".into(),
                        node_id: 80977,
                        role: "outer".into(),
                    },
                ],
                tags: vec![types::Tag {
                    k: "type".into(),
                    v: "multipolygon".into(),
                }]
            },
            "relation/"
        ),
    )]
    fn test_base_url<E>(element: E, expected: &str)
    where
        E: OpenstreetmapNode,
    {
        /*
        GIVEN an struct implementing OpenstreetmapNode trait
        WHEN calling base_url() method
        THEN returns the expected string
        */
        // WHEN
        let actual = element.base_url();

        // THEN
        assert_eq!(actual, expected);
    }

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
}
