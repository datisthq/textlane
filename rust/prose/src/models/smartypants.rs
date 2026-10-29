use serde::Deserialize;
use serde::de::{Deserializer, Error};

/// Options of `prose.smartypants`, as retext-smartypants takes them.
#[derive(Clone, Debug, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SmartypantsOptions {
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
    pub quotes: bool,
}

/// The quote characters to use for double and single quotes.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct QuoteCharacters {
    pub double: String,
    pub single: String,
}

/// How `prose.smartypants` treats backticks: `false`, `true` or `'all'`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Backticks {
    Off,
    #[default]
    On,
    All,
}

/// How `prose.smartypants` treats dashes: `false`, `true`, `'oldschool'` or `'inverted'`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Dashes {
    Off,
    #[default]
    On,
    Oldschool,
    Inverted,
}

/// How `prose.smartypants` treats dots: `false`, `true`, `'spaced'` or `'unspaced'`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Ellipses {
    Off,
    #[default]
    On,
    Spaced,
    Unspaced,
}

impl Default for SmartypantsOptions {
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

#[derive(Deserialize)]
#[serde(untagged)]
enum Switch {
    Flag(bool),
    Name(String),
}

// `null` and a missing option both mean the default, as upstream's `!== false` checks do
fn read_mode<'de, D: Deserializer<'de>>(
    deserializer: D,
    names: &[&str],
) -> Result<Option<Result<bool, usize>>, D::Error> {
    match Option::<Switch>::deserialize(deserializer)? {
        None => Ok(None),
        Some(Switch::Flag(flag)) => Ok(Some(Ok(flag))),
        Some(Switch::Name(name)) => match names.iter().position(|known| *known == name) {
            Some(index) => Ok(Some(Err(index))),
            None => Err(D::Error::custom(format!(
                "unknown variant `{name}`, expected a boolean or one of {names:?}"
            ))),
        },
    }
}

fn read_switch<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    Ok(Option::<bool>::deserialize(deserializer)? != Some(false))
}

impl<'de> Deserialize<'de> for Backticks {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match read_mode(deserializer, &["all"])? {
            Some(Ok(false)) => Backticks::Off,
            Some(Err(_)) => Backticks::All,
            _ => Backticks::On,
        })
    }
}

impl<'de> Deserialize<'de> for Dashes {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match read_mode(deserializer, &["oldschool", "inverted"])? {
            Some(Ok(false)) => Dashes::Off,
            Some(Err(0)) => Dashes::Oldschool,
            Some(Err(_)) => Dashes::Inverted,
            _ => Dashes::On,
        })
    }
}

impl<'de> Deserialize<'de> for Ellipses {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match read_mode(deserializer, &["spaced", "unspaced"])? {
            Some(Ok(false)) => Ellipses::Off,
            Some(Err(0)) => Ellipses::Spaced,
            Some(Err(_)) => Ellipses::Unspaced,
            _ => Ellipses::On,
        })
    }
}
