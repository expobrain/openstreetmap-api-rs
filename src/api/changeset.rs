use crate::types;
use crate::Openstreetmap;
use crate::OpenstreetmapError;

#[derive(Debug, Serialize)]
#[serde(rename = "changeset")]
struct ChangesetUpdate {
    #[serde(rename = "tag")]
    pub tags: Vec<types::Tag>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "osm")]
struct Osm {
    pub changeset: types::Changeset,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "osm")]
struct OsmCreate {
    #[serde(rename = "changeset")]
    pub changesets: Vec<types::ChangesetCreate>,
}

impl OsmCreate {
    pub fn new(changesets: Vec<types::ChangesetCreate>) -> Self {
        OsmCreate { changesets }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename = "osm")]
struct OsmUpdate {
    pub changeset: ChangesetUpdate,
}

impl OsmUpdate {
    pub fn new(tags: Vec<types::Tag>) -> Self {
        OsmUpdate {
            changeset: ChangesetUpdate { tags },
        }
    }
}

#[derive(Debug, Serialize)]
struct Comment<'a> {
    pub text: &'a str,
}

impl<'a> Comment<'a> {
    fn new(text: &'a str) -> Self {
        Comment { text }
    }
}

pub struct Changeset {
    client: Openstreetmap,
}

impl Changeset {
    pub fn new(client: &Openstreetmap) -> Self {
        Changeset {
            client: client.clone(),
        }
    }

    pub async fn create(
        &self,
        changesets: Vec<types::ChangesetCreate>,
    ) -> Result<u64, OpenstreetmapError> {
        let body = types::RequestBody::Xml(OsmCreate::new(changesets));
        let changeset_id = self
            .client
            .request::<OsmCreate, u64>(
                reqwest::Method::PUT,
                Some(&self.client.api_version),
                "changeset/create",
                body,
            )
            .await?;

        Ok(changeset_id)
    }

    pub async fn update_tags_on_changeset(
        &self,
        changeset_id: u64,
        tags: Vec<types::Tag>,
    ) -> Result<types::Changeset, OpenstreetmapError> {
        let body = types::RequestBody::Xml(OsmUpdate::new(tags));
        let url = format!("changeset/{}", changeset_id);
        let changeset = self
            .client
            .request::<OsmUpdate, Osm>(
                reqwest::Method::PUT,
                Some(&self.client.api_version),
                &url,
                body,
            )
            .await?
            .changeset;

        Ok(changeset)
    }

    #[inline]
    pub async fn get(&self, changeset_id: u64) -> Result<types::Changeset, OpenstreetmapError> {
        self.inner_get(changeset_id, false).await
    }

    #[inline]
    pub async fn get_with_discussion(
        &self,
        changeset_id: u64,
    ) -> Result<types::Changeset, OpenstreetmapError> {
        self.inner_get(changeset_id, true).await
    }

    async fn inner_get(
        &self,
        changeset_id: u64,
        include_discussions: bool,
    ) -> Result<types::Changeset, OpenstreetmapError> {
        let mut url = format!("changeset/{}", changeset_id);

        if include_discussions {
            url = format!("{}?include_discussion=true", url);
        }

        let changeset = self
            .client
            .request_including_version::<(), Osm>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
            )
            .await?
            .changeset;

        Ok(changeset)
    }

    pub async fn close(&self, changeset_id: u64) -> Result<(), OpenstreetmapError> {
        let url = format!("changeset/{}/close", changeset_id);

        // Use Vec<u8> because `serde` cannot deserialise EOF when using Unit;
        self.client
            .request_including_version::<(), Vec<u8>>(
                reqwest::Method::PUT,
                &url,
                types::RequestBody::None,
            )
            .await?;

        Ok(())
    }

    pub async fn download(
        &self,
        changeset_id: u64,
    ) -> Result<types::ChangesetChanges, OpenstreetmapError> {
        let url = format!("changeset/{}/download", changeset_id);

        let changes = self
            .client
            .request_including_version::<(), types::ChangesetChanges>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
            )
            .await?;

        Ok(changes)
    }

    pub async fn upload(
        &self,
        changeset_id: u64,
        changeset_change: types::ChangesetChanges,
    ) -> Result<types::DiffResult, OpenstreetmapError> {
        let url = format!("changeset/{}/upload", changeset_id);

        let diffs = self
            .client
            .request_including_version::<types::ChangesetChanges, types::DiffResult>(
                reqwest::Method::POST,
                &url,
                types::RequestBody::Xml(changeset_change),
            )
            .await?;

        Ok(diffs)
    }

    pub async fn comment(
        &self,
        changeset_id: u64,
        comment: &str,
    ) -> Result<(), OpenstreetmapError> {
        let url = format!("changeset/{}/comment", changeset_id);
        let body = types::RequestBody::Form(Comment::new(comment));

        // Use Vec<u8> because `serde` cannot deserialise EOF when using Unit;
        self.client
            .request_including_version::<Comment, Vec<u8>>(reqwest::Method::POST, &url, body)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use quick_xml::se::to_string;
    use rstest::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[fixture]
    fn credentials() -> types::Credentials {
        types::Credentials::Basic("user".into(), "password".into())
    }

    #[test]
    fn test_osm_serialise() {
        /*
        GIVEN an Osm instance
        WHEN serialised
        THEN matches the expectation
        */
        // GIVEN
        let osm_create = OsmCreate::new(vec![types::ChangesetCreate::new(
            "0.6",
            "iD",
            vec![
                types::Tag::new("comment", "aaa"),
                types::Tag::new("created_by", "iD 2.19.5"),
                types::Tag::new("host", "https://master.apis.dev.openstreetmap.org/edit"),
                types::Tag::new("locale", "en-GB"),
                types::Tag::new("imagery_used", "Bing aerial imagery"),
                types::Tag::new("changeset_count", "1"),
            ],
        )]);

        // WHEN
        let actual = to_string(&osm_create).unwrap();

        // THEN
        let expected = r#"
            <osm>
                <changeset version="0.6" generator="iD">
                    <tag k="comment" v="aaa"/>
                    <tag k="created_by" v="iD 2.19.5"/>
                    <tag k="host" v="https://master.apis.dev.openstreetmap.org/edit"/>
                    <tag k="locale" v="en-GB"/>
                    <tag k="imagery_used" v="Bing aerial imagery"/>
                    <tag k="changeset_count" v="1"/>
                </changeset>
            </osm>
        "#
        .split('\n')
        .map(|s| s.trim().into())
        .collect::<Vec<String>>()
        .join("");

        assert_eq!(actual, expected);
    }

    #[rstest(body, response_str, expected,
        case(
            types::ChangesetCreate::new(
                "0.6",
                "iD",
                vec![
                    types::Tag::new("comment", "aaa"),
                    types::Tag::new("created_by", "iD 2.19.5"),
                    types::Tag::new("host", "https://master.apis.dev.openstreetmap.org/edit"),
                    types::Tag::new("locale", "en_GB"),
                    types::Tag::new("imagery", "Bing aerial imagery"),
                    types::Tag::new("changeset_count", "1"),
                ]
            ),
            "188664",
            188664
        )
    )]
    #[actix_rt::test]
    async fn test_create(
        credentials: types::Credentials,
        body: types::ChangesetCreate,
        response_str: &str,
        expected: u64,
    ) {
        /*
        GIVEN an OSM client
            AND a changeset
        WHEN calling the create() function
        THEN returns the created changeset ID
        */

        // GIVEN
        let mock_server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/api/0.6/changeset/create"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "text/plain"))
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), credentials);

        // WHEN
        let actual = client.changeset().create(vec![body]).await.unwrap();

        // THEN
        assert_eq!(actual, expected);
    }

    #[rstest(response_str, expected,
        case(
            r#"
            <osm>
                <changeset id="10" user="fred" uid="123" created_at="2008-11-08T19:07:39+01:00" open="true" min_lon="7.0191821" min_lat="49.2785426" max_lon="7.0197485" max_lat="49.2793101">
                    <tag k="created_by" v="JOSM 1.61"/>
                </changeset>
            </osm>
            "#,
            types::Changeset {
                id: 10,
                user: "fred".into(),
                uid: 123,
                created_at: "2008-11-08T19:07:39+01:00".into(),
                closed_at: None,
                open: true,
                min_lon: Some(7.0191821),
                min_lat: Some(49.2785426),
                max_lon: Some(7.0197485),
                max_lat: Some(49.2793101),
                discussion: None,
                tags: vec![types::Tag {
                    k: "created_by".into(),
                    v: "JOSM 1.61".into(),
                }],
            }
        )
    )]
    #[actix_rt::test]
    async fn test_get(
        credentials: types::Credentials,
        response_str: &str,
        expected: types::Changeset,
    ) {
        /*
        GIVEN an OSM client
        WHEN calling the get() function with a changeset ID
        THEN returns the requested changeset
        */

        // GIVEN
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/0.6/changeset/10"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), credentials);

        // WHEN
        let actual = client.changeset().get(10).await.unwrap();

        // THEN
        assert_eq!(actual, expected);
    }

    #[rstest(response_str, expected,
        case(
            r#"
            <osm>
                <changeset id="10" user="fred" uid="123" created_at="2008-11-08T19:07:39+01:00" open="true" min_lon="7.0191821" min_lat="49.2785426" max_lon="7.0197485" max_lat="49.2793101">
                    <tag k="created_by" v="JOSM 1.61"/>
                    <discussion>
                        <comment date="2015-01-01T18:56:48Z" uid="1841" user="metaodi">
                            <text>Did you verify those street names?</text>
                        </comment>
                    </discussion>
                </changeset>
            </osm>
            "#,
            types::Changeset {
                id: 10,
                user: "fred".into(),
                uid: 123,
                created_at: "2008-11-08T19:07:39+01:00".into(),
                closed_at: None,
                open: true,
                min_lon: Some(7.0191821),
                min_lat: Some(49.2785426),
                max_lon: Some(7.0197485),
                max_lat: Some(49.2793101),
                discussion: Some(types::Discussion {
                    comments: vec![types::Comment {
                        date: "2015-01-01T18:56:48Z".into(),
                        uid: 1841,
                        user: "metaodi".into(),
                        text: "Did you verify those street names?".into(),
                    }],
                }),
                tags: vec![types::Tag {
                    k: "created_by".into(),
                    v: "JOSM 1.61".into(),
                }],
            }
        )
    )]
    #[actix_rt::test]
    async fn test_get_with_discussion(
        credentials: types::Credentials,
        response_str: &str,
        expected: types::Changeset,
    ) {
        /*
        GIVEN an OSM client
        WHEN calling the get() function with a changeset ID
            AND includes discussion
        THEN returns the requested changeset
        */

        // GIVEN
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/0.6/changeset/10"))
            .and(query_param("include_discussion", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), credentials);

        // WHEN
        let actual = client.changeset().get_with_discussion(10).await.unwrap();

        // THEN
        assert_eq!(actual, expected);
    }

    #[rstest(body, response_str, expected,
        case(
            vec![types::Tag::new("created_by", "JOSM 1.61")],
            r#"
            <osm>
                <changeset id="10" user="fred" uid="123" created_at="2008-11-08T19:07:39+01:00" open="true" min_lon="7.0191821" min_lat="49.2785426" max_lon="7.0197485" max_lat="49.2793101">
                    <tag k="created_by" v="JOSM 1.61"/>
                </changeset>
            </osm>
            "#,
            types::Changeset {
                id: 10,
                user: "fred".into(),
                uid: 123,
                created_at: "2008-11-08T19:07:39+01:00".into(),
                closed_at: None,
                open: true,
                min_lon: Some(7.0191821),
                min_lat: Some(49.2785426),
                max_lon: Some(7.0197485),
                max_lat: Some(49.2793101),
                discussion: None,
                tags: vec![types::Tag {
                    k: "created_by".into(),
                    v: "JOSM 1.61".into(),
                }],
            }
        )
    )]
    #[actix_rt::test]
    async fn test_update_tags_on_changeset(
        credentials: types::Credentials,
        body: Vec<types::Tag>,
        response_str: &str,
        expected: types::Changeset,
    ) {
        /*
        GIVEN an OSM client
        WHEN calling the update_tags_on_changeset() function with a changeset ID
            AND a list of tags
        THEN returns the updated changeset
        */

        // GIVEN
        let mock_server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/api/0.6/changeset/10"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), credentials);

        // WHEN
        let actual = client
            .changeset()
            .update_tags_on_changeset(10, body.clone())
            .await
            .unwrap();

        // THEN
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[actix_rt::test]
    async fn test_close(credentials: types::Credentials) {
        /*
        GIVEN an OSM client
        WHEN calling the close() function with a changeset ID
        THEN returns nothing
        */

        // GIVEN
        let mock_server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/api/0.6/changeset/10/close"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), credentials);

        // WHEN
        let actual = client.changeset().close(10).await.unwrap();

        // THEN
        let expected = ();

        assert_eq!(actual, expected);
    }

    #[rstest(response_str, expected,
        case(
            r#"
            <osmChange version="0.6" generator="acme osm editor">
                <modify>
                    <node id="1234" changeset="42" version="2" lat="12.1234567" lon="-8.7654321" timestamp="2009-12-09T08:19:00Z" uid="1" user="user" visible="true">
                        <tag k="amenity" v="school"/>
                    </node>
                </modify>
            </osmChange>
            "#,
            types::ChangesetChanges {
                modifications: vec![types::Modification {
                    nodes: vec![types::Node {
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
                    }],
                    ways: vec![],
                    relations: vec![],
                }],
                creations: vec![],
                deletions: vec![],
            }
        )
    )]
    #[actix_rt::test]
    async fn test_download(
        credentials: types::Credentials,
        response_str: &str,
        expected: types::ChangesetChanges,
    ) {
        /*
        GIVEN an OSM client
        WHEN calling the download() function with a changeset ID
            AND a list of tags
        THEN returns the updated changeset
        */

        // GIVEN
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/0.6/changeset/10/download"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), credentials);

        // WHEN
        let actual = client.changeset().download(10).await.unwrap();

        // THEN
        assert_eq!(actual, expected);
    }

    #[rstest( body, response_str, expected,
        case(
            types::ChangesetChanges {
                modifications: vec![types::Modification {
                    nodes: vec![types::Node {
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
                    }],
                    ways: vec![],
                    relations: vec![],
                }],
                creations: vec![],
                deletions: vec![],
            },
            r#"
            <diffResult generator="OpenStreetMap Server" version="0.6">
                <node old_id="1234" new_id="42" new_version="2" />
                <way old_id="1234" new_id="42" new_version="2" />
                <relation old_id="1234" new_id="42" new_version="2" />
            </diffResult>
            "#,
            types::DiffResult {
                nodes:vec![types::DiffNode { old_id:1234, new_id:42, new_version:2 }],
                ways:vec![types::DiffWay { old_id:1234, new_id:42, new_version:2 }],
                relations:vec![types::DiffRelation { old_id:1234, new_id:42, new_version:2 }],
            }
        ),
    )]
    #[actix_rt::test]
    async fn test_upload(
        credentials: types::Credentials,
        body: types::ChangesetChanges,
        response_str: &str,
        expected: types::DiffResult,
    ) {
        /*
        GIVEN an OSM client
        WHEN calling the upload() function with a changeset ID
            AND a ChangesetChange
        THEN returns the list of diffs
        */

        // GIVEN
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/0.6/changeset/10/upload"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), credentials);

        // WHEN
        let actual = client.changeset().upload(10, body).await.unwrap();

        // THEN
        assert_eq!(actual, expected);
    }

    #[rstest(changeset_id, comment, case(10, "my_comment"))]
    #[actix_rt::test]
    async fn test_comment(credentials: types::Credentials, changeset_id: u64, comment: &str) {
        /*
        GIVEN an OSM client
        WHEN calling the comment() function with a changeset ID
        THEN returns nothing
        */

        // GIVEN
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path(format!("/api/0.6/changeset/{}/comment", changeset_id)))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), credentials);

        // WHEN
        let actual = client
            .changeset()
            .comment(changeset_id, comment)
            .await
            .unwrap();

        // THEN
        assert_eq!(actual, ());
    }
}
