use napi_derive::napi;

/// The package version.
#[napi]
pub fn version() -> &'static str {
    crate::read_version()
}
