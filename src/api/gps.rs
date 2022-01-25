use crate::types;
use crate::Openstreetmap;
use crate::OpenstreetmapError;
use crate::RequestOptions;

#[derive(Serialize, Deserialize)]
struct GpxList {
    #[serde(default, rename = "trk")]
    tracks: Vec<types::Track>,
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
    ) -> Result<Vec<types::Track>, OpenstreetmapError> {
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
                RequestOptions::new().with_version(),
            )
            .await?;

        Ok(())
    }
}
