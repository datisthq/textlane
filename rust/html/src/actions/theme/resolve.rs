use serde_json::Value;
use textlane_core::Error;

use crate::helpers::registry::load_registry;
use crate::helpers::theme_table::THEMES;
use crate::models::highlight::{Theme, Themes};
use crate::models::options::Highlight;

// Shiki's `defaultColor`, which takes plain `color` while the others take CSS variables
const DEFAULT_MODE: &str = "light";

/// The themes `html.highlight` colors with, failing as Shiki does on ones it lacks.
pub(crate) fn resolve_themes(options: &Highlight) -> Result<Themes, Error> {
    let Some(themes) = &options.themes else {
        let Some(name) = &options.theme else {
            return Err(invalid("either `theme` or `themes` must be provided"));
        };
        return Ok(Themes {
            list: vec![find_theme(DEFAULT_MODE, name)?],
            single: true,
        });
    };
    let mut list = vec![];
    for (mode, name) in themes {
        match name {
            Value::String(name) if !name.is_empty() => list.push(find_theme(mode, name)?),
            Value::Null | Value::Bool(false) | Value::String(_) => {}
            _ => return Err(invalid("a theme must be the name of one Shiki bundles")),
        }
    }
    if list.is_empty() {
        return Err(invalid("`themes` must not be empty"));
    }
    if !list.iter().any(|theme| theme.mode == DEFAULT_MODE) {
        return Err(invalid("`themes` must have a `light` theme"));
    }
    list.sort_by_key(|theme| theme.mode != DEFAULT_MODE);
    Ok(Themes {
        list,
        single: false,
    })
}

fn find_theme(mode: &str, name: &str) -> Result<Theme, Error> {
    let colors = THEMES.iter().find(|theme| theme.name == name);
    let registry = load_registry().ok_or_else(|| {
        Error::new("highlight-unavailable", "giallo's registry failed to load")
    })?;
    match colors {
        Some(colors) if registry.contains_theme(name) => Ok(Theme {
            mode: mode.to_owned(),
            colors,
        }),
        _ => Err(invalid(&format!("theme `{name}` is not one Shiki bundles"))),
    }
}

fn invalid(message: &str) -> Error {
    Error::new("invalid-options", format!("`html.highlight`: {message}"))
}
