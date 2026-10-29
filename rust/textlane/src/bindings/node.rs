use napi_derive::napi;
use serde_json::Value;
use textlane_core::File;

use crate::Api;

/// The package version.
#[napi]
pub fn version() -> &'static str {
    crate::read_version()
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
    pub fn run(&mut self, tree: Value) -> Result<Value> {
        self.inner
            .run(&tree, &mut File::new(), Api::Async)
            .map_err(throw)
    }

    #[napi]
    pub fn run_sync(&mut self, tree: Value) -> Result<Value> {
        self.inner
            .run(&tree, &mut File::new(), Api::Sync)
            .map_err(throw)
    }

    #[napi]
    pub fn stringify(&mut self, tree: Value) -> Result<String> {
        self.inner.stringify(&tree, &mut File::new()).map_err(throw)
    }

    #[napi]
    pub fn process(&mut self, text: String) -> Result<Value> {
        self.process_with(&text, Api::Async)
    }

    #[napi]
    pub fn process_sync(&mut self, text: String) -> Result<Value> {
        self.process_with(&text, Api::Sync)
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

    fn process_with(&mut self, text: &str, api: Api) -> Result<Value> {
        let mut file = File::new();
        let value = self.inner.process(text, &mut file, api).map_err(throw)?;
        Ok(crate::write_file(&file, &value))
    }
}

// napi sets a thrown error's `code` from its status, so the crate's code becomes `code`
fn throw(error: crate::Error) -> napi::Error<String> {
    napi::Error::new(error.code, error.message)
}
