use napi_derive::napi;

/// The package version.
#[napi]
pub fn version() -> &'static str {
    crate::read_version()
}

/// Run one conformance case given as JSON; only in addons built with `testing`.
#[cfg(feature = "testing")]
#[napi]
pub fn run_case(case: String) -> napi::Result<String> {
    let case = serde_json::from_str(&case)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(crate::run_case(&case).to_string())
}
