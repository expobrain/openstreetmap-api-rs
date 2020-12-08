use quick_xml::de::DeError;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum OpenstreetmapError {
    /// error associated with http request
    Http(reqwest::Error),

    /// error caused by invalid URLs
    Url(url::ParseError),

    /// error associated with parsing or serializing
    Serde(DeError),

    /// client request errors
    Client {
        code: reqwest::StatusCode,
        error: String,
    },

    /// invalid credentials
    Unauthorized,

    /// HTTP method is not allowed
    MethodNotAllowed,

    /// Page not found
    NotFound,
}

impl error::Error for OpenstreetmapError {}

impl fmt::Display for OpenstreetmapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SuperError is here!")
    }
}

impl From<reqwest::Error> for OpenstreetmapError {
    fn from(error: reqwest::Error) -> Self {
        OpenstreetmapError::Http(error)
    }
}

impl From<url::ParseError> for OpenstreetmapError {
    fn from(error: url::ParseError) -> Self {
        OpenstreetmapError::Url(error)
    }
}

impl From<DeError> for OpenstreetmapError {
    fn from(error: DeError) -> Self {
        OpenstreetmapError::Serde(error)
    }
}
