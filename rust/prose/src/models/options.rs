use serde::Deserialize;

/// Options of `prose.parse`.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct ParseOptions {
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

/// Options of `prose.equality`, retext-equality's.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct EqualityOptions {
    /// Phrases not to warn about.
    pub ignore: Option<Vec<String>>,
    /// Whether a phrase used next to its counterpart (`he or she`) is fine.
    pub binary: Option<bool>,
}

/// Options of `prose.readability`, retext-readability's.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct ReadabilityOptions {
    /// The age of the target audience: 16 by default.
    pub age: Option<f64>,
    /// The share of algorithms that must find a sentence too hard: 4/7 by default.
    pub threshold: Option<f64>,
    /// The fewest words a sentence needs to be checked: 5 by default.
    pub min_words: Option<f64>,
}

/// Options of `prose.passive`, retext-passive's.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct PassiveOptions {
    /// Participles not to warn about.
    pub ignore: Option<Vec<String>>,
}

/// Options of `prose.sentenceSpacing`, retext-sentence-spacing's.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct SentenceSpacingOptions {
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
