use crate::Openstreetmap;
use crate::OpenstreetmapError;

#[derive(Debug, Deserialize)]
struct Version {
    #[serde(rename = "$value")]
    pub version: String,
}

#[derive(Debug, Deserialize)]
struct Osm {
    #[serde(rename = "api", default)]
    pub versions: Vec<Version>,
}

pub struct Versions {
    client: Openstreetmap,
}

impl Versions {
    pub fn new(client: &Openstreetmap) -> Self {
        Versions {
            client: client.clone(),
        }
    }

    pub async fn get(&self) -> Result<Vec<String>, OpenstreetmapError> {
        let versions = self
            .client
            .request::<Osm>(reqwest::Method::GET, None, "versions", None)
            .await?
            .versions
            .into_iter()
            .map(|v| v.version)
            .collect::<Vec<String>>();

        Ok(versions)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::Credentials;
    use crate::Openstreetmap;

    use lazy_static::lazy_static;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const VERSION_STR: &str = r#"
        <osm generator="OpenStreetMap server" copyright="OpenStreetMap and contributors" attribution="http://www.openstreetmap.org/copyright" license="http://opendatacommons.org/licenses/odbl/1-0/">
            <api>
                <version>0.6</version>
            </api>
        </osm>
    "#;
    const VERSION_UNKNOWN_VERSION_STR: &str = r#"
        <osm generator="OpenStreetMap server" copyright="OpenStreetMap and contributors" attribution="http://www.openstreetmap.org/copyright" license="http://opendatacommons.org/licenses/odbl/1-0/">
            <api>
                <version>0.7</version>
            </api>
        </osm>
    "#;

    lazy_static! {
        static ref CREDENTIALS: Credentials = Credentials::Basic("user".into(), "password".into());
    }

    #[actix_rt::test]
    async fn test_get() {
        /*
        GIVEN an OSM client
        WHEN calling the versions() function
        THEN returns a list of supported versions
        */

        // GIVEN
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/versions"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(VERSION_STR, "application/xml"))
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), CREDENTIALS.clone());

        // WHEN
        let actual = client.versions().await.unwrap();

        // THEN
        let expected = vec!["0.6".to_string()];

        assert_eq!(actual, expected);
    }
    #[actix_rt::test]
    async fn test_get_returns_unknown_version() {
        /*
        GIVEN an OSM client
            AND an unknown version number
        WHEN calling the versions() function
        THEN returns a list of supported versions
        */

        // GIVEN
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/versions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(VERSION_UNKNOWN_VERSION_STR, "application/xml"),
            )
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), CREDENTIALS.clone());

        // WHEN
        let actual = client.versions().await.unwrap();

        // THEN
        let expected = vec!["0.7".to_string()];

        assert_eq!(actual, expected);
    }
}
