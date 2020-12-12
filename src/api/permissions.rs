use crate::types;
use crate::Openstreetmap;
use crate::OpenstreetmapError;

#[derive(Debug, Deserialize)]
struct InnerPermissions {
    #[serde(rename = "permission", default)]
    pub permissions: Vec<types::Permission>,
}

#[derive(Debug, Deserialize)]
struct Osm {
    #[serde(rename = "permissions")]
    pub inner_permissions: InnerPermissions,
}

pub struct Permissions {
    client: Openstreetmap,
}

impl Permissions {
    pub fn new(client: &Openstreetmap) -> Self {
        Permissions {
            client: client.clone(),
        }
    }

    pub async fn get(&self) -> Result<Vec<types::Permission>, OpenstreetmapError> {
        let permissions = self
            .client
            .request_including_version::<(), Osm>(
                reqwest::Method::GET,
                "permissions",
                types::RequestBody::None,
            )
            .await?
            .inner_permissions
            .permissions;

        Ok(permissions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[fixture]
    fn credentials() -> types::Credentials {
        types::Credentials::Basic("user".into(), "password".into())
    }

    #[rstest(response_str, expected,
        case(
            r#"
            <osm version="0.6" generator="OpenStreetMap server">
                <permissions>
                    <permission name="allow_read_prefs"/>
                    <permission name="allow_read_gpx"/>
                    <permission name="allow_write_gpx"/>
                </permissions>
            </osm>
            "#,
            vec![
                types::Permission {
                    name: "allow_read_prefs".into(),
                },
                types::Permission {
                    name: "allow_read_gpx".into(),
                },
                types::Permission {
                    name: "allow_write_gpx".into(),
                },
            ]
        )
    )]
    #[actix_rt::test]
    async fn test_get(
        credentials: types::Credentials,
        response_str: &str,
        expected: Vec<types::Permission>,
    ) {
        /*
        GIVEN an OSM client
        WHEN calling the permissions() function
        THEN returns a list of permissions for the current user
        */

        // GIVEN
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/0.6/permissions"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/xml"))
            .mount(&mock_server)
            .await;

        let client = Openstreetmap::new(mock_server.uri(), credentials);

        // WHEN
        let actual = client.permissions().await.unwrap();

        // THEN
        assert_eq!(actual, expected);
    }
}
