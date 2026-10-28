use pyo3::prelude::*;

#[pymodule]
mod textlane {
    use pyo3::prelude::*;

    /// The package version.
    #[pyfunction]
    fn version() -> &'static str {
        crate::read_version()
    }

    /// Run one conformance case given as JSON; only in builds with `testing`.
    #[cfg(feature = "testing")]
    #[pyfunction]
    fn run_case(case: &str) -> PyResult<String> {
        let case = serde_json::from_str(case)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(crate::run_case(&case).to_string())
    }
}
