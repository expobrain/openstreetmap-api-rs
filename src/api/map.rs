use crate::types;
use crate::Openstreetmap;
use crate::OpenstreetmapError;

#[derive(Debug, Deserialize)]
struct Bounds {
    pub minlat: f64,
    pub minlon: f64,
    pub maxlat: f64,
    pub maxlon: f64,
}

impl Into<types::BoundingBox> for Bounds {
    fn into(self) -> types::BoundingBox {
        types::BoundingBox {
            left: self.minlon,
            bottom: self.minlat,
            right: self.maxlon,
            top: self.maxlat,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Osm {
    pub bounds: Bounds,
    #[serde(rename = "node", default)]
    pub nodes: Vec<types::Node>,
    #[serde(rename = "way", default)]
    pub ways: Vec<types::Way>,
    #[serde(rename = "relation", default)]
    pub relations: Vec<types::Relation>,
}

impl Into<types::Map> for Osm {
    fn into(self) -> types::Map {
        types::Map {
            bounds: self.bounds.into(),
            nodes: self.nodes,
            ways: self.ways,
            relations: self.relations,
        }
    }
}

pub struct Map {
    client: Openstreetmap,
}

impl Map {
    pub fn new(client: &Openstreetmap) -> Self {
        Map {
            client: client.clone(),
        }
    }

    pub async fn get(&self, bbox: &types::BoundingBox) -> Result<types::Map, OpenstreetmapError> {
        let url = format!(
            "map?bbox={},{},{},{}",
            bbox.left, bbox.bottom, bbox.right, bbox.top
        );
        let map = self
            .client
            .request::<Osm>(
                reqwest::Method::GET,
                Some(&self.client.api_version),
                &url,
                None,
            )
            .await?
            .into();

        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::Credentials;
    use crate::Openstreetmap;

    use super::*;
    use lazy_static::lazy_static;
    use pretty_assertions::assert_eq;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const MAP_STR: &str = r##"
        <osm version="0.6" generator="CGImap 0.8.3 (27359 errol.openstreetmap.org)" copyright="OpenStreetMap and contributors" attribution="http://www.openstreetmap.org/copyright" license="http://opendatacommons.org/licenses/odbl/1-0/">
            <bounds minlat="2.0000000" minlon="1.0000000" maxlat="4.0000000" maxlon="3.0000000"/>
            <node id="1150316" visible="true" version="1" changeset="2297" timestamp="2009-12-09T08:19:00Z" user="guggis" uid="1" lat="1.0000000" lon="1.0000000"/>
            <node id="2935283" visible="true" version="1" changeset="3180" timestamp="2010-02-19T16:29:45Z" user="EtienneChove" uid="34" lat="1.0000000" lon="1.0000000">
                <tag k="#" v="#"/>
                <tag k="place" v="locality"/>
            </node>
            <way id="49780" visible="true" version="1" changeset="2308" timestamp="2009-12-09T08:51:50Z" user="guggis" uid="1">
                <nd ref="1150401"/>
                <nd ref="1150400"/>
                <tag k="random-key.1" v="random-value.1"/>
            </way>
            <relation id="4507" visible="true" version="1" changeset="3198" timestamp="2010-02-25T19:52:18Z" user="rus" uid="96">
                <member type="way" ref="80976" role="outer"/>
                <member type="way" ref="80977" role="outer"/>
                <tag k="type" v="multipolygon"/>
            </relation>
        </osm>
    "##;

    lazy_static! {
        static ref CREDENTIALS: Credentials = Credentials::Basic("user".into(), "password".into());
    }

    #[actix_rt::test]
    async fn test_get() {
        /*
        GIVEN an OSM client
        WHEN calling the map() function
        THEN returns the list of nodes, ways and relations inside the bbox
        */

        // GIVEN
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/0.6/map"))
            .and(query_param("bbox", "1,2,3,4"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(MAP_STR, "application/xml"))
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), CREDENTIALS.clone());
        let bbox = types::BoundingBox {
            left: 1.0,
            bottom: 2.0,
            right: 3.0,
            top: 4.0,
        };

        // WHEN
        let actual = client.map(&bbox).await.unwrap();

        // THEN
        let expected = types::Map {
            bounds: types::BoundingBox {
                left: 1.0,
                bottom: 2.0,
                right: 3.0,
                top: 4.0,
            },
            nodes: vec![
                types::Node {
                    id: 1150316,
                    visible: true,
                    version: 1,
                    changeset: 2297,
                    timestamp: "2009-12-09T08:19:00Z".into(),
                    user: "guggis".into(),
                    uid: 1,
                    lat: 1.0,
                    lon: 1.0,
                    tags: vec![],
                },
                types::Node {
                    id: 2935283,
                    visible: true,
                    version: 1,
                    changeset: 3180,
                    timestamp: "2010-02-19T16:29:45Z".into(),
                    user: "EtienneChove".into(),
                    uid: 34,
                    lat: 1.0,
                    lon: 1.0,
                    tags: vec![
                        types::Tag {
                            k: "#".into(),
                            v: "#".into(),
                        },
                        types::Tag {
                            k: "place".into(),
                            v: "locality".into(),
                        },
                    ],
                },
            ],
            ways: vec![types::Way {
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
            }],
            relations: vec![types::Relation {
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
            }],
        };

        assert_eq!(actual, expected);
    }
}
