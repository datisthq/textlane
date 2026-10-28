use textlane_core::Error;

use crate::models::options::{GfmOptions, StringifyOptions};

/// `md::stringify`'s options, checked once, with their defaults filled in.
#[derive(Clone, Debug)]
pub(crate) struct Settings {
    pub(crate) bullet: char,
    pub(crate) bullet_ordered: char,
    pub(crate) bullet_other: char,
    pub(crate) close_atx: bool,
    pub(crate) emphasis: char,
    pub(crate) fence: char,
    pub(crate) fences: bool,
    pub(crate) increment_list_marker: bool,
    pub(crate) list_item_indent: ListItemIndent,
    pub(crate) quote: char,
    pub(crate) resource_link: bool,
    pub(crate) rule: char,
    pub(crate) rule_repetition: usize,
    pub(crate) rule_spaces: bool,
    pub(crate) setext: bool,
    pub(crate) strong: char,
    pub(crate) tight_definitions: bool,
    /// GFM's options, when `md::gfm` is used: its handlers, unsafe patterns and joins.
    pub(crate) gfm: Option<GfmOptions>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ListItemIndent {
    One,
    Tab,
    Mixed,
}

impl Settings {
    /// Check `options` with mdast-util-to-markdown's messages.
    pub(crate) fn from_options(options: &StringifyOptions) -> Result<Settings, Error> {
        let bullet = read_marker(options.bullet.as_deref(), '*', "*+-", |marker| {
            format!(
                "Cannot serialize items with `{marker}` for `options.bullet`, expected `*`, `+`, or `-`"
            )
        })?;
        let bullet_other = match options.bullet_other.as_deref() {
            None | Some("") => {
                if bullet == '*' {
                    '-'
                } else {
                    '*'
                }
            }
            Some(other) => {
                let marker = read_marker(Some(other), '*', "*+-", |marker| {
                    format!(
                        "Cannot serialize items with `{marker}` for `options.bulletOther`, expected `*`, `+`, or `-`"
                    )
                })?;
                if marker == bullet {
                    return Err(invalid(format!(
                        "Expected `bullet` (`{bullet}`) and `bulletOther` (`{marker}`) to be different"
                    )));
                }
                marker
            }
        };
        let list_item_indent = match options.list_item_indent.as_deref() {
            None | Some("" | "one") => ListItemIndent::One,
            Some("tab") => ListItemIndent::Tab,
            Some("mixed") => ListItemIndent::Mixed,
            Some(style) => {
                return Err(invalid(format!(
                    "Cannot serialize items with `{style}` for `options.listItemIndent`, expected `tab`, `one`, or `mixed`"
                )));
            }
        };
        let rule_repetition = match options.rule_repetition {
            None | Some(0) => 3,
            Some(repetition) if repetition < 3 => {
                return Err(invalid(format!(
                    "Cannot serialize rules with repetition `{repetition}` for `options.ruleRepetition`, expected `3` or more"
                )));
            }
            Some(repetition) => usize::try_from(repetition).unwrap_or(usize::MAX),
        };
        Ok(Settings {
            bullet,
            bullet_ordered: read_marker(
                options.bullet_ordered.as_deref(),
                '.',
                ".)",
                |marker| {
                    format!(
                        "Cannot serialize items with `{marker}` for `options.bulletOrdered`, expected `.` or `)`"
                    )
                },
            )?,
            bullet_other,
            close_atx: options.close_atx,
            emphasis: read_marker(options.emphasis.as_deref(), '*', "*_", |marker| {
                format!(
                    "Cannot serialize emphasis with `{marker}` for `options.emphasis`, expected `*`, or `_`"
                )
            })?,
            fence: read_marker(options.fence.as_deref(), '`', "`~", |marker| {
                format!(
                    "Cannot serialize code with `{marker}` for `options.fence`, expected `` ` `` or `~`"
                )
            })?,
            fences: options.fences,
            increment_list_marker: options.increment_list_marker,
            list_item_indent,
            quote: read_marker(options.quote.as_deref(), '"', "\"'", |marker| {
                format!(
                    "Cannot serialize title with `{marker}` for `options.quote`, expected `\"`, or `'`"
                )
            })?,
            resource_link: options.resource_link,
            rule: read_marker(options.rule.as_deref(), '*', "*-_", |marker| {
                format!(
                    "Cannot serialize rules with `{marker}` for `options.rule`, expected `*`, `-`, or `_`"
                )
            })?,
            rule_repetition,
            rule_spaces: options.rule_spaces,
            setext: options.setext,
            strong: read_marker(options.strong.as_deref(), '*', "*_", |marker| {
                format!(
                    "Cannot serialize strong with `{marker}` for `options.strong`, expected `*`, or `_`"
                )
            })?,
            tight_definitions: options.tight_definitions,
            gfm: None,
        })
    }
}

// An empty string falls back to the default, as JavaScript's `||` does
fn read_marker(
    value: Option<&str>,
    default: char,
    allowed: &str,
    message: impl Fn(&str) -> String,
) -> Result<char, Error> {
    let value = match value {
        None | Some("") => return Ok(default),
        Some(value) => value,
    };
    let mut characters = value.chars();
    match (characters.next(), characters.next()) {
        (Some(marker), None) if allowed.contains(marker) => Ok(marker),
        _ => Err(invalid(message(value))),
    }
}

fn invalid(message: String) -> Error {
    Error::new("invalid-options", message)
}
