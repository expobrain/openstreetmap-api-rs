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

#[derive(Debug, PartialEq, Deserialize)]
pub struct NodeRef {
    #[serde(rename = "ref")]
    pub node_id: u64,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Member {
    #[serde(rename = "type")]
    pub member_type: String,
    #[serde(rename = "ref")]
    pub node_id: u64,
    pub role: String,
}

#[derive(Debug, PartialEq, Deserialize)]
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

#[derive(Debug, PartialEq, Deserialize)]
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

#[derive(Debug, PartialEq, Deserialize)]
pub struct Modification {
    #[serde(rename = "node", default)]
    pub nodes: Vec<Node>,
    #[serde(rename = "way", default)]
    pub ways: Vec<Way>,
    #[serde(rename = "relation", default)]
    pub relations: Vec<Relation>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Creation {
    #[serde(rename = "node", default)]
    pub nodes: Vec<Node>,
    #[serde(rename = "way", default)]
    pub ways: Vec<Way>,
    #[serde(rename = "relation", default)]
    pub relations: Vec<Relation>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Deletion {
    #[serde(rename = "node", default)]
    pub nodes: Vec<Node>,
    #[serde(rename = "way", default)]
    pub ways: Vec<Way>,
    #[serde(rename = "relation", default)]
    pub relations: Vec<Relation>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct ChangesetChanges {
    #[serde(rename = "modify", default)]
    pub modifications: Vec<Modification>,
    #[serde(rename = "create", default)]
    pub creations: Vec<Creation>,
    #[serde(rename = "delete", default)]
    pub deletions: Vec<Deletion>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use quick_xml::de::from_str;
    use quick_xml::se::to_string;
    use rstest::rstest;

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

    #[rstest(
        data,
        expected,
        case(
            r#"
                <osmChange version="0.6" generator="acme osm editor">
                    <modify>
                        <node id="1234" changeset="42" version="2" lat="12.1234567" lon="-8.7654321" timestamp="2009-12-09T08:19:00Z" uid="1" user="user" visible="true">
                            <tag k="amenity" v="school"/>
                        </node>
                    </modify>
                </osmChange>
            "#
            , ChangesetChanges {
                modifications: vec![Modification {
                    nodes: vec![Node {
                        id: 1234,
                        changeset: 42,
                        version: 2,
                        uid: 1,
                        timestamp: "2009-12-09T08:19:00Z".into(),
                        user: "user".into(),
                        visible: true,
                        lat: 12.1234567,
                        lon: -8.7654321,
                        tags: vec![Tag {
                            k: "amenity".into(),
                            v: "school".into(),
                        }],
                    }],
                    ways: vec![],
                    relations: vec![],
                }],
                creations: vec![],
                deletions: vec![],
            }
        ),
        case(
            r#"
                <osmChange version="0.6" generator="acme osm editor">
                    <create>
                        <node id="1234" changeset="42" version="2" lat="12.1234567" lon="-8.7654321" timestamp="2009-12-09T08:19:00Z" uid="1" user="user" visible="true">
                            <tag k="amenity" v="school"/>
                        </node>
                    </create>
                </osmChange>
            "#
            , ChangesetChanges {
                modifications: vec![],
                creations:vec![Creation {
                    nodes: vec![Node {
                        id: 1234,
                        changeset: 42,
                        version: 2,
                        uid: 1,
                        timestamp: "2009-12-09T08:19:00Z".into(),
                        user: "user".into(),
                        visible: true,
                        lat: 12.1234567,
                        lon: -8.7654321,
                        tags: vec![Tag {
                            k: "amenity".into(),
                            v: "school".into(),
                        }],
                    }],
                    ways: vec![],
                    relations: vec![],
                }],
                deletions: vec![],
            }
        ),
        case(
            r#"
                <osmChange version="0.6" generator="acme osm editor">
                    <delete>
                        <node id="1234" changeset="42" version="2" lat="12.1234567" lon="-8.7654321" timestamp="2009-12-09T08:19:00Z" uid="1" user="user" visible="true">
                            <tag k="amenity" v="school"/>
                        </node>
                    </delete>
                </osmChange>
            "#
            , ChangesetChanges {
                modifications: vec![],
                creations: vec![],
                deletions:vec![Deletion {
                    nodes: vec![Node {
                        id: 1234,
                        changeset: 42,
                        version: 2,
                        uid: 1,
                        timestamp: "2009-12-09T08:19:00Z".into(),
                        user: "user".into(),
                        visible: true,
                        lat: 12.1234567,
                        lon: -8.7654321,
                        tags: vec![Tag {
                            k: "amenity".into(),
                            v: "school".into(),
                        }],
                    }],
                    ways: vec![],
                    relations: vec![],
                }],
            }
        ),
    )]
    fn test_changeset_change_deserilise_xml(data: &str, expected: ChangesetChanges) {
        /*
        GIVEN a string with XML data
        WHEN deserialising into a ChangesetChanges
        THEN matches the expectations
        */
        // WHEN
        let actual: ChangesetChanges = from_str(data).unwrap();

        // THEN
        assert_eq!(actual, expected);
    }
}
