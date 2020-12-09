use crate::types;
use crate::Openstreetmap;
use crate::OpenstreetmapError;

#[derive(Debug, Deserialize)]
#[serde(rename = "osm")]
struct Osm {
    pub changeset: types::Changeset,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "osm")]
struct OsmCreate {
    pub changeset: types::ChangesetCreate,
}

impl OsmCreate {
    pub fn new(changeset: types::ChangesetCreate) -> Self {
        OsmCreate { changeset }
    }
}

pub struct Changesets {
    client: Openstreetmap,
}

impl Changesets {
    pub fn new(client: &Openstreetmap) -> Self {
        Changesets {
            client: client.clone(),
        }
    }

    pub async fn create(
        &self,
        changeset: types::ChangesetCreate,
    ) -> Result<u32, OpenstreetmapError> {
        let body = Some(OsmCreate::new(changeset));
        let changeset_id = self
            .client
            .request::<OsmCreate, u32>(
                reqwest::Method::PUT,
                Some(&self.client.api_version),
                "changeset/create",
                body,
            )
            .await?;

        Ok(changeset_id)
    }

    pub async fn get(&self, changeset_id: u32) -> Result<types::Changeset, OpenstreetmapError> {
        let endpoint = format!("changeset/{}", changeset_id);
        let changeset = self
            .client
            .request::<(), Osm>(
                reqwest::Method::GET,
                Some(&self.client.api_version),
                &endpoint,
                None,
            )
            .await?
            .changeset;

        Ok(changeset)
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
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const CHANGESETS_CREATE_STR: &str = "188664";

    lazy_static! {
        static ref CHANGESETS_CREATE_BODY: types::ChangesetCreate = types::ChangesetCreate::new(
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

    const CHANGESET_STR: &str = r#"
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

    lazy_static! {
        static ref CREDENTIALS: Credentials = Credentials::Basic("user".into(), "password".into());
    }

    #[test]
    fn test_osm_serialise() {
        /*
        GIVEN an Osm instance
        WHEN serialised
        THEN matches the expectation
        */
        // GIVEN
        let osm_create = OsmCreate::new(types::ChangesetCreate::new(
            "0.6",
            "iD",
            vec![
                types::Tag::new("comment", "aaa"),
                types::Tag::new("created_by", "iD 2.19.5"),
                types::Tag::new("host", "https://master.apis.dev.openstreetmap.org/edit"),
                types::Tag::new("locale", "en-GB"),
                types::Tag::new("imagery_used", "Bing aerial imagery"),
                types::Tag::new("changesets_count", "1"),
            ],
        ));

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
                    <tag k="changesets_count" v="1"/>
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
                ResponseTemplate::new(200).set_body_raw(CHANGESETS_CREATE_STR, "text/plain"),
            )
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), CREDENTIALS.clone());

        // WHEN
        let actual = client
            .changesets()
            .create(CHANGESETS_CREATE_BODY.clone())
            .await
            .unwrap();

        // THEN
        let expected = CHANGESETS_CREATE_STR.parse::<u32>().unwrap();

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
            min_lon: 7.0191821,
            min_lat: 49.2785426,
            max_lon: 7.0197485,
            max_lat: 49.2793101,
            tags: vec![types::Tag::new("created_by", "JOSM 1.61")],
        };

        assert_eq!(actual, expected);
    }
}
