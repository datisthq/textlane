use serde::Deserialize;
use serde::de::{Deserializer, Error};

/// The quote characters to use for double and single quotes.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QuoteCharacters {
    pub double: String,
    pub single: String,
}

/// How smartypants treats backticks: `false`, `true` or `'all'`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Backticks {
    Off,
    #[default]
    On,
    All,
}

/// How smartypants treats dashes: `false`, `true`, `'oldschool'` or `'inverted'`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Dashes {
    Off,
    #[default]
    On,
    Oldschool,
    Inverted,
}

/// How smartypants treats dots: `false`, `true`, `'spaced'` or `'unspaced'`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Ellipses {
    Off,
    #[default]
    On,
    Spaced,
    Unspaced,
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

/// A switch upstream reads as `!== false`: `null` and a missing option mean `true`.
pub fn read_switch<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
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

#[cfg(feature = "schema")]
fn write_mode(names: &[&str]) -> schemars::Schema {
    schemars::json_schema!({ "anyOf": [{ "type": "boolean" }, { "enum": names }] })
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for Backticks {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Backticks".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        write_mode(&["all"])
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for Dashes {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Dashes".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        write_mode(&["oldschool", "inverted"])
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for Ellipses {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Ellipses".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        write_mode(&["spaced", "unspaced"])
    }
}

/// What a `Typographer` replaces, as retext-smartypants' options name it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Typography {
    pub quotes: bool,
    pub ellipses: Ellipses,
    pub backticks: Backticks,
    pub dashes: Dashes,
    pub opening_quotes: Option<QuoteCharacters>,
    pub closing_quotes: Option<QuoteCharacters>,
}
