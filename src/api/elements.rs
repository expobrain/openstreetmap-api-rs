use crate::types;
use crate::Openstreetmap;
use crate::OpenstreetmapError;
use crate::RequestOptions;

use serde::de::{self, Deserialize, DeserializeOwned, Deserializer};
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::fmt;
use std::marker::PhantomData;

pub trait OpenstreetmapNode {
    fn base_url() -> String;
    fn base_url_plural() -> String;
    fn element_name_plural() -> String;
    fn id(&self) -> u64;
}

impl OpenstreetmapNode for types::Node {
    #[inline]
    fn base_url() -> String {
        "node/".into()
    }

    #[inline]
    fn base_url_plural() -> String {
        "nodes/".into()
    }

    #[inline]
    fn element_name_plural() -> String {
        "nodes".into()
    }

    #[inline]
    fn id(&self) -> u64 {
        self.id
    }
}

impl OpenstreetmapNode for types::Way {
    #[inline]
    fn base_url() -> String {
        "way/".into()
    }

    #[inline]
    fn base_url_plural() -> String {
        "ways/".into()
    }

    #[inline]
    fn element_name_plural() -> String {
        "ways".into()
    }

    #[inline]
    fn id(&self) -> u64 {
        self.id
    }
}

impl OpenstreetmapNode for types::Relation {
    #[inline]
    fn base_url() -> String {
        "relation/".into()
    }

    #[inline]
    fn base_url_plural() -> String {
        "relations/".into()
    }

    #[inline]
    fn element_name_plural() -> String {
        "relations".into()
    }

    #[inline]
    fn id(&self) -> u64 {
        self.id
    }
}

#[derive(Debug, PartialEq)]
pub struct OsmSingle<E> {
    element: E,
}

impl<E: OpenstreetmapNode> OsmSingle<E> {
    pub fn new(element: E) -> Self {
        Self { element }
    }
}

impl<E: OpenstreetmapNode> Serialize for OsmSingle<E>
where
    E: OpenstreetmapNode + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("osm", &self.element)?;
        map.end()
    }
}

impl<'de, E: OpenstreetmapNode> Deserialize<'de> for OsmSingle<E>
where
    E: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Node,
            Way,
            Relation,
            None,
        }

        struct FieldVisitor;

        impl<'de> de::Visitor<'de> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                fmt::Formatter::write_str(formatter, "field identifier")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    0u64 => Ok(Field::Node),
                    1u64 => Ok(Field::Way),
                    2u64 => Ok(Field::Relation),
                    _ => Err(de::Error::invalid_value(
                        de::Unexpected::Unsigned(value),
                        &"field index 0 <= i < 3",
                    )),
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    "node" => Ok(Field::Node),
                    "way" => Ok(Field::Way),
                    "relation" => Ok(Field::Relation),
                    _ => Ok(Field::None),
                }
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    b"node" => Ok(Field::Node),
                    b"way" => Ok(Field::Way),
                    b"relation" => Ok(Field::Relation),
                    _ => Ok(Field::None),
                }
            }
        }

        impl<'de> Deserialize<'de> for Field {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserializer::deserialize_identifier(deserializer, FieldVisitor)
            }
        }

        struct Visitor<'de, E: OpenstreetmapNode>
        where
            E: Deserialize<'de>,
        {
            marker: PhantomData<OsmSingle<E>>,
            lifetime: PhantomData<&'de ()>,
        }

        impl<'de, E: OpenstreetmapNode> de::Visitor<'de> for Visitor<'de, E>
        where
            E: Deserialize<'de>,
        {
            type Value = OsmSingle<E>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                fmt::Formatter::write_str(formatter, "struct Osm")
            }

            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let element = match de::SeqAccess::next_element::<E>(&mut seq)? {
                    Some(value) => value,
                    None => {
                        return Err(de::Error::invalid_length(
                            0usize,
                            &"struct Osm with 1 element",
                        ));
                    }
                };

                Ok(OsmSingle { element })
            }

            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut value: Option<E> = None;

                while let Some(key) = de::MapAccess::next_key::<Field>(&mut map)? {
                    match key {
                        Field::Node | Field::Way | Field::Relation => {
                            if value.is_some() {
                                return Err(<A::Error as de::Error>::duplicate_field("element"));
                            }

                            value = Some(de::MapAccess::next_value::<E>(&mut map)?);
                        }
                        _ => {
                            de::MapAccess::next_value::<de::IgnoredAny>(&mut map)?;
                        }
                    }
                }

                let element = value.ok_or_else(|| serde::de::Error::missing_field("element"))?;

                Ok(OsmSingle { element })
            }
        }

        const FIELDS: &[&str] = &["element"];

        Deserializer::deserialize_struct(
            deserializer,
            "Osm",
            FIELDS,
            Visitor {
                marker: PhantomData::<OsmSingle<E>>,
                lifetime: PhantomData,
            },
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct OsmList<E> {
    elements: Vec<E>,
}

impl<'de, E: OpenstreetmapNode> Deserialize<'de> for OsmList<E>
where
    E: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Node,
            Way,
            Relation,
            None,
        }

        struct FieldVisitor;

        impl<'de> de::Visitor<'de> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                fmt::Formatter::write_str(formatter, "field identifier")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    0u64 => Ok(Field::Node),
                    1u64 => Ok(Field::Way),
                    2u64 => Ok(Field::Relation),
                    _ => Err(de::Error::invalid_value(
                        de::Unexpected::Unsigned(value),
                        &"field index 0 <= i < 3",
                    )),
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    "node" => Ok(Field::Node),
                    "way" => Ok(Field::Way),
                    "relation" => Ok(Field::Relation),
                    _ => Ok(Field::None),
                }
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    b"node" => Ok(Field::Node),
                    b"way" => Ok(Field::Way),
                    b"relation" => Ok(Field::Relation),
                    _ => Ok(Field::None),
                }
            }
        }

        impl<'de> Deserialize<'de> for Field {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserializer::deserialize_identifier(deserializer, FieldVisitor)
            }
        }

        struct Visitor<'de, E: OpenstreetmapNode>
        where
            E: Deserialize<'de>,
        {
            marker: PhantomData<OsmList<E>>,
            lifetime: PhantomData<&'de ()>,
        }

        impl<'de, E: OpenstreetmapNode> de::Visitor<'de> for Visitor<'de, E>
        where
            E: Deserialize<'de>,
        {
            type Value = OsmList<E>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                fmt::Formatter::write_str(formatter, "struct OsmList")
            }

            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let elements = match de::SeqAccess::next_element::<Vec<E>>(&mut seq)? {
                    Some(value) => value,
                    None => {
                        return Err(de::Error::invalid_length(
                            0usize,
                            &"struct OsmList with 1 element",
                        ));
                    }
                };

                Ok(OsmList { elements })
            }

            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut value: Option<Vec<E>> = None;

                while let Some(key) = de::MapAccess::next_key::<Field>(&mut map)? {
                    match key {
                        Field::Node | Field::Way | Field::Relation => {
                            if value.is_some() {
                                return Err(<A::Error as de::Error>::duplicate_field("element"));
                            }

                            value = Some(serde::de::MapAccess::next_value::<Vec<E>>(&mut map)?);
                        }
                        _ => {
                            de::MapAccess::next_value::<de::IgnoredAny>(&mut map)?;
                        }
                    }
                }

                let elements = value.ok_or_else(|| serde::de::Error::missing_field("element"))?;

                Ok(OsmList { elements })
            }
        }

        const FIELDS: &[&str] = &["elements"];

        Deserializer::deserialize_struct(
            deserializer,
            "OsmList",
            FIELDS,
            Visitor {
                marker: PhantomData::<OsmList<E>>,
                lifetime: PhantomData,
            },
        )
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
        let url = format!("{}create", E::base_url());
        let body = types::RequestBody::Xml(OsmSingle::new(element));

        let element_id = self
            .client
            .request::<OsmSingle<E>, u64>(
                reqwest::Method::PUT,
                &url,
                body,
                RequestOptions::new().with_version().with_auth(),
            )
            .await?;

        Ok(element_id)
    }

    pub async fn get(&self, element_id: u64) -> Result<E, OpenstreetmapError> {
        let url = format!("{}{}", E::base_url(), element_id);
        let element = self
            .client
            .request::<u64, OsmSingle<E>>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?
            .element;

        Ok(element)
    }

    pub async fn update(&self, element: E) -> Result<u64, OpenstreetmapError> {
        let url = format!("{}{}", E::base_url(), element.id());
        let body = types::RequestBody::Xml(OsmSingle::new(element));

        let version = self
            .client
            .request::<OsmSingle<E>, u64>(
                reqwest::Method::PUT,
                &url,
                body,
                RequestOptions::new().with_version().with_auth(),
            )
            .await?;

        Ok(version)
    }

    pub async fn delete(&self, element: E) -> Result<u64, OpenstreetmapError> {
        let url = format!("{}{}", E::base_url(), element.id());
        let body = types::RequestBody::Xml(OsmSingle::new(element));

        let version = self
            .client
            .request::<OsmSingle<E>, u64>(
                reqwest::Method::DELETE,
                &url,
                body,
                RequestOptions::new().with_version().with_auth(),
            )
            .await?;

        Ok(version)
    }

    pub async fn history(&self, element_id: u64) -> Result<Vec<E>, OpenstreetmapError> {
        let url = format!("{}{}/history", E::base_url(), element_id);
        let elements = self
            .client
            .request::<u64, OsmList<E>>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?
            .elements;

        Ok(elements)
    }

    pub async fn version(&self, element_id: u64, version_id: u64) -> Result<E, OpenstreetmapError> {
        let url = format!("{}{}/{}", E::base_url(), element_id, version_id);
        let element = self
            .client
            .request::<u64, OsmSingle<E>>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?
            .element;

        Ok(element)
    }

    pub async fn multi_get(
        &self,
        element_id_params: Vec<types::ElementIdParam>,
    ) -> Result<Vec<E>, OpenstreetmapError> {
        let element_id_params_raw = element_id_params
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(",");
        let qs = serde_urlencoded::to_string(&[(E::element_name_plural(), element_id_params_raw)])?;
        let url = format!("{}?{}", E::base_url_plural(), qs);

        let elements = self
            .client
            .request::<u64, OsmList<E>>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?
            .elements;

        Ok(elements)
    }

    pub async fn relations(
        &self,
        element_id: u64,
    ) -> Result<Vec<types::Relation>, OpenstreetmapError> {
        let url = format!("{}{}/relations", E::base_url(), element_id);
        let elements = self
            .client
            .request::<u64, OsmList<types::Relation>>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?
            .elements;

        Ok(elements)
    }
}

impl Elements<types::Node> {
    pub async fn ways(&self, node_id: u64) -> Result<Vec<types::Way>, OpenstreetmapError> {
        let url = format!("node/{}/ways", node_id);
        let elements = self
            .client
            .request::<u64, OsmList<types::Way>>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?
            .elements;

        Ok(elements)
    }
}

impl Elements<types::Way> {
    pub async fn full(&self, way_id: u64) -> Result<types::WayFull, OpenstreetmapError> {
        let url = format!("way/{}/full", way_id);
        let full = self
            .client
            .request::<u64, types::WayFull>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?;

        Ok(full)
    }
}

impl Elements<types::Relation> {
    pub async fn full(&self, relation_id: u64) -> Result<types::RelationFull, OpenstreetmapError> {
        let url = format!("relation/{}/full", relation_id);
        let full = self
            .client
            .request::<u64, types::RelationFull>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?;

        Ok(full)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use quick_xml::de::from_str;
    use quick_xml::se::to_string;
    use rstest::*;
    use serde::ser::Serialize;

    #[rstest(
        element_base_url,
        expected,
        case(types::Node::base_url(), "node/"),
        case(types::Way::base_url(), "way/"),
        case(types::Relation::base_url(), "relation/")
    )]
    fn test_base_url(element_base_url: String, expected: &str) {
        /*
        GIVEN an struct implementing OpenstreetmapNode trait
        WHEN calling base_url() method
        THEN returns the expected string
        */
        assert_eq!(element_base_url, expected);
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
            vec![
                r#"<osm>"#,
                r#"<node id="1234" visible="true" version="2" changeset="42" timestamp="2009-12-09T08:19:00Z" user="user" uid="1" lat="12.1234567" lon="-8.7654321">"#,
                r#"<tag k="amenity" v="school"/>"#,
                r#"</node>"#,
                r#"</osm>"#,
            ].join("")
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
            vec![
                r#"<osm>"#,
                r#"<way id="49780" visible="true" version="1" changeset="2308" timestamp="2009-12-09T08:51:50Z" user="guggis" uid="1">"#,
                r#"<nd ref="1150401"/>"#,
                r#"<nd ref="1150400"/>"#,
                r#"<tag k="random-key.1" v="random-value.1"/>"#,
                r#"</way>"#,
                r#"</osm>"#,
            ].join("")
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
                }],
            },
            vec![
                r#"<osm>"#,
                r#"<relation id="4507" visible="true" version="1" changeset="3198" timestamp="2010-02-25T19:52:18Z" user="rus" uid="96">"#,
                r#"<tag k="type" v="multipolygon"/>"#,
                r#"<member type="way" ref="80976" role="outer"/>"#,
                r#"<member type="way" ref="80977" role="outer"/>"#,
                r#"</relation>"#,
                r#"</osm>"#,
            ].join("")
        )
    )]
    fn test_osm_serialise<E>(element: E, expected: String)
    where
        E: OpenstreetmapNode + Serialize,
    {
        /*
        GIVEN an struct implementing OpenstreetmapNode trait
        WHEN serialising the element into with Osm struct
        THEN returns the expected string
        */
        // GIVEN
        let osm = OsmSingle::new(element);

        // WHEN
        let actual = to_string(&osm).unwrap();

        // THEN
        assert_eq!(actual, expected);
    }

    #[rstest(osm_str, expected,
        case(
            r#"
            <osm>
                <node id="1234" visible="true" version="2" changeset="42" timestamp="2009-12-09T08:19:00Z" user="user" uid="1" lat="12.1234567" lon="-8.7654321">
                    <tag k="amenity" v="school"/>
                </node>
            </osm>
            "#,
            OsmSingle {
                element: types::Node {
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
                }
            }
        ),
        case(
            r#"
            <osm>
                <way id="49780" visible="true" version="1" changeset="2308" timestamp="2009-12-09T08:51:50Z" user="guggis" uid="1">
                    <nd ref="1150401"/>
                    <nd ref="1150400"/>
                    <tag k="random-key.1" v="random-value.1"/>
                </way>
            </osm>
            "#,
            OsmSingle {
                element: types::Way {
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
                }
            }
        ),
        case(
            r#"
            <osm>
                <relation id="4507" visible="true" version="1" changeset="3198" timestamp="2010-02-25T19:52:18Z" user="rus" uid="96">
                    <member type="way" ref="80976" role="outer"/>
                    <member type="way" ref="80977" role="outer"/>
                    <tag k="type" v="multipolygon"/>
                </relation>
            </osm>
            "#,
            OsmSingle {
                element: types::Relation {
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
                    }],
                }
            }
        )
    )]
    fn test_osm_deserialise<E>(osm_str: &str, expected: OsmSingle<E>)
    where
        E: DeserializeOwned + OpenstreetmapNode + std::fmt::Debug + std::cmp::PartialEq,
    {
        /*
        GIVEN an string representing an Osm struct
        WHEN deserialising into with Osm struct
        THEN returns the expected element
        */

        // WHEN
        let actual: OsmSingle<E> = from_str(osm_str).unwrap();

        // THEN
        assert_eq!(actual, expected);
    }
}
