use pyo3::prelude::*;

#[pymodule]
mod textlane {
    use pyo3::prelude::*;

    /// The package version.
    #[pyfunction]
    fn version() -> &'static str {
        crate::read_version()
    }
}
