use crate::types;
use crate::Openstreetmap;
use crate::OpenstreetmapError;
use crate::RequestOptions;

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
                "changeset/create",
                body,
                RequestOptions::new().with_version().with_auth(),
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
                &url,
                body,
                RequestOptions::new().with_version().with_auth(),
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
            .request::<(), Osm>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?
            .changeset;

        Ok(changeset)
    }

    pub async fn close(&self, changeset_id: u64) -> Result<(), OpenstreetmapError> {
        let url = format!("changeset/{}/close", changeset_id);

        // Use Vec<u8> because `serde` cannot deserialise EOF when using Unit;
        self.client
            .request::<(), Vec<u8>>(
                reqwest::Method::PUT,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version().with_auth(),
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
            .request::<(), types::ChangesetChanges>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
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
            .request::<types::ChangesetChanges, types::DiffResult>(
                reqwest::Method::POST,
                &url,
                types::RequestBody::Xml(changeset_change),
                RequestOptions::new().with_version().with_auth(),
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
            .request::<Comment, Vec<u8>>(
                reqwest::Method::POST,
                &url,
                body,
                RequestOptions::new().with_version().with_auth(),
            )
            .await?;

        Ok(())
    }

    pub async fn subscribe(
        &self,
        changeset_id: u64,
    ) -> Result<types::Changeset, OpenstreetmapError> {
        let url = format!("changeset/{}/subscribe", changeset_id);

        let changeset = self
            .client
            .request::<(), Osm>(
                reqwest::Method::POST,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version().with_auth(),
            )
            .await?
            .changeset;

        Ok(changeset)
    }

    pub async fn unsubscribe(
        &self,
        changeset_id: u64,
    ) -> Result<types::Changeset, OpenstreetmapError> {
        let url = format!("changeset/{}/unsubscribe", changeset_id);

        let changeset = self
            .client
            .request::<(), Osm>(
                reqwest::Method::POST,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version().with_auth(),
            )
            .await?
            .changeset;

        Ok(changeset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use quick_xml::se::to_string;

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
}
