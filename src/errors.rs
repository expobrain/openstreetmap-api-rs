use reqwest;
// use serde_json;

#[derive(Debug)]
pub enum Error {
    /// error associated with http request
    Http(reqwest::Error),
    /// error associated with parsing or serializing
    // Serde(serde_json::error::Error),
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
