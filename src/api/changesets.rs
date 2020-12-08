use crate::types;
use crate::Openstreetmap;
use crate::OpenstreetmapError;

#[derive(Debug, Serialize)]
#[serde(rename = "osm")]
struct Osm {
    pub changeset: types::Changeset,
}

impl Osm {
    pub fn new(changeset: types::Changeset) -> Self {
        Osm { changeset }
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

    pub async fn create(&self, changeset: types::Changeset) -> Result<u32, OpenstreetmapError> {
        let body = Some(Osm::new(changeset));
        let changeset_id = self
            .client
            .request::<Osm, u32>(reqwest::Method::PUT, Some("0.6"), "changesets/create", body)
            .await?;

        Ok(changeset_id)
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
        static ref CHANGESETS_CREATE_BODY: types::Changeset = types::Changeset::new(
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
        static ref CREDENTIALS: Credentials = Credentials::Basic("user".into(), "password".into());
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
            .and(path("/api/0.6/changesets/create"))
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

    #[test]
    fn test_osm_serialise() {
        /*
        GIVEN an Osm instance
        WHEN serialised
        THEN matches the expectation
        */
        // GIVEN
        let osm = Osm::new(types::Changeset::new(
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
        let actual = to_string(&osm).unwrap();

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
}
