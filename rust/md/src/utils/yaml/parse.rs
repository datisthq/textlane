use serde_json::Value;
use textlane_core::Error;

use crate::utils::yaml::compose::compose_document;
use crate::utils::yaml::convert::convert_document;

/// YAML text as the `yaml` package's `parse` gives it, then as JSON: one document under the
/// core schema of YAML 1.2, null when it is empty.
pub(crate) fn parse_yaml(source: &str) -> Result<Value, Error> {
    // yaml-rust2 keeps `unreachable!`s of its own: a panic on hostile input is a bad document,
    // not a crash of whoever embeds textlane
    let parsed = std::panic::catch_unwind(|| compose_document(source));
    let document = parsed.unwrap_or_else(|_| {
        Err(Error::new(
            "invalid-frontmatter",
            "The YAML parser failed on this frontmatter",
        ))
    })?;
    convert_document(&document)
}

#[cfg(test)]
#[path = "parse_unit.rs"]
mod unit;
