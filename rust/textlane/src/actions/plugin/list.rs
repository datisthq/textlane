use crate::actions::plugin::find::PLUGINS;

/// The name of every public plugin this build carries, as a pipeline names it.
pub fn list_plugins() -> Vec<&'static str> {
    PLUGINS.iter().map(|entry| entry.name).collect()
}

#[cfg(test)]
#[path = "list_unit.rs"]
mod unit;
