use serde::Deserialize;

/// `prose.parse`: natural language into nlcst.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "ParseOptions"))]
pub struct Parse {
    /// The rules sentences and words are found by.
    pub language: Language,
}

/// The language `prose.parse` reads: parse-latin's rules, or parse-english's on top of them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Latin,
    English,
}

/// `prose.equality`: warnings about insensitive, inconsiderate wording, as
/// retext-equality; options are retext-equality's.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "EqualityOptions"))]
pub struct Equality {
    /// Phrases not to warn about.
    pub ignore: Option<Vec<String>>,
    /// Whether a phrase used next to its counterpart (`he or she`) is fine.
    pub binary: Option<bool>,
}

/// `prose.readability`: warnings about sentences too hard to read, as retext-readability;
/// options are retext-readability's.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "ReadabilityOptions"))]
pub struct Readability {
    /// The age of the target audience: 16 by default.
    pub age: Option<f64>,
    /// The share of algorithms that must find a sentence too hard: 4/7 by default.
    pub threshold: Option<f64>,
    /// The fewest words a sentence needs to be checked: 5 by default.
    pub min_words: Option<f64>,
}

/// `prose.passive`: warnings about the passive voice, as retext-passive; options are
/// retext-passive's.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "PassiveOptions"))]
pub struct Passive {
    /// Participles not to warn about.
    pub ignore: Option<Vec<String>>,
}

/// `prose.sentenceSpacing`: warnings about the spaces between sentences, as
/// retext-sentence-spacing; options are retext-sentence-spacing's.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "SentenceSpacingOptions"))]
pub struct SentenceSpacing {
    /// The spacing expected between sentences: `1` or `"space"` (the default), `2` or
    /// `"double-space"`, `0` or `"newline"`.
    pub preferred: Option<Spacing>,
}

/// A spacing between sentences, as a count of spaces or by name.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Spacing {
    Count(f64),
    Name(String),
}
