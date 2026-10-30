use pyo3::exceptions::PyException;
use pyo3::prelude::*;

pyo3::create_exception!(textlane, TextlaneError, PyException);

#[pymodule(name = "_native")]
mod native {
    use pyo3::prelude::*;
    use serde_json::Value;
    use textlane_core::File;

    use crate::{Api, Outcome, Progress};

    #[pymodule_export]
    use super::TextlaneError;

    /// The package version.
    #[pyfunction]
    fn version() -> &'static str {
        crate::read_version()
    }

    /// The most memory, in bytes, each thread keeps after a document to reuse for the next.
    #[pyfunction]
    fn retained_memory() -> usize {
        crate::read_retained_memory()
    }

    /// Cap the memory, in bytes, each thread keeps after a document; `0` keeps none.
    #[pyfunction]
    fn set_retained_memory(bytes: usize) {
        crate::set_retained_memory(bytes);
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

        /// The ids of the enabled host plugins, in the order their stages run.
        fn hosts(&self) -> Vec<u32> {
            self.inner.hosts()
        }

        #[pyo3(signature = (tree, value, sync))]
        fn start_run(
            &mut self,
            py: Python<'_>,
            tree: &str,
            value: Option<String>,
            sync: bool,
        ) -> PyResult<Run> {
            let tree = read_json(py, tree)?;
            let file = File {
                value,
                ..File::new()
            };
            let progress = self.inner.start_run(&tree, file, read_api(sync));
            Ok(Run::new(progress.map_err(|error| throw(py, error))?))
        }

        fn start_process(
            &mut self,
            py: Python<'_>,
            text: &str,
            sync: bool,
        ) -> PyResult<Run> {
            let progress = self.inner.start_process(text, File::new(), read_api(sync));
            Ok(Run::new(progress.map_err(|error| throw(py, error))?))
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

    /// The crate's `Progress`: a run paused before host plugins until `resume`, then
    /// finished; trees and messages cross as JSON text.
    #[pyclass(unsendable)]
    struct Run {
        progress: Option<Progress>,
    }

    impl Run {
        fn new(progress: Progress) -> Self {
            Self {
                progress: Some(progress),
            }
        }
    }

    #[pymethods]
    impl Run {
        /// The ids of the host plugins to run next, in order; empty once finished.
        fn hosts(&self) -> Vec<u32> {
            match &self.progress {
                Some(Progress::Paused(pause)) => pause.hosts().to_vec(),
                _ => Vec::new(),
            }
        }

        /// The tree the next host plugin takes.
        fn tree(&self) -> String {
            match &self.progress {
                Some(Progress::Paused(pause)) => pause.tree().to_string(),
                _ => Value::Null.to_string(),
            }
        }

        /// The document's text, which the tree's positions point into.
        fn value(&self) -> Option<String> {
            match &self.progress {
                Some(Progress::Paused(pause)) => pause.file().value.clone(),
                _ => None,
            }
        }

        /// Every message the run reported so far, native stages' and host plugins', in order.
        fn messages(&self) -> String {
            match &self.progress {
                Some(Progress::Paused(pause)) => {
                    crate::write_messages(&pause.file().messages).to_string()
                }
                _ => "[]".to_string(),
            }
        }

        /// Continue with the tree the host plugins returned and the messages they reported.
        fn resume(&mut self, py: Python<'_>, tree: &str, messages: &str) -> PyResult<()> {
            let Some(Progress::Paused(mut pause)) = self.progress.take() else {
                let error =
                    crate::Error::new("finished-run", "Cannot resume a finished run");
                return Err(throw(py, error));
            };
            let tree = read_json(py, tree)?;
            let messages = read_json(py, messages)?;
            let messages =
                crate::read_messages(&messages).map_err(|error| throw(py, error))?;
            pause.file_mut().messages.extend(messages);
            let progress = pause.resume(&tree).map_err(|error| throw(py, error))?;
            self.progress = Some(progress);
            Ok(())
        }

        /// What the finished run produced: a vfile for a process, a tree for a run.
        fn result(&self) -> String {
            match &self.progress {
                Some(Progress::Finished(finished)) => match &finished.output {
                    Outcome::Text(text) => {
                        crate::write_file(&finished.file, text).to_string()
                    }
                    Outcome::Tree(tree) => tree.to_string(),
                },
                _ => Value::Null.to_string(),
            }
        }
    }

    fn read_api(sync: bool) -> Api {
        if sync { Api::Sync } else { Api::Async }
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
