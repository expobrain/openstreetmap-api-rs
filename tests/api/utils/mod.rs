use openstreetmap_api::types;
use rstest::fixture;

#[fixture]
#[inline]
pub fn credentials() -> types::Credentials {
    types::Credentials::Basic("user".into(), "password".into())
}

#[fixture]
#[inline]
pub fn no_credentials() -> types::Credentials {
    types::Credentials::None
}
