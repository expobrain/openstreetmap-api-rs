#[derive(Debug, Clone)]
pub enum Credentials {
    Basic(String, String), // Username, password
}
