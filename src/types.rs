use serde::ser::Serialize;

#[derive(Debug, Clone)]
pub enum Credentials {
    Basic(String, String), // Username, password
}

pub enum RequestBody<S: Serialize> {
    Xml(S),
    Form(S),
    None,
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
    pub tracepoints_per_page: u64,
    pub maximum_waynodes: u64,
    pub maximum_changeset_elements: u64,
    pub timeout: u64,
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BoundingBox {
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
    pub top: f64,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Tag {
    pub k: String,
    pub v: String,
}

impl Tag {
    pub fn new(k: &str, v: &str) -> Self {
        Tag {
            k: k.into(),
            v: v.into(),
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Node {
    pub id: u64,
    pub visible: bool,
    pub version: u64,
    pub changeset: u64,
    pub timestamp: String,
    pub user: String,
    pub uid: u64,
    pub lat: f64,
    pub lon: f64,
    #[serde(rename = "tag", default)]
    pub tags: Vec<Tag>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct NodeRef {
    #[serde(rename = "ref")]
    pub node_id: u64,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Member {
    #[serde(rename = "type")]
    pub member_type: String,
    #[serde(rename = "ref")]
    pub node_id: u64,
    pub role: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Way {
    pub id: u64,
    pub visible: bool,
    pub version: u64,
    pub changeset: u64,
    pub timestamp: String,
    pub user: String,
    pub uid: u64,
    #[serde(rename = "nd", default)]
    pub node_refs: Vec<NodeRef>,
    #[serde(rename = "tag", default)]
    pub tags: Vec<Tag>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Relation {
    pub id: u64,
    pub visible: bool,
    pub version: u64,
    pub changeset: u64,
    pub timestamp: String,
    pub user: String,
    pub uid: u64,
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

#[derive(Debug, PartialEq, Deserialize)]
pub struct Permission {
    pub name: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
#[serde(rename = "changeset")]
pub struct ChangesetCreate {
    version: String,
    generator: String,
    #[serde(rename = "tag", default)]
    tags: Vec<Tag>,
}

impl ChangesetCreate {
    pub fn new(version: &str, generator: &str, tags: Vec<Tag>) -> Self {
        ChangesetCreate {
            version: version.into(),
            generator: generator.into(),
            tags,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Comment {
    pub date: String,
    pub uid: u64,
    pub user: String,
    #[serde(rename = "$value")]
    pub text: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Discussion {
    #[serde(rename = "comment", default)]
    pub comments: Vec<Comment>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Changeset {
    pub id: u64,
    pub user: String,
    pub uid: u64,
    pub created_at: String,
    pub closed_at: Option<String>,
    pub open: bool,
    pub discussion: Option<Discussion>,
    #[serde(rename = "tag", default)]
    pub tags: Vec<Tag>,

    // The bounding box attributes will be missing for an empty changeset
    pub min_lon: Option<f64>,
    pub min_lat: Option<f64>,
    pub max_lon: Option<f64>,
    pub max_lat: Option<f64>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Modification {
    #[serde(rename = "node", default)]
    pub nodes: Vec<Node>,
    #[serde(rename = "way", default)]
    pub ways: Vec<Way>,
    #[serde(rename = "relation", default)]
    pub relations: Vec<Relation>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Creation {
    #[serde(rename = "node", default)]
    pub nodes: Vec<Node>,
    #[serde(rename = "way", default)]
    pub ways: Vec<Way>,
    #[serde(rename = "relation", default)]
    pub relations: Vec<Relation>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Deletion {
    #[serde(rename = "node", default)]
    pub nodes: Vec<Node>,
    #[serde(rename = "way", default)]
    pub ways: Vec<Way>,
    #[serde(rename = "relation", default)]
    pub relations: Vec<Relation>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename = "osmChange")]
pub struct ChangesetChanges {
    #[serde(rename = "modify", default)]
    pub modifications: Vec<Modification>,
    #[serde(rename = "create", default)]
    pub creations: Vec<Creation>,
    #[serde(rename = "delete", default)]
    pub deletions: Vec<Deletion>,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename = "node")]
pub struct DiffNode {
    pub old_id: u64,
    pub new_id: u64,
    pub new_version: u64,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename = "node")]
pub struct DiffWay {
    pub old_id: u64,
    pub new_id: u64,
    pub new_version: u64,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename = "node")]
pub struct DiffRelation {
    pub old_id: u64,
    pub new_id: u64,
    pub new_version: u64,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename = "diffResult")]
pub struct DiffResult {
    #[serde(rename = "node", default)]
    pub nodes: Vec<DiffNode>,
    #[serde(rename = "way", default)]
    pub ways: Vec<DiffWay>,
    #[serde(rename = "relation", default)]
    pub relations: Vec<DiffRelation>,
}

#[derive(Debug, Default, PartialEq)]
pub struct ChangesetQueryParams {
    pub bbox: Option<BoundingBox>,
    pub user_id: Option<u64>,
    pub display_name: Option<String>,
    pub closed_after: Option<String>,
    pub created_before: Option<String>,
    pub open: Option<bool>,
    pub closed: Option<bool>,
    pub changeset_ids: Option<Vec<u64>>,
}
