use serde::Deserialize;

/// Options of `md::parse`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ParseOptions {}

/// Options of `md::to_html`, named as remark-rehype names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ToHtmlOptions {
    /// Keep raw HTML in the tree instead of dropping it.
    pub allow_dangerous_html: bool,
}
