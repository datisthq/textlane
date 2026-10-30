use std::sync::LazyLock;

use regex::Regex;
use textlane_core::Error;
use yaml_rust2::parser::Tag;
use yaml_rust2::scanner::TScalarStyle;

use crate::utils::yaml::document::{Scalar, yaml_error};
use crate::utils::yaml::number::parse_digits;

const CORE: &str = "tag:yaml.org,2002:";

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Null,
    Bool,
    Int,
    Float,
}

struct Rule {
    kind: Kind,
    pattern: LazyLock<Option<Regex>>,
    resolve: fn(&str) -> Scalar,
}

// The `yaml` package's core schema, in its order: a plain scalar takes the first that matches
static RULES: [Rule; 8] = [
    Rule {
        kind: Kind::Null,
        pattern: LazyLock::new(|| Regex::new(r"^(?:~|[Nn]ull|NULL)?$").ok()),
        resolve: |_| Scalar::Null,
    },
    Rule {
        kind: Kind::Bool,
        pattern: LazyLock::new(|| Regex::new(r"^(?:[Tt]rue|TRUE|[Ff]alse|FALSE)$").ok()),
        resolve: |text| Scalar::Bool(text.starts_with(['t', 'T'])),
    },
    Rule {
        kind: Kind::Int,
        pattern: LazyLock::new(|| Regex::new(r"^0o[0-7]+$").ok()),
        resolve: |text| Scalar::Number(parse_digits(&text[2..], 8)),
    },
    Rule {
        kind: Kind::Int,
        pattern: LazyLock::new(|| Regex::new(r"^[-+]?[0-9]+$").ok()),
        resolve: |text| Scalar::Number(parse_digits(text, 10)),
    },
    Rule {
        kind: Kind::Int,
        pattern: LazyLock::new(|| Regex::new(r"^0x[0-9a-fA-F]+$").ok()),
        resolve: |text| Scalar::Number(parse_digits(&text[2..], 16)),
    },
    Rule {
        kind: Kind::Float,
        pattern: LazyLock::new(|| {
            Regex::new(r"^(?:[-+]?\.(?:inf|Inf|INF)|\.nan|\.NaN|\.NAN)$").ok()
        }),
        resolve: |text| {
            Scalar::Number(if text.to_ascii_lowercase().ends_with("nan") {
                f64::NAN
            } else if text.starts_with('-') {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            })
        },
    },
    Rule {
        kind: Kind::Float,
        pattern: LazyLock::new(|| {
            Regex::new(r"^[-+]?(?:\.[0-9]+|[0-9]+(?:\.[0-9]*)?)[eE][-+]?[0-9]+$").ok()
        }),
        resolve: parse_float,
    },
    Rule {
        kind: Kind::Float,
        pattern: LazyLock::new(|| Regex::new(r"^[-+]?(?:\.[0-9]+|[0-9]+\.[0-9]*)$").ok()),
        resolve: parse_float,
    },
];

/// A scalar's value under the core schema: an explicit tag first, then a plain scalar's
/// pattern; quoted and block scalars and unknown tags are strings, as `yaml` resolves them.
pub(crate) fn resolve_scalar(
    text: String,
    style: TScalarStyle,
    tag: Option<&Tag>,
) -> Result<Scalar, Error> {
    let Some(tag) = tag else {
        return Ok(match style {
            TScalarStyle::Plain => {
                resolve_with(&text, |_| true).unwrap_or(Scalar::String(text))
            }
            _ => Scalar::String(text),
        });
    };
    let kind = match (tag.handle == CORE).then_some(tag.suffix.as_str()) {
        Some("null") => Kind::Null,
        Some("bool") => Kind::Bool,
        Some("int") => Kind::Int,
        Some("float") => Kind::Float,
        Some(other @ ("binary" | "timestamp" | "merge")) => {
            return Err(yaml_error(format!("Unsupported tag: {CORE}{other}")));
        }
        _ => return Ok(Scalar::String(text)),
    };
    Ok(resolve_with(&text, |rule| rule.kind == kind).unwrap_or(Scalar::String(text)))
}

/// Fails on the `yaml` 1.1 collection tags, whose JavaScript values (`Set`, `Map`) JSON lacks.
pub(crate) fn check_collection_tag(
    tag: Option<&Tag>,
    mapping: bool,
) -> Result<(), Error> {
    let Some(tag) = tag.filter(|tag| tag.handle == CORE) else {
        return Ok(());
    };
    match (tag.suffix.as_str(), mapping) {
        ("set", true) | ("omap" | "pairs", false) => {
            Err(yaml_error(format!("Unsupported tag: {CORE}{}", tag.suffix)))
        }
        _ => Ok(()),
    }
}

fn resolve_with(text: &str, accept: impl Fn(&Rule) -> bool) -> Option<Scalar> {
    RULES
        .iter()
        .filter(|rule| accept(rule))
        .find(|rule| {
            rule.pattern
                .as_ref()
                .is_some_and(|pattern| pattern.is_match(text))
        })
        .map(|rule| (rule.resolve)(text))
}

fn parse_float(text: &str) -> Scalar {
    Scalar::Number(text.parse().unwrap_or(f64::NAN))
}
