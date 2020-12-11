use std::error;
use std::fmt;

#[derive(Debug)]
pub enum OpenstreetmapError {
    /// error associated with http request
    Http(reqwest::Error),

    /// error caused by invalid URLs
    Url(url::ParseError),

    /// error associated with parsing or serializing
    Serde(quick_xml::de::DeError),

    /// error associated with parsing or serializing query strings
    UrlEncode(serde_urlencoded::ser::Error),

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

impl From<quick_xml::de::DeError> for OpenstreetmapError {
    fn from(error: quick_xml::de::DeError) -> Self {
        OpenstreetmapError::Serde(error)
    }
}

impl From<serde_urlencoded::ser::Error> for OpenstreetmapError {
    fn from(error: serde_urlencoded::ser::Error) -> Self {
        OpenstreetmapError::UrlEncode(error)
    }
}
