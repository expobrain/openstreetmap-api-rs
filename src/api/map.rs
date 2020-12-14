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
            .request_including_version::<(), Osm>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
            )
            .await?
            .into();

        Ok(map)
    }
}
