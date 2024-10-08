use crate::errors::OpenstreetmapError;
use crate::types;
use crate::Openstreetmap;
use crate::RequestOptions;

#[derive(Debug, PartialEq, Deserialize)]
struct OsmSingle {
    pub user: UserRaw,
}

#[derive(Debug, PartialEq, Deserialize)]
struct OsmList {
    #[serde(default, rename = "user")]
    pub users: Vec<UserRaw>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Preference {
    #[serde(rename = "@k")]
    pub k: String,
    #[serde(rename = "@v")]
    pub v: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Preferences {
    #[serde(default, rename = "preference")]
    pub list: Vec<Preference>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename = "osm")]
struct OsmPreferences {
    pub preferences: Preferences,
}

impl From<OsmPreferences> for types::UserPreferences {
    fn from(value: OsmPreferences) -> types::UserPreferences {
        value
            .preferences
            .list
            .into_iter()
            .map(|p| (p.k, p.v))
            .collect()
    }
}

impl From<&types::UserPreferences> for OsmPreferences {
    fn from(preferences: &types::UserPreferences) -> Self {
        Self {
            preferences: Preferences {
                list: preferences
                    .iter()
                    .map(|(k, v)| Preference {
                        k: k.clone(),
                        v: v.clone(),
                    })
                    .collect(),
            },
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct BlockRaw {
    pub received: types::Block,
}

#[derive(Debug, PartialEq, Deserialize)]
struct Language {
    #[serde(rename = "$value")]
    pub lang: String,
}

#[derive(Debug, PartialEq, Deserialize)]
struct Languages {
    #[serde(default)]
    pub lang: Vec<Language>,
}

#[derive(Debug, Default, PartialEq, Deserialize)]
struct MessageSent {
    #[serde(rename = "@count")]
    pub count: u64,
}

#[derive(Debug, Default, PartialEq, Deserialize)]
struct MessageReceived {
    #[serde(rename = "@count")]
    pub count: u64,
    #[serde(rename = "@unread")]
    pub unread: u64,
}

#[derive(Debug, Default, PartialEq, Deserialize)]
struct MessagesRaw {
    pub sent: MessageSent,
    pub received: MessageReceived,
}

impl From<MessagesRaw> for types::Messages {
    fn from(value: MessagesRaw) -> types::Messages {
        types::Messages {
            received: value.received.count,
            unread: value.received.unread,
            sent: value.sent.count,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct UserRaw {
    #[serde(rename = "@id")]
    pub id: u64,
    #[serde(rename = "@display_name")]
    pub display_name: String,
    #[serde(rename = "@account_created")]
    pub account_created: String,
    #[serde(rename = "description")]
    pub description: Option<String>,
    #[serde(rename = "contributor-terms")]
    pub contributor_terms: types::ContributorTerms,
    #[serde(rename = "img")]
    pub image: Option<types::Image>,
    #[serde(default)]
    pub changesets: types::UserChangesets,
    #[serde(default)]
    pub traces: types::Traces,
    #[serde(default)]
    pub blocks: Vec<BlockRaw>,
    pub home: Option<types::CoordsView>,
    #[serde(default)]
    pub languages: Option<Languages>,
    #[serde(default)]
    pub messages: MessagesRaw,
}

impl From<UserRaw> for types::User {
    fn from(value: UserRaw) -> types::User {
        types::User {
            id: value.id,
            display_name: value.display_name,
            account_created: value.account_created,
            description: value.description,
            contributor_terms: value.contributor_terms,
            image: value.image,
            changesets: value.changesets,
            traces: value.traces,
            blocks: value.blocks.into_iter().map(|b| b.received).collect(),
            home: value.home,
            languages: value
                .languages
                .map(|l| l.lang.into_iter().map(|l| l.lang).collect())
                .unwrap_or_default(),
            messages: value.messages.into(),
        }
    }
}

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
        let url = format!("user/{user_id}");

        let user = self
            .client
            .request::<(), OsmSingle>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?
            .user
            .into();

        Ok(user)
    }

    pub async fn users(&self, user_ids: &[u64]) -> Result<Vec<types::User>, OpenstreetmapError> {
        let user_ids_raw = user_ids
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(",");
        let qs = serde_urlencoded::to_string([("users", user_ids_raw)])?;
        let url = format!("users?{qs}");

        let users = self
            .client
            .request::<(), OsmList>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?
            .users
            .into_iter()
            .map(|u| u.into())
            .collect();

        Ok(users)
    }

    pub async fn details(&self) -> Result<types::User, OpenstreetmapError> {
        let user = self
            .client
            .request::<(), OsmSingle>(
                reqwest::Method::GET,
                "user/details",
                types::RequestBody::None,
                RequestOptions::new().with_version().with_auth(),
            )
            .await?
            .user
            .into();

        Ok(user)
    }

    pub async fn preferences(&self) -> Result<types::UserPreferences, OpenstreetmapError> {
        let user = self
            .client
            .request::<(), OsmPreferences>(
                reqwest::Method::GET,
                "user/preferences",
                types::RequestBody::None,
                RequestOptions::new().with_version().with_auth(),
            )
            .await?
            .into();

        Ok(user)
    }

    pub async fn preferences_update(
        &self,
        preferences: &types::UserPreferences,
    ) -> Result<(), OpenstreetmapError> {
        let payload = types::RequestBody::Xml(OsmPreferences::from(preferences));

        // Use Vec<u8> because `serde` cannot deserialise EOF when using Unit;
        self.client
            .request::<OsmPreferences, Vec<u8>>(
                reqwest::Method::PUT,
                "user/preferences",
                payload,
                RequestOptions::new().with_version().with_auth(),
            )
            .await?;

        Ok(())
    }

    pub async fn preference(&self, key: &str) -> Result<String, OpenstreetmapError> {
        let url = format!("user/preferences/{key}");

        let user = self
            .client
            .request::<(), String>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version().with_auth(),
            )
            .await?;

        Ok(user)
    }

    pub async fn preference_update(
        &self,
        key: &str,
        value: &str,
    ) -> Result<(), OpenstreetmapError> {
        let url = format!("user/preferences/{key}");
        let payload = types::RequestBody::RawForm(value.into());

        self.client
            .request::<String, Vec<u8>>(
                reqwest::Method::PUT,
                &url,
                payload,
                RequestOptions::new().with_version().with_auth(),
            )
            .await?;

        Ok(())
    }

    pub async fn preference_delete(&self, key: &str) -> Result<(), OpenstreetmapError> {
        let url = format!("user/preferences/{key}");

        // Use Vec<u8> because `serde` cannot deserialise EOF when using Unit;
        self.client
            .request::<(), Vec<u8>>(
                reqwest::Method::DELETE,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version().with_auth(),
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use quick_xml::de::from_str;
    use quick_xml::se::to_string;
    use rstest::*;

    #[rstest(data, expected,
        case(
            r#"
            <user id="12023" display_name="jbpbis" account_created="2007-08-16T01:35:56Z">
                <description />
                <contributor-terms agreed="false"/>
                <img href="http://www.gravatar.com/avatar/c8c86cd15f60ecca66ce2b10cb6b9a00.jpg"/>
                <roles />
                <changesets count="1"/>
                <traces count="0"/>
                <blocks>
                    <received count="0" active="0"/>
                </blocks>
            </user>
            "#,
            types::User {
                id: 12023,
                display_name: "jbpbis".into(),
                account_created: "2007-08-16T01:35:56Z".into(),
                description: Some("".into()),
                image: Some(types::Image {
                    url: "http://www.gravatar.com/avatar/c8c86cd15f60ecca66ce2b10cb6b9a00.jpg".into()
                }),
                changesets: types::UserChangesets { count: 1 },
                blocks: vec![types::Block::default()],
                ..Default::default()
            }
        ),
        case(
            r#"
            <user display_name="Max Muster" account_created="2006-07-21T19:28:26Z" id="1234">
                <contributor-terms agreed="true" pd="true"/>
                <img href="https://www.openstreetmap.org/attachments/users/images/000/000/1234/original/someLongURLOrOther.JPG"/>
                <roles />
                <changesets count="4182"/>
                <traces count="513"/>
                <blocks>
                    <received count="0" active="0"/>
                </blocks>
                <home lat="49.4733718952806" lon="8.89285988577866" zoom="3"/>
                <description>The description of your profile</description>
                <languages>
                    <lang>de-DE</lang>
                    <lang>de</lang>
                    <lang>en-US</lang>
                    <lang>en</lang>
                </languages>
                <messages>
                    <received count="1" unread="0"/>
                    <sent count="0"/>
                </messages>
            </user>
            "#,
            types::User {
                id: 1234,
                display_name: "Max Muster".into(),
                account_created: "2006-07-21T19:28:26Z".into(),
                contributor_terms: types::ContributorTerms {
                    agreed: true,
                    public_domain: true
                },
                image: Some(types::Image {
                    url: "https://www.openstreetmap.org/attachments/users/images/000/000/1234/original/someLongURLOrOther.JPG".into()
                }),
                changesets: types::UserChangesets { count: 4182 },
                traces: types::Traces{ count: 513 },
                blocks: vec![types::Block::default()],
                home: Some(types::CoordsView {
                    lat: 49.4733718952806,
                    lon: 8.89285988577866,
                    zoom: 3
                }),
                description: Some("The description of your profile".into()),
                languages: vec![
                    "de-DE".into(),
                    "de".into(),
                    "en-US".into(),
                    "en".into(),
                ],
                messages: types::Messages {
                    received: 1,
                    ..Default::default()
                }
            }
        )
    )]
    fn test_user_raw_deserialise_into_user(data: &str, expected: types::User) {
        /*
        GIVEN an user's data
        WHEN deserialising
        THEN an User struct is returned
        */
        // WHEN
        let actual_raw: UserRaw = from_str(data).unwrap();
        let actual: types::User = actual_raw.into();

        // THEN
        assert_eq!(actual, expected);
    }

    #[rstest(data, expected,
        case(
            r#"
            <osm version="0.6" generator="OpenStreetMap server">
                <preferences>
                    <preference k="somekey" v="somevalue" />
                </preferences>
            </osm>
            "#,
            [("somekey".to_string(), "somevalue".to_string())]
                .iter()
                .cloned()
                .collect::<types::UserPreferences>()
        )
    )]
    fn test_osm_preferences_deserialise(data: &str, expected: types::UserPreferences) {
        /*
        GIVEN an OSM user's preferences data
        WHEN deserialising
        THEN the UserPreferences type is returned
        */
        // WHEN
        let actual_raw: OsmPreferences = from_str(data).unwrap();
        let actual: types::UserPreferences = actual_raw.into();

        // THEN
        assert_eq!(actual, expected);
    }

    #[rstest(preferences, expected,
        case(
            [("somekey".to_string(), "somevalue".to_string())]
                .iter()
                .cloned()
                .collect::<types::UserPreferences>(),
            vec![
                r#"<osm>"#,
                r#"<preferences>"#,
                r#"<preference k="somekey" v="somevalue"/>"#,
                r#"</preferences>"#,
                r#"</osm>"#,
            ].join("")
        )
    )]
    fn test_osm_preferences_serialise(preferences: types::UserPreferences, expected: String) {
        /*
        GIVEN an OSM user's preferences data
        WHEN deserialising
        THEN the UserPreferences type is returned
        */
        // GIVEN
        let payload = OsmPreferences::from(&preferences);

        // WHEN
        let actual = to_string(&payload).unwrap();

        // THEN
        assert_eq!(actual, expected);
    }
}
