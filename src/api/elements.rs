use crate::types;
use crate::Openstreetmap;
use crate::OpenstreetmapError;

use serde::de::{self, Deserialize, DeserializeOwned, Deserializer};
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::fmt;
use std::marker::PhantomData;

pub trait OpenstreetmapNode {
    fn base_url() -> String;
    fn id(&self) -> u64;
}

impl OpenstreetmapNode for types::Node {
    #[inline]
    fn base_url() -> String {
        "node/".into()
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
    fn id(&self) -> u64 {
        self.id
    }
}

#[derive(Debug, PartialEq)]
pub struct Osm<E> {
    element: E,
}

impl<E: OpenstreetmapNode> Osm<E> {
    pub fn new(element: E) -> Self {
        Osm { element }
    }
}

impl<E: OpenstreetmapNode> Serialize for Osm<E>
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

impl<'de, E: OpenstreetmapNode> Deserialize<'de> for Osm<E>
where
    E: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // #[allow(non_camel_case_types)]
        enum Field {
            Node,
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
                    _ => Err(de::Error::invalid_value(
                        de::Unexpected::Unsigned(value),
                        &"field index 0 <= i < 1",
                    )),
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    "node" => Ok(Field::Node),
                    _ => Ok(Field::None),
                }
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    b"node" => Ok(Field::Node),
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
            marker: PhantomData<Osm<E>>,
            lifetime: PhantomData<&'de ()>,
        }

        impl<'de, E: OpenstreetmapNode> de::Visitor<'de> for Visitor<'de, E>
        where
            E: Deserialize<'de>,
        {
            type Value = Osm<E>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                fmt::Formatter::write_str(formatter, "struct Osm")
            }

            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let element = match match de::SeqAccess::next_element::<E>(&mut seq) {
                    Ok(val) => val,
                    Err(err) => {
                        return Err(err);
                    }
                } {
                    Some(value) => value,
                    None => {
                        return Err(de::Error::invalid_length(
                            0usize,
                            &"struct Osm with 1 element",
                        ));
                    }
                };

                Ok(Osm { element })
            }

            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut node: Option<E> = None;

                while let Some(key) = match de::MapAccess::next_key::<Field>(&mut map) {
                    Ok(val) => val,
                    Err(err) => {
                        return Err(err);
                    }
                } {
                    match key {
                        Field::Node => {
                            if node.is_some() {
                                return Err(<A::Error as de::Error>::duplicate_field("element"));
                            }

                            node = Some(de::MapAccess::next_value::<E>(&mut map)?);
                        }
                        _ => {
                            de::MapAccess::next_value::<de::IgnoredAny>(&mut map)?;
                        }
                    }
                }

                let element = match node {
                    Some(node) => node,
                    None => serde::private::de::missing_field("element")?,
                };

                Ok(Osm { element })
            }
        }

        const FIELDS: &[&str] = &["element"];

        Deserializer::deserialize_struct(
            deserializer,
            "Osm",
            FIELDS,
            Visitor {
                marker: PhantomData::<Osm<E>>,
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
        let body = types::RequestBody::Xml(Osm::new(element));

        let element_id = self
            .client
            .request_including_version::<Osm<E>, u64>(reqwest::Method::PUT, &url, body)
            .await?;

        Ok(element_id)
    }

    pub async fn get(&self, element_id: u64) -> Result<E, OpenstreetmapError> {
        let url = format!("{}{}", E::base_url(), element_id);
        let element = self
            .client
            .request_including_version::<u64, Osm<E>>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
            )
            .await?
            .element;

        Ok(element)
    }

    pub async fn update(&self, element: E) -> Result<u64, OpenstreetmapError> {
        let url = format!("{}{}", E::base_url(), element.id());
        let body = types::RequestBody::Xml(Osm::new(element));

        let version = self
            .client
            .request_including_version::<Osm<E>, u64>(reqwest::Method::PUT, &url, body)
            .await?;

        Ok(version)
    }

    pub async fn delete(&self, element: E) -> Result<u64, OpenstreetmapError> {
        let url = format!("{}{}", E::base_url(), element.id());
        let body = types::RequestBody::Xml(Osm::new(element));

        let version = self
            .client
            .request_including_version::<Osm<E>, u64>(reqwest::Method::DELETE, &url, body)
            .await?;

        Ok(version)
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
        let osm = Osm::new(element);

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
            Osm{
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
        )
    )]
    fn test_osm_deserialise<E>(osm_str: &str, expected: Osm<E>)
    where
        E: DeserializeOwned + OpenstreetmapNode + std::fmt::Debug + std::cmp::PartialEq,
    {
        /*
        GIVEN an string representing an Osm struct
        WHEN deserialising into with Osm struct
        THEN returns the expected element
        */

        // WHEN
        let actual: Osm<E> = from_str(&osm_str).unwrap();

        // THEN
        assert_eq!(actual, expected);
    }
}
