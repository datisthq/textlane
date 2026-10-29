use pyo3::exceptions::PyException;
use pyo3::prelude::*;

pyo3::create_exception!(textlane, TextlaneError, PyException);

#[pymodule(name = "_native")]
mod native {
    use pyo3::prelude::*;
    use serde_json::Value;
    use textlane_core::File;

    use crate::Api;

    #[pymodule_export]
    use super::TextlaneError;

    /// The package version.
    #[pyfunction]
    fn version() -> &'static str {
        crate::read_version()
    }

    /// The name of every public plugin, as a pipeline names it.
    #[pyfunction]
    fn plugin_names() -> Vec<&'static str> {
        crate::list_plugins()
    }

    /// Run one conformance case given as JSON; only in builds with `testing`.
    #[cfg(feature = "testing")]
    #[pyfunction]
    fn run_case(case: &str) -> PyResult<String> {
        let case = serde_json::from_str(case)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(crate::run_case(&case).to_string())
    }

    /// The crate's `Pipeline`: plugins, data and trees cross as JSON text.
    #[pyclass(unsendable)]
    struct Pipeline {
        inner: crate::Pipeline,
    }

    #[pymethods]
    impl Pipeline {
        #[new]
        fn new() -> Self {
            Self {
                inner: crate::Pipeline::new(),
            }
        }

        #[pyo3(name = "use", signature = (value, options=None))]
        fn use_value(
            &mut self,
            py: Python<'_>,
            value: &str,
            options: Option<&str>,
        ) -> PyResult<()> {
            let options = options.map(|options| read_json(py, options)).transpose()?;
            let value = read_json(py, value)?;
            self.inner
                .use_value(&value, options)
                .map_err(|error| throw(py, error))
        }

        #[pyo3(signature = (key=None))]
        fn data(&self, key: Option<&str>) -> String {
            self.inner.data(key).to_string()
        }

        fn set_data(&mut self, py: Python<'_>, key: &str, value: &str) -> PyResult<()> {
            let value = read_json(py, value)?;
            self.inner
                .set_data(key, value)
                .map_err(|error| throw(py, error))
        }

        fn replace_data(&mut self, py: Python<'_>, data: &str) -> PyResult<()> {
            let data = read_json(py, data)?;
            self.inner
                .replace_data(data)
                .map_err(|error| throw(py, error))
        }

        fn freeze(&mut self, py: Python<'_>) -> PyResult<()> {
            self.inner
                .freeze()
                .map(|_| ())
                .map_err(|error| throw(py, error))
        }

        fn parse(&mut self, py: Python<'_>, text: &str) -> PyResult<String> {
            let tree = self.inner.parse(text, &mut File::new());
            Ok(tree.map_err(|error| throw(py, error))?.to_string())
        }

        #[pyo3(signature = (tree, value=None))]
        fn run(
            &mut self,
            py: Python<'_>,
            tree: &str,
            value: Option<String>,
        ) -> PyResult<String> {
            self.run_with(py, tree, value, Api::Async)
        }

        #[pyo3(signature = (tree, value=None))]
        fn run_sync(
            &mut self,
            py: Python<'_>,
            tree: &str,
            value: Option<String>,
        ) -> PyResult<String> {
            self.run_with(py, tree, value, Api::Sync)
        }

        fn stringify(&mut self, py: Python<'_>, tree: &str) -> PyResult<String> {
            let tree = read_json(py, tree)?;
            let value = self.inner.stringify(&tree, &mut File::new());
            value.map_err(|error| throw(py, error))
        }

        fn process(&mut self, py: Python<'_>, text: &str) -> PyResult<String> {
            self.process_with(py, text, Api::Async)
        }

        fn process_sync(&mut self, py: Python<'_>, text: &str) -> PyResult<String> {
            self.process_with(py, text, Api::Sync)
        }
    }

    impl Pipeline {
        fn run_with(
            &mut self,
            py: Python<'_>,
            tree: &str,
            value: Option<String>,
            api: Api,
        ) -> PyResult<String> {
            let tree = read_json(py, tree)?;
            let mut file = File {
                value,
                ..File::new()
            };
            let tree = self.inner.run(&tree, &mut file, api);
            Ok(tree.map_err(|error| throw(py, error))?.to_string())
        }

        fn process_with(
            &mut self,
            py: Python<'_>,
            text: &str,
            api: Api,
        ) -> PyResult<String> {
            let mut file = File::new();
            let value = self.inner.process(text, &mut file, api);
            let value = value.map_err(|error| throw(py, error))?;
            Ok(crate::write_file(&file, &value).to_string())
        }
    }

    fn read_json(py: Python<'_>, text: &str) -> PyResult<Value> {
        serde_json::from_str(text).map_err(|error| {
            throw(py, crate::Error::new("invalid-json", error.to_string()))
        })
    }

    // The crate's code rides on the exception as `code`, as Node's errors carry it
    fn throw(py: Python<'_>, error: crate::Error) -> PyErr {
        let raised = TextlaneError::new_err(error.message);
        if let Err(failed) = raised.value(py).setattr("code", error.code) {
            return failed;
        }
        raised
    }
}
