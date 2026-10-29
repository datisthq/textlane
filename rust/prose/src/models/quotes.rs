use serde::Deserialize;

/// Options of `prose.quotes`, as retext-quotes takes them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct QuotesOptions {
    /// The style of quotes and apostrophes to prefer: `smart` by default.
    pub preferred: Option<QuoteStyle>,
    /// Smart quotes by nesting level, each one character or an opening and closing pair:
    /// `["“”", "‘’"]` by default.
    pub smart: Option<Vec<String>>,
    /// Straight quotes by nesting level: `["\"", "'"]` by default.
    pub straight: Option<Vec<String>>,
}

/// A style of quotes: straight (`"`) or smart (`“”`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum QuoteStyle {
    #[default]
    Smart,
    Straight,
}
