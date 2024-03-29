use serde::ser::Serialize;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Credentials {
    Basic(String, String), // Username, password
    None,
}

pub enum RequestBody<S: Serialize> {
    Xml(S),
    Form(S),
    RawForm(Vec<u8>),
    None,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VersionRange {
    pub minimum: String,
    pub maximum: String,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Blacklist {
    pub regex: String,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Imagery {
    pub blacklist: Vec<Blacklist>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
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
#[serde(rename = "node")]
pub struct Node {
    pub id: u64,
    pub visible: bool,
    pub version: u64,
    pub changeset: u64,
    pub timestamp: String,
    pub user: Option<String>,
    pub uid: Option<u64>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    #[serde(rename = "tag", default)]
    pub tags: Vec<Tag>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeRef {
    #[serde(rename = "ref")]
    pub node_id: u64,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Member {
    #[serde(rename = "type")]
    pub member_type: String,
    #[serde(rename = "ref")]
    pub node_id: u64,
    pub role: String,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename = "way")]
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

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename = "relation")]
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

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Permission {
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
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

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct DiscussionComment {
    pub date: String,
    pub uid: u64,
    pub user: String,
    pub text: String,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Discussion {
    #[serde(rename = "comment", default)]
    pub comments: Vec<DiscussionComment>,
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

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename = "node")]
pub struct DiffNode {
    pub old_id: u64,
    pub new_id: u64,
    pub new_version: u64,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename = "node")]
pub struct DiffWay {
    pub old_id: u64,
    pub new_id: u64,
    pub new_version: u64,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename = "node")]
pub struct DiffRelation {
    pub old_id: u64,
    pub new_id: u64,
    pub new_version: u64,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
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

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ElementIdParam {
    pub id: u64,
    pub version: Option<u64>,
}

impl ElementIdParam {
    pub fn new(id: u64, version: Option<u64>) -> Self {
        Self { id, version }
    }
}

impl fmt::Display for ElementIdParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.id,
            self.version.map_or("".to_string(), |v| format!("v{v}"))
        )
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct WayFull {
    pub way: Way,
    #[serde(rename = "node", default)]
    pub nodes: Vec<Node>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct RelationFull {
    pub relation: Relation,
    #[serde(rename = "way", default)]
    pub ways: Vec<Way>,
    #[serde(rename = "node", default)]
    pub nodes: Vec<Node>,
}

#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
pub struct ContributorTerms {
    pub agreed: bool,
    #[serde(rename = "pd", default)]
    pub public_domain: bool,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Image {
    #[serde(rename = "href")]
    pub url: String,
}

#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
pub struct UserChangesets {
    pub count: u64,
}

#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Traces {
    pub count: u64,
}

#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Block {
    pub count: u64,
    pub active: u64,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct CoordsView {
    pub lat: f64,
    pub lon: f64,
    pub zoom: u8,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Messages {
    pub received: u64,
    pub unread: u64,
    pub sent: u64,
}

#[derive(Debug, Default, PartialEq)]
pub struct User {
    pub id: u64,
    pub display_name: String,
    pub account_created: String,
    pub description: Option<String>,
    pub contributor_terms: ContributorTerms,
    pub image: Option<Image>,
    pub changesets: UserChangesets,
    pub traces: Traces,
    pub blocks: Vec<Block>,
    pub home: Option<CoordsView>,
    pub languages: Vec<String>,
    pub messages: Messages,
}

pub type UserPreferences = HashMap<String, String>;

#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Comment {
    #[serde(rename = "uid")]
    pub id: u64,
    pub date: String,
    pub user: String,
    pub user_url: String,
    pub action: String,
    pub text: String,
    pub html: String,
}

#[derive(Debug, Default, PartialEq)]
pub struct Note {
    pub id: u64,
    pub lon: f64,
    pub lat: f64,
    pub url: String,
    pub comment_url: String,
    pub close_url: String,
    pub created_at: String,
    pub status: String,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Default, PartialEq, Serialize)]
pub struct NoteContent {
    pub lat: f64,
    pub lon: f64,
    pub text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteSearchSortOption {
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NoteSearchOrderOption {
    Oldest,
    Newest,
}

#[derive(Debug, Default, Serialize)]
pub struct NoteSearchOptions {
    pub q: String,
    pub limit: Option<u16>,
    pub closed: Option<i64>,
    pub display_name: Option<String>,
    pub user: Option<u64>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub sort: Option<NoteSearchSortOption>,
    pub order: Option<NoteSearchOrderOption>,
}
