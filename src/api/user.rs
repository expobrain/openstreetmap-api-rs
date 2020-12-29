use crate::errors::OpenstreetmapError;
use crate::types;
use crate::Openstreetmap;

pub struct User {
    client: Openstreetmap,
}

impl User {
    pub fn new(client: &Openstreetmap) -> Self {
        Self {
            client: client.clone(),
        }
    }

    pub async fn get(&self, user_id: u64) -> Result<types::User, OpenstreetmapError> {
        let url = format!("user/{}", user_id);

        let user = self
            .client
            .request_including_version::<(), types::User>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
            )
            .await?;

        Ok(user)
    }
}
