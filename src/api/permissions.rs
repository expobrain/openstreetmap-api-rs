use crate::types;
use crate::Openstreetmap;
use crate::OpenstreetmapError;
use crate::RequestOptions;

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
            .request::<(), Osm>(
                reqwest::Method::GET,
                "permissions",
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?
            .inner_permissions
            .permissions;

        Ok(permissions)
    }
}
