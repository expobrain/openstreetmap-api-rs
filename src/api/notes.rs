use crate::errors::OpenstreetmapError;
use crate::types;
use crate::Openstreetmap;
use crate::RequestOptions;

use urlencoding::encode;

#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
pub struct CommentsRaw {
    #[serde(default, rename = "comment")]
    comments: Vec<types::Comment>,
}

#[derive(Debug, Default, PartialEq, Deserialize)]
pub struct NoteRaw {
    pub id: u64,
    #[serde(rename = "@lon")]
    pub lon: f64,
    #[serde(rename = "@lat")]
    pub lat: f64,
    pub url: String,
    pub comment_url: String,
    pub close_url: String,
    #[serde(rename = "date_created")]
    pub created_at: String,
    pub status: String,
    pub comments: CommentsRaw,
}

impl From<NoteRaw> for types::Note {
    fn from(value: NoteRaw) -> types::Note {
        types::Note {
            id: value.id,
            lon: value.lon,
            lat: value.lat,
            url: value.url,
            comment_url: value.comment_url,
            close_url: value.close_url,
            created_at: value.created_at,
            status: value.status,
            comments: value.comments.comments,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct OsmList {
    #[serde(default, rename = "note")]
    notes: Vec<NoteRaw>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct OsmSingle {
    #[serde(default, rename = "note")]
    note: NoteRaw,
}

pub struct Notes {
    client: Openstreetmap,
}

impl Notes {
    pub fn new(client: &Openstreetmap) -> Self {
        Self {
            client: client.clone(),
        }
    }

    pub async fn get_by_bounding_box(
        &self,
        bbox: &types::BoundingBox,
        limit: Option<u16>,
        closed: Option<i64>,
    ) -> Result<Vec<types::Note>, OpenstreetmapError> {
        let mut url = format!(
            "notes?bbox={},{},{},{}",
            bbox.left, bbox.bottom, bbox.right, bbox.top
        );

        if let Some(limit_value) = limit {
            url = format!("{url}&limit={limit_value}");
        }

        if let Some(closed_value) = closed {
            url = format!("{url}&closed={closed_value}");
        }

        let notes = self
            .client
            .request::<(), OsmList>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?
            .notes
            .into_iter()
            .map(|n| n.into())
            .collect();

        Ok(notes)
    }

    pub async fn get(&self, note_id: u64) -> Result<types::Note, OpenstreetmapError> {
        let url = format!("notes/{note_id}");

        let note = self
            .client
            .request::<(), OsmSingle>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?
            .note
            .into();

        Ok(note)
    }

    pub async fn create(
        &self,
        note_content: types::NoteContent,
    ) -> Result<types::Note, OpenstreetmapError> {
        let text_encoded = encode(&note_content.text);
        let lat = note_content.lat;
        let lon = note_content.lon;

        let url = format!("notes?lat={lat}&lon={lon}&text={text_encoded}");

        let note = self
            .client
            .request::<types::NoteContent, OsmSingle>(
                reqwest::Method::POST,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version().with_auth(),
            )
            .await?
            .note
            .into();

        Ok(note)
    }

    pub async fn create_comment(
        &self,
        note_id: u64,
        text: &str,
    ) -> Result<types::Note, OpenstreetmapError> {
        let text_encoded = encode(text);
        let url = format!("notes/{note_id}/comment?text={text_encoded}");

        let note = self
            .client
            .request::<(), OsmSingle>(
                reqwest::Method::POST,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version().with_auth(),
            )
            .await?
            .note
            .into();

        Ok(note)
    }

    pub async fn close(&self, note_id: u64, text: &str) -> Result<types::Note, OpenstreetmapError> {
        let text_encoded = encode(text);
        let url = format!("notes/{note_id}/close?text={text_encoded}");

        let note = self
            .client
            .request::<(), OsmSingle>(
                reqwest::Method::POST,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version().with_auth(),
            )
            .await?
            .note
            .into();

        Ok(note)
    }

    pub async fn reopen(
        &self,
        note_id: u64,
        text: &str,
    ) -> Result<types::Note, OpenstreetmapError> {
        let text_encoded = encode(text);
        let url = format!("notes/{note_id}/reopen?text={text_encoded}");

        let note = self
            .client
            .request::<(), OsmSingle>(
                reqwest::Method::POST,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version().with_auth(),
            )
            .await?
            .note
            .into();

        Ok(note)
    }

    pub async fn search(
        &self,
        search_options: &types::NoteSearchOptions,
    ) -> Result<Vec<types::Note>, OpenstreetmapError> {
        let qs = serde_urlencoded::to_string(search_options)?;
        let url = format!("notes/search?{qs}");

        let notes = self
            .client
            .request::<(), OsmList>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?
            .notes
            .into_iter()
            .map(|n| n.into())
            .collect();

        Ok(notes)
    }

    pub async fn feed_by_bounding_box(
        &self,
        bbox: &types::BoundingBox,
    ) -> Result<Vec<types::Note>, OpenstreetmapError> {
        let url = format!(
            "notes/feed?bbox={},{},{},{}",
            bbox.left, bbox.bottom, bbox.right, bbox.top
        );

        let notes = self
            .client
            .request::<(), OsmList>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?
            .notes
            .into_iter()
            .map(|n| n.into())
            .collect();

        Ok(notes)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use quick_xml::de::from_str;
    use rstest::*;

    #[rstest(data, expected,
        case(
            r#"
            <note lon="0.1000000" lat="51.0000000">
                <id>16659</id>
                <url>https://master.apis.dev.openstreetmap.org/api/0.6/notes/16659</url>
                <comment_url>https://master.apis.dev.openstreetmap.org/api/0.6/notes/16659/comment</comment_url>
                <close_url>https://master.apis.dev.openstreetmap.org/api/0.6/notes/16659/close</close_url>
                <date_created>2019-06-15 08:26:04 UTC</date_created>
                <status>open</status>
                <comments>
                    <comment>
                        <date>2019-06-15 08:26:04 UTC</date>
                        <uid>1234</uid>
                        <user>userName</user>
                        <user_url>https://master.apis.dev.openstreetmap.org/user/userName</user_url>
                        <action>opened</action>
                        <text>ThisIsANote</text>
                        <html>&lt;p&gt;ThisIsANote&lt;/p&gt;</html>
                    </comment>
                </comments>
            </note>
            "#,
            types::Note {
                id: 16659,
                lat: 51.0000000,
                lon: 0.1000000,
                url: "https://master.apis.dev.openstreetmap.org/api/0.6/notes/16659".into(),
                comment_url: "https://master.apis.dev.openstreetmap.org/api/0.6/notes/16659/comment".into(),
                close_url: "https://master.apis.dev.openstreetmap.org/api/0.6/notes/16659/close".into(),
                created_at: "2019-06-15 08:26:04 UTC".into(),
                status: "open".into(),
                comments: vec![types::Comment {
                    id: 1234,
                    date: "2019-06-15 08:26:04 UTC".into(),
                    user: "userName".into(),
                    user_url: "https://master.apis.dev.openstreetmap.org/user/userName".into(),
                    action: "opened".into(),
                    text: "ThisIsANote".into(),
                    html: "<p>ThisIsANote</p>".into()
                }]
            }
        )
    )]
    fn test_note_raw_deserialise_into_note(data: &str, expected: types::Note) {
        /*
        GIVEN an notes's data
        WHEN deserialising
        THEN an Note struct is returned
        */
        // WHEN
        let actual_raw: NoteRaw = from_str(data).unwrap();
        let actual: types::Note = actual_raw.into();

        // THEN
        assert_eq!(actual, expected);
    }
}
