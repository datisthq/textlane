use serde::Deserialize;

/// Options of `html.stringify`, named as hast-util-to-html names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StringifyOptions {
    /// Write raw nodes as they are instead of escaping them.
    pub allow_dangerous_html: bool,
}
