use napi::bindgen_prelude::AsyncTask;
use napi::{Env, Task};
use napi_derive::napi;
use serde_json::Value;
use textlane_core::File;

use crate::{Api, Outcome, Progress};

/// The package version.
#[napi]
pub fn version() -> &'static str {
    crate::read_version()
}

/// The most memory, in bytes, each thread keeps after a document to reuse for the next.
#[napi]
pub fn retained_memory() -> i64 {
    i64::try_from(crate::read_retained_memory()).unwrap_or(i64::MAX)
}

/// Cap the memory, in bytes, each thread keeps after a document; `0` keeps none.
#[napi]
pub fn set_retained_memory(bytes: i64) -> napi::Result<()> {
    let bytes = usize::try_from(bytes).map_err(|_| {
        napi::Error::new(napi::Status::InvalidArg, "bytes must not be negative")
    })?;
    crate::set_retained_memory(bytes);
    Ok(())
}

/// The name of every public plugin, as a pipeline names it.
#[napi]
pub fn plugin_names() -> Vec<String> {
    crate::list_plugins()
        .into_iter()
        .map(str::to_string)
        .collect()
}

/// Run one conformance case given as JSON; only in addons built with `testing`.
#[cfg(feature = "testing")]
#[napi]
pub fn run_case(case: String) -> napi::Result<String> {
    let case = serde_json::from_str(&case)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(crate::run_case(&case).to_string())
}

type Result<T> = napi::Result<T, String>;

/// The crate's `Pipeline`: plugins, data and trees cross as JSON values.
#[napi]
pub struct Pipeline {
    inner: crate::Pipeline,
}

#[napi]
impl Pipeline {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: crate::Pipeline::new(),
        }
    }

    #[napi(js_name = "use")]
    pub fn use_value(&mut self, value: Value, options: Option<Value>) -> Result<()> {
        self.inner.use_value(&value, options).map_err(throw)
    }

    #[napi]
    pub fn data(&self, key: Option<String>) -> Value {
        self.inner.data(key.as_deref())
    }

    #[napi]
    pub fn set_data(&mut self, key: String, value: Value) -> Result<()> {
        self.inner.set_data(&key, value).map_err(throw)
    }

    #[napi]
    pub fn replace_data(&mut self, data: Value) -> Result<()> {
        self.inner.replace_data(data).map_err(throw)
    }

    #[napi]
    pub fn freeze(&mut self) -> Result<()> {
        self.inner.freeze().map(|_| ()).map_err(throw)
    }

    #[napi]
    pub fn parse(&mut self, text: String) -> Result<Value> {
        self.inner.parse(&text, &mut File::new()).map_err(throw)
    }

    #[napi]
    pub fn run(&mut self, tree: Value, value: Option<String>) -> Result<Value> {
        self.run_with(&tree, value, Api::Async)
    }

    #[napi]
    pub fn run_sync(&mut self, tree: Value, value: Option<String>) -> Result<Value> {
        self.run_with(&tree, value, Api::Sync)
    }

    #[napi]
    pub fn stringify(&mut self, tree: Value) -> Result<String> {
        self.inner.stringify(&tree, &mut File::new()).map_err(throw)
    }

    /// The ids of the enabled host plugins, in the order their stages run.
    #[napi]
    pub fn hosts(&self) -> Vec<u32> {
        self.inner.hosts()
    }

    #[napi]
    pub fn start_run(
        &mut self,
        tree: Value,
        value: Option<String>,
        sync: bool,
    ) -> Result<Run> {
        let file = File {
            value,
            ..File::new()
        };
        let progress = self.inner.start_run(&tree, file, read_api(sync));
        progress.map(Run::new).map_err(throw)
    }

    #[napi]
    pub fn start_process(
        &mut self,
        text: String,
        sync: bool,
        path: Option<String>,
    ) -> Result<Run> {
        let file = File {
            path,
            ..File::new()
        };
        let progress = self.inner.start_process(&text, file, read_api(sync));
        progress.map(Run::new).map_err(throw)
    }

    /// Process on a libuv worker thread, so documents run in parallel and the event loop
    /// stays free; resolves to the file or the error the crate raised. Takes `&self`, as
    /// napi holds a `&mut` borrow until the task settles: `freeze` first.
    #[napi(ts_return_type = "Promise<ProcessOutcome>")]
    pub fn process(
        &self,
        text: String,
        path: Option<String>,
    ) -> Result<AsyncTask<ProcessTask>> {
        let job = self.inner.prepare_process(Api::Async).map_err(throw)?;
        Ok(AsyncTask::new(ProcessTask { job, text, path }))
    }

    #[napi]
    pub fn process_sync(
        &mut self,
        text: String,
        path: Option<String>,
    ) -> Result<ProcessedFile> {
        self.process_with(&text, path, Api::Sync)
    }

    fn run_with(
        &mut self,
        tree: &Value,
        value: Option<String>,
        api: Api,
    ) -> Result<Value> {
        let mut file = File {
            value,
            ..File::new()
        };
        self.inner.run(tree, &mut file, api).map_err(throw)
    }

    fn process_with(
        &mut self,
        text: &str,
        path: Option<String>,
        api: Api,
    ) -> Result<ProcessedFile> {
        let mut file = File {
            path,
            ..File::new()
        };
        let value = self.inner.process(text, &mut file, api).map_err(throw)?;
        Ok(write_processed(value, file))
    }
}

fn write_processed(value: String, file: File) -> ProcessedFile {
    ProcessedFile {
        value,
        messages: crate::write_messages(&file.messages),
        data: Value::Object(file.data),
    }
}

/// One document's `process` on a libuv worker thread.
pub struct ProcessTask {
    job: crate::ProcessJob,
    text: String,
    path: Option<String>,
}

impl Task for ProcessTask {
    type Output = std::result::Result<ProcessedFile, crate::Error>;
    type JsValue = ProcessOutcome;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let mut file = File {
            path: self.path.take(),
            ..File::new()
        };
        let value = self.job.process(&self.text, &mut file);
        Ok(value.map(|value| write_processed(value, file)))
    }

    fn resolve(&mut self, _: Env, output: Self::Output) -> napi::Result<ProcessOutcome> {
        Ok(match output {
            Ok(file) => ProcessOutcome {
                file: Some(file),
                error: None,
            },
            Err(error) => ProcessOutcome {
                file: None,
                error: Some(ProcessError {
                    code: error.code,
                    message: error.message,
                }),
            },
        })
    }
}

/// What an async `process` settles with: the file, or the crate's error, whose `code` a
/// task's rejection could not carry.
#[napi(object)]
pub struct ProcessOutcome {
    pub file: Option<ProcessedFile>,
    pub error: Option<ProcessError>,
}

/// An error the crate raised, as a thrown one carries it.
#[napi(object)]
pub struct ProcessError {
    pub code: String,
    pub message: String,
}

/// A processed document: its text, messages and data. A typed object rather than a JSON
/// value, so the text is not copied once more on its way to JavaScript.
#[napi(object)]
pub struct ProcessedFile {
    pub value: String,
    pub messages: Value,
    pub data: Value,
}

/// The crate's `Progress`: a run paused before host plugins until `resume`, then finished.
#[napi]
pub struct Run {
    progress: Option<Progress>,
}

#[napi]
impl Run {
    fn new(progress: Progress) -> Self {
        Self {
            progress: Some(progress),
        }
    }

    /// The ids of the host plugins to run next, in order; empty once finished.
    #[napi]
    pub fn hosts(&self) -> Vec<u32> {
        match &self.progress {
            Some(Progress::Paused(pause)) => pause.hosts().to_vec(),
            _ => Vec::new(),
        }
    }

    /// The tree the next host plugin takes.
    #[napi]
    pub fn tree(&self) -> Value {
        match &self.progress {
            Some(Progress::Paused(pause)) => pause.tree().clone(),
            _ => Value::Null,
        }
    }

    /// The document's text, which the tree's positions point into.
    #[napi]
    pub fn value(&self) -> Option<String> {
        match &self.progress {
            Some(Progress::Paused(pause)) => pause.file().value.clone(),
            _ => None,
        }
    }

    /// Every message the run reported so far, native stages' and host plugins', in order.
    #[napi]
    pub fn messages(&self) -> Value {
        match &self.progress {
            Some(Progress::Paused(pause)) => {
                crate::write_messages(&pause.file().messages)
            }
            _ => Value::Array(Vec::new()),
        }
    }

    /// Continue with the tree the host plugins returned and the messages they reported.
    #[napi]
    pub fn resume(&mut self, tree: Value, messages: Value) -> Result<()> {
        let Some(Progress::Paused(mut pause)) = self.progress.take() else {
            return Err(napi::Error::new(
                "finished-run".to_string(),
                "Cannot resume a finished run",
            ));
        };
        let messages = crate::read_messages(&messages).map_err(throw)?;
        pause.file_mut().messages.extend(messages);
        self.progress = Some(pause.resume(&tree).map_err(throw)?);
        Ok(())
    }

    /// What the finished run produced: a vfile for a process, a tree for a run.
    #[napi]
    pub fn result(&self) -> Value {
        match &self.progress {
            Some(Progress::Finished(finished)) => match &finished.output {
                Outcome::Text(text) => crate::write_file(&finished.file, text),
                Outcome::Tree(tree) => tree.clone(),
            },
            _ => Value::Null,
        }
    }
}

fn read_api(sync: bool) -> Api {
    if sync { Api::Sync } else { Api::Async }
}

// napi sets a thrown error's `code` from its status, so the crate's code becomes `code`
fn throw(error: crate::Error) -> napi::Error<String> {
    napi::Error::new(error.code, error.message)
}
