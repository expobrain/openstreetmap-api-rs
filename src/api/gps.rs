use crate::types;
use crate::Openstreetmap;
use crate::OpenstreetmapError;
use crate::RequestOptions;

#[derive(Serialize, Deserialize)]
struct GpxList {
    #[serde(default, rename = "trk")]
    tracks: Vec<types::gpx::Track>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct OsmSingle {
    #[serde(default, rename = "gpx_file")]
    metadata: types::gpx::Metadata,
}

pub struct Gps {
    client: Openstreetmap,
}

impl Gps {
    pub fn new(client: &Openstreetmap) -> Self {
        Gps {
            client: client.clone(),
        }
    }

    pub async fn get_by_bounding_box(
        &self,
        bbox: &types::BoundingBox,
        page: Option<u64>,
    ) -> Result<Vec<types::gpx::Track>, OpenstreetmapError> {
        let mut url = format!(
            "trackpoints?bbox={},{},{},{}",
            bbox.left, bbox.bottom, bbox.right, bbox.top
        );

        if let Some(page_num) = page {
            url = format!("{url}&page={page_num}")
        }

        let tracks = self
            .client
            .request::<(), GpxList>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?
            .tracks;

        Ok(tracks)
    }

    pub async fn delete(&self, gpx_id: u64) -> Result<(), OpenstreetmapError> {
        let url = format!("gpx/{gpx_id}");

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

    pub async fn get_metadata(
        &self,
        gpx_id: u64,
    ) -> Result<types::gpx::Metadata, OpenstreetmapError> {
        let url = format!("gpx/{gpx_id}/details");

        let metadata = self
            .client
            .request::<(), OsmSingle>(
                reqwest::Method::GET,
                &url,
                types::RequestBody::None,
                RequestOptions::new().with_version(),
            )
            .await?
            .metadata;

        Ok(metadata)
    }
}