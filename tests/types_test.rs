use openstreetmap_api::types::*;
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
                    uid: Some(1),
                    timestamp: "2009-12-09T08:19:00Z".into(),
                    user:Some( "user".into()),
                    visible: true,
                    lat: Some(12.1234567),
                    lon: Some(-8.7654321),
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
                    uid:Some( 1),
                    timestamp: "2009-12-09T08:19:00Z".into(),
                    user:Some( "user".into()),
                    visible: true,
                    lat: Some(12.1234567),
                    lon: Some(-8.7654321),
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
        "#,
        ChangesetChanges {
            modifications: vec![],
            creations: vec![],
            deletions:vec![Deletion {
                nodes: vec![Node {
                    id: 1234,
                    changeset: 42,
                    version: 2,
                    uid: Some(1),
                    timestamp: "2009-12-09T08:19:00Z".into(),
                    user:Some( "user".into()),
                    visible: true,
                    lat: Some(12.1234567),
                    lon: Some(-8.7654321),
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

#[rstest(
    element_id_param,
    expected,
    case(
        ElementIdParam { id: 1, version: None },
        "1",
    ),
    case(
        ElementIdParam { id: 1, version: Some(1) },
        "1v1",
    )
)]
fn test_element_id_param_display(element_id_param: ElementIdParam, expected: &str) {
    /*
    GIVEN an ElementIdParam instance
    WHEN rendering to a string
    THEN the string matches the expectation
    */
    // WHEN
    let actual = format!("{element_id_param}");

    // THEN
    assert_eq!(actual, expected);
}

#[rstest(
    id,
    version,
    expected,
    case(
        1,
        None,
        ElementIdParam { id: 1, version: None },
    ),
    case(
        1,
        Some(1),
        ElementIdParam { id: 1, version: Some(1) },
    )
)]
fn test_element_id_param_new(id: u64, version: Option<u64>, expected: ElementIdParam) {
    /*
    GIVEN an id and version
    WHEN calling new()
    THEN an ElementIdParam struct is returned
    */
    // WHEN
    let actual = ElementIdParam::new(id, version);

    // THEN
    assert_eq!(actual, expected);
}
