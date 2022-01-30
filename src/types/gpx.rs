#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Waypoint {
    #[serde(alias = "lat")]
    pub lat: f64,
    #[serde(alias = "lon")]
    pub lon: f64,
    pub time: Option<String>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TrackSegment {
    #[serde(alias = "trkpt")]
    pub points: Vec<Waypoint>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub name: Option<String>,
    pub comment: Option<String>,
    #[serde(alias = "desc")]
    pub description: Option<String>,
    pub url: Option<String>,
    pub source: Option<String>,
    pub number: Option<u32>,
    #[serde(default, alias = "trkseg")]
    pub segments: Vec<TrackSegment>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub id: u64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub lat: f64,
    pub lon: f64,
    pub user: Option<String>,
    #[serde(alias = "timestamp")]
    pub time: Option<String>,
    pub visibility: Option<String>,
    pub pending: bool,
    #[serde(default, alias = "tag")]
    pub keywords: Vec<String>,
}
