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
    pub uid: u32,
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
    pub id: u32,
    pub user: String,
    pub uid: u32,
    pub created_at: String,
    pub open: bool,
    pub min_lon: f64,
    pub min_lat: f64,
    pub max_lon: f64,
    pub max_lat: f64,
    pub discussion: Option<Discussion>,
    #[serde(rename = "tag", default)]
    pub tags: Vec<Tag>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use quick_xml::se::to_string;

    #[test]
    fn test_changeset_create_serialize_xml() {
        /*
        GIVEN a ChangesetCreate instance
        WHEN serialised
        THEN matches the expectation
        */
        // GIVEN
        let changeset_create = ChangesetCreate::new(
            "0.6",
            "iD",
            vec![
                Tag::new("comment", "aaa"),
                Tag::new("created_by", "iD 2.19.5"),
                Tag::new("host", "https://master.apis.dev.openstreetmap.org/edit"),
                Tag::new("locale", "en-GB"),
                Tag::new("imagery_used", "Bing aerial imagery"),
                Tag::new("changesets_count", "1"),
            ],
        );

        // WHEN
        let actual = to_string(&changeset_create).unwrap();

        // THEN
        let expected = r#"
            <changeset version="0.6" generator="iD">
                <tag k="comment" v="aaa"/>
                <tag k="created_by" v="iD 2.19.5"/>
                <tag k="host" v="https://master.apis.dev.openstreetmap.org/edit"/>
                <tag k="locale" v="en-GB"/>
                <tag k="imagery_used" v="Bing aerial imagery"/>
                <tag k="changesets_count" v="1"/>
            </changeset>
        "#
        .split('\n')
        .map(|s| s.trim().into())
        .collect::<Vec<String>>()
        .join("");

        assert_eq!(actual, expected);
    }
}
