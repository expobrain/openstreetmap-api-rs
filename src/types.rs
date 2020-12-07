#[derive(Debug, Clone)]
pub enum Credentials {
    Basic(String, String), // Username, password
}

#[derive(Debug, PartialEq)]
pub struct VersionRange {
    pub minimum: String,
    pub maximum: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Status {
    pub database: String,
    pub api: String,
    pub gpx: String,
}

#[derive(Debug, PartialEq)]
pub struct Capabilities {
    pub versions: VersionRange,
    pub maximum_area: f64,
    pub maximum_note_area: f64,
    pub tracepoints_per_page: u32,
    pub maximum_waynodes: u32,
    pub maximum_changeset_elements: u32,
    pub timeout: u32,
    pub status: Status,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Blacklist {
    pub regex: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Imagery {
    pub blacklist: Vec<Blacklist>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Policy {
    pub imagery: Imagery,
}

#[derive(Debug, PartialEq)]
pub struct CapabilitiesAndPolicy {
    pub capabilities: Capabilities,
    pub policy: Policy,
}

#[derive(Debug, PartialEq)]
pub struct BoundingBox {
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
    pub top: f64,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Tag {
    pub k: String,
    pub v: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Node {
    pub id: u32,
    pub visible: bool,
    pub version: u32,
    pub changeset: u32,
    pub timestamp: String,
    pub user: String,
    pub uid: u32,
    pub lat: f64,
    pub lon: f64,
    #[serde(rename = "tag", default)]
    pub tags: Vec<Tag>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct NodeRef {
    #[serde(rename = "ref")]
    pub node_id: u32,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Member {
    #[serde(rename = "type")]
    pub member_type: String,
    #[serde(rename = "ref")]
    pub node_id: u32,
    pub role: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Way {
    pub id: u32,
    pub visible: bool,
    pub version: u32,
    pub changeset: u32,
    pub timestamp: String,
    pub user: String,
    pub uid: u32,
    #[serde(rename = "nd", default)]
    pub node_refs: Vec<NodeRef>,
    #[serde(rename = "tag", default)]
    pub tags: Vec<Tag>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Relation {
    pub id: u32,
    pub visible: bool,
    pub version: u32,
    pub changeset: u32,
    pub timestamp: String,
    pub user: String,
    pub uid: u32,
    #[serde(rename = "tag", default)]
    pub tags: Vec<Tag>,
    #[serde(rename = "member", default)]
    pub members: Vec<Member>,
}

#[derive(Debug, PartialEq)]
pub struct Map {
    pub bounds: BoundingBox,
    pub nodes: Vec<Node>,
    pub ways: Vec<Way>,
    pub relations: Vec<Relation>,
}
