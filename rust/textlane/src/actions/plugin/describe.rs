use serde_json::{Map, Value, json};

use crate::actions::plugin::find::PLUGINS;

/// Each domain's plugins with the JSON Schema of their options (`null` for none), and the
/// definitions those schemas refer to, as `{domain: {plugins, $defs}}`.
pub fn describe_plugins() -> Value {
    let mut domains = Map::new();
    let mut generators: Vec<(&str, schemars::SchemaGenerator)> = Vec::new();
    for entry in PLUGINS {
        let domain = entry.name.split('.').next().unwrap_or_default();
        let index = match generators.iter().position(|(known, _)| *known == domain) {
            Some(index) => index,
            None => {
                let settings = schemars::generate::SchemaSettings::draft2020_12();
                generators.push((domain, settings.into_generator()));
                generators.len() - 1
            }
        };
        let generator = &mut generators[index].1;
        let options = entry.options.map(|options| options(generator));
        let plugins = domains
            .entry(domain)
            .or_insert_with(|| json!({ "plugins": [] }));
        if let Some(Value::Array(list)) = plugins.get_mut("plugins") {
            list.push(json!({ "name": entry.name, "upstream": entry.upstream, "options": options }));
        }
    }
    for (domain, mut generator) in generators {
        if let Some(Value::Object(described)) = domains.get_mut(domain) {
            described.insert(
                "$defs".into(),
                Value::Object(generator.take_definitions(true)),
            );
        }
    }
    Value::Object(domains)
}

#[cfg(test)]
#[path = "describe_unit.rs"]
mod unit;
