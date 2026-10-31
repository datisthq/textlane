use serde::Deserialize;
use textlane_core::read_switch;
pub use textlane_core::{Backticks, Dashes, Ellipses, QuoteCharacters};

/// `nl.smartypants`: straight quotes, dashes, dots and backticks into typographic
/// ones; options as retext-smartypants takes them.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "SmartypantsOptions"))]
pub struct Smartypants {
    /// Turn double backticks and double straight single quotes into double quotes; `all`
    /// turns single ones into single quotes too, and needs `quotes: false`.
    pub backticks: Backticks,
    /// The closing quotes to use: `”` and `’` by default.
    pub closing_quotes: Option<QuoteCharacters>,
    /// Turn two dashes into an em dash; `oldschool` and `inverted` treat three too.
    pub dashes: Dashes,
    /// Turn three dots into an ellipsis, with spaces between them or without.
    pub ellipses: Ellipses,
    /// The opening quotes to use: `“` and `‘` by default.
    pub opening_quotes: Option<QuoteCharacters>,
    /// Turn straight quotes into smart ones.
    #[serde(deserialize_with = "read_switch")]
    #[cfg_attr(feature = "schema", schemars(with = "Option<bool>"))]
    pub quotes: bool,
}

impl Default for Smartypants {
    fn default() -> Self {
        Self {
            backticks: Backticks::On,
            closing_quotes: None,
            dashes: Dashes::On,
            ellipses: Ellipses::On,
            opening_quotes: None,
            quotes: true,
        }
    }
}
