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
        let body = Some(OsmCreate::new(changesets));
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
        let body = Some(OsmUpdate::new(tags));
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
            .request_including_version::<(), Osm>(reqwest::Method::GET, &url, None)
            .await?
            .changeset;

        Ok(changeset)
    }

    pub async fn close(&self, changeset_id: u64) -> Result<(), OpenstreetmapError> {
        let url = format!("changeset/{}/close", changeset_id);

        // Use Vec<u8> because `serde` cannot deserialise EOF;
        self.client
            .request_including_version::<(), Vec<u8>>(reqwest::Method::PUT, &url, None)
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
                None,
            )
            .await?;

        Ok(changes)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::Credentials;
    use crate::Openstreetmap;

    use super::*;
    use lazy_static::lazy_static;
    use pretty_assertions::assert_eq;
    use quick_xml::se::to_string;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const CHANGESET_CREATE_STR: &str = "188664";

    lazy_static! {
        static ref CREDENTIALS: Credentials = Credentials::Basic("user".into(), "password".into());
    }

    lazy_static! {
        static ref CHANGESET_CREATE_BODY: types::ChangesetCreate = types::ChangesetCreate::new(
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
        );
    }

    lazy_static! {
        static ref CHANGESET_UPDATE_BODY: Vec<types::Tag> =
            vec![types::Tag::new("created_by", "JOSM 1.61")];
    }

    const CHANGESET_STR: &str = r#"
        <osm>
            <changeset id="10" user="fred" uid="123" created_at="2008-11-08T19:07:39+01:00" open="true" min_lon="7.0191821" min_lat="49.2785426" max_lon="7.0197485" max_lat="49.2793101">
                <tag k="created_by" v="JOSM 1.61"/>
            </changeset>
        </osm>
    "#;

    const CHANGESET_WITH_DISCUSSION_STR: &str = r#"
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
    "#;

    const CHANGESET_DOWNLOAD_STR: &str = r#"
        <osmChange version="0.6" generator="acme osm editor">
            <modify>
                <node id="1234" changeset="42" version="2" lat="12.1234567" lon="-8.7654321" timestamp="2009-12-09T08:19:00Z" uid="1" user="user" visible="true">
                    <tag k="amenity" v="school"/>
                </node>
            </modify>
        </osmChange>
    "#;

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

    #[actix_rt::test]
    async fn test_create() {
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
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(CHANGESET_CREATE_STR, "text/plain"),
            )
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), CREDENTIALS.clone());

        // WHEN
        let actual = client
            .changesets()
            .create(vec![CHANGESET_CREATE_BODY.clone()])
            .await
            .unwrap();

        // THEN
        let expected = CHANGESET_CREATE_STR.parse::<u64>().unwrap();

        assert_eq!(actual, expected);
    }

    #[actix_rt::test]
    async fn test_get() {
        /*
        GIVEN an OSM client
        WHEN calling the get() function with a changeset ID
        THEN returns the requested changeset
        */

        // GIVEN
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/0.6/changeset/10"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(CHANGESET_STR, "application/xml"))
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), CREDENTIALS.clone());

        // WHEN
        let actual = client.changesets().get(10).await.unwrap();

        // THEN
        let expected = types::Changeset {
            id: 10,
            user: "fred".into(),
            uid: 123,
            created_at: "2008-11-08T19:07:39+01:00".into(),
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
        };

        assert_eq!(actual, expected);
    }

    #[actix_rt::test]
    async fn test_get_with_discussion() {
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
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(CHANGESET_WITH_DISCUSSION_STR, "application/xml"),
            )
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), CREDENTIALS.clone());

        // WHEN
        let actual = client.changesets().get_with_discussion(10).await.unwrap();

        // THEN
        let expected = types::Changeset {
            id: 10,
            user: "fred".into(),
            uid: 123,
            created_at: "2008-11-08T19:07:39+01:00".into(),
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
        };

        assert_eq!(actual, expected);
    }

    #[actix_rt::test]
    async fn test_update_tags_on_changeset() {
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
            .respond_with(ResponseTemplate::new(200).set_body_raw(CHANGESET_STR, "application/xml"))
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), CREDENTIALS.clone());

        // WHEN
        let actual = client
            .changesets()
            .update_tags_on_changeset(10, CHANGESET_UPDATE_BODY.clone())
            .await
            .unwrap();

        // THEN
        let expected = types::Changeset {
            id: 10,
            user: "fred".into(),
            uid: 123,
            created_at: "2008-11-08T19:07:39+01:00".into(),
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
        };

        assert_eq!(actual, expected);
    }

    #[actix_rt::test]
    async fn test_close() {
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

        let client = Openstreetmap::new(mock_server.uri(), CREDENTIALS.clone());

        // WHEN
        let actual = client.changesets().close(10).await.unwrap();

        // THEN
        let expected = ();

        assert_eq!(actual, expected);
    }

    #[actix_rt::test]
    async fn test_download() {
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
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(CHANGESET_DOWNLOAD_STR, "application/xml"),
            )
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), CREDENTIALS.clone());

        // WHEN
        let actual = client.changesets().download(10).await.unwrap();

        // THEN
        let expected = types::ChangesetChanges {
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
        };

        assert_eq!(actual, expected);
    }
}
