#[derive(Debug, Clone)]
pub enum Credentials {
    Basic(String, String), // Username, password
}

#[derive(Debug, PartialEq)]
pub struct VersionRange {
    pub minimum: String,
    pub maximum: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Status {
    pub database: String,
    pub api: String,
    pub gpx: String,
}

#[derive(Debug, PartialEq)]
pub struct Capabilities {
    pub versions: VersionRange,
    pub maximum_area: f64,
    pub maximum_note_area: f64,
    pub tracepoints_per_page: u32,
    pub maximum_waynodes: u32,
    pub maximum_changeset_elements: u32,
    pub timeout: u32,
    pub status: Status,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Blacklist {
    pub regex: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Imagery {
    pub blacklist: Vec<Blacklist>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Policy {
    pub imagery: Imagery,
}

#[derive(Debug, PartialEq)]
pub struct CapabilitiesAndPolicy {
    pub capabilities: Capabilities,
    pub policy: Policy,
}
