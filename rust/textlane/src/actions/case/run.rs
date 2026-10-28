use serde_json::{Map, Value, json};
use textlane_core::{Allocator, Error, File, Lines, Tree, TreeKind};

use crate::models::processor::Processor;

/// Run one conformance case, `{pipeline, input | tree}`, through the crate's plugins.
///
/// The result has one key, as a test's `expected` does: `txt`, `tree` or `error`.
pub fn run_case(case: &Value) -> Value {
    if let Some(pipeline) = read_plugin_pipeline(case) {
        return match run_processor(case, pipeline) {
            Ok(result) => result,
            Err(error) => {
                json!({ "error": { "code": error.code, "message": error.message } })
            }
        };
    }
    match run_pipeline(case) {
        Ok(result) => result,
        Err((code, message)) => json!({ "error": { "code": code, "message": message } }),
    }
}

// A pipeline of the crate's plugins; the `test.*` stub plugins run without the processor
fn read_plugin_pipeline(case: &Value) -> Option<&Vec<Value>> {
    let pipeline = case.get("pipeline").and_then(Value::as_array)?;
    let is_stub = |entry: &Value| {
        entry
            .get(0)
            .and_then(Value::as_str)
            .is_none_or(|name| name.starts_with("test."))
    };
    let plugins = case.get("operations").is_none() && !pipeline.iter().all(is_stub);
    plugins.then_some(pipeline)
}

type Failure = (&'static str, String);

fn run_pipeline(case: &Value) -> Result<Value, Failure> {
    let pipeline = case
        .get("pipeline")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            (
                "invalid-case",
                "a case needs a `pipeline` array".to_string(),
            )
        })?;
    if case.get("operations").is_some() {
        return Err((
            "unsupported-operations",
            "the stub runs pipelines, not operations".into(),
        ));
    }
    let mut state = match (case.get("input"), case.get("tree")) {
        (Some(Value::String(text)), None) => State::Text(text.clone()),
        (None, Some(tree)) => State::Tree(tree.clone()),
        _ => {
            return Err((
                "invalid-case",
                "a case needs either `input` or `tree`".into(),
            ));
        }
    };
    for entry in pipeline {
        let name = entry
            .get(0)
            .and_then(Value::as_str)
            .ok_or_else(|| ("invalid-case", format!("not a pipeline entry: {entry}")))?;
        state = apply_plugin(name, state)?;
    }
    Ok(match state {
        State::Text(text) => json!({ "txt": text }),
        State::Tree(tree) => json!({ "tree": tree }),
    })
}

enum State {
    Text(String),
    Tree(Value),
}

fn apply_plugin(name: &str, state: State) -> Result<State, Failure> {
    match (name, state) {
        ("test.parseText", State::Text(text)) => {
            let mut node = Map::new();
            node.insert("type".into(), "text".into());
            node.insert("value".into(), text.into());
            Ok(State::Tree(Value::Object(node)))
        }
        ("test.compileValue", State::Tree(tree)) => {
            let value = tree
                .get("value")
                .and_then(Value::as_str)
                .unwrap_or_default();
            Ok(State::Text(value.to_string()))
        }
        ("test.parseText" | "test.compileValue", _) => Err((
            "invalid-stage",
            format!("`{name}` cannot run at this point of the pipeline"),
        )),
        _ => Err(("unknown-plugin", format!("no plugin named `{name}`"))),
    }
}

fn run_processor(case: &Value, pipeline: &[Value]) -> Result<Value, Error> {
    let mut processor = Processor::new();
    for entry in pipeline {
        let name = entry.get(0).and_then(Value::as_str).unwrap_or_default();
        let options = entry.get(1).cloned().unwrap_or(Value::Null);
        processor = add_plugin(processor, name, options)?;
    }
    let processor = processor.freeze()?;
    let alloc = Allocator::new();
    let mut file = File::new();
    let input = case.get("input").and_then(Value::as_str);
    let lines = input.map(Lines::new);
    let tree = match (input, case.get("tree")) {
        (Some(input), _) if processor.has_parser() => {
            processor.parse(input, &mut file, &alloc)?
        }
        (_, Some(tree)) => read_tree(tree, processor.input_kind(), &alloc)?,
        _ => return Err(Error::new("missing-parser", "the processor has no parser")),
    };
    let tree = processor.run(tree, &mut file, &alloc)?;
    match processor.extension() {
        Some(extension) => {
            let value = processor.stringify(&tree, &mut file)?;
            Ok(json!({ extension: value }))
        }
        None => Ok(json!({ "tree": write_tree(&tree, lines.as_ref())? })),
    }
}

fn add_plugin(
    processor: Processor,
    name: &str,
    options: Value,
) -> Result<Processor, Error> {
    Ok(match name {
        #[cfg(feature = "md")]
        "md.parse" => processor.with(textlane_md::parse(read_options(options)?)),
        #[cfg(feature = "md")]
        "md.gemoji" => processor.with(textlane_md::gemoji()),
        #[cfg(feature = "md")]
        "md.stringify" => processor.with(textlane_md::stringify(read_options(options)?)),
        #[cfg(feature = "md")]
        "md.toHtml" => processor.with(textlane_md::to_html(read_options(options)?)),
        #[cfg(feature = "html")]
        "html.stringify" => {
            processor.with(textlane_html::stringify(read_options(options)?))
        }
        _ => {
            return Err(Error::new(
                "unknown-plugin",
                format!("no plugin named `{name}`"),
            ));
        }
    })
}

fn read_options<T: serde::de::DeserializeOwned + Default>(
    options: Value,
) -> Result<T, Error> {
    if options.is_null() {
        return Ok(T::default());
    }
    serde_json::from_value(options)
        .map_err(|error| Error::new("invalid-options", error.to_string()))
}

fn read_tree<'a>(
    tree: &Value,
    kind: Option<TreeKind>,
    alloc: &'a Allocator,
) -> Result<Tree<'a>, Error> {
    match kind {
        Some(TreeKind::Hast) => Ok(Tree::Hast(textlane_core::hast::deserialize_hast(
            tree, alloc,
        )?)),
        _ => Ok(Tree::Mdast(textlane_core::mdast::deserialize_mdast(
            tree, alloc,
        )?)),
    }
}

fn write_tree(tree: &Tree<'_>, lines: Option<&Lines<'_>>) -> Result<Value, Error> {
    match tree {
        Tree::Mdast(node) => textlane_core::mdast::serialize_mdast(node, lines),
        Tree::Hast(node) => textlane_core::hast::serialize_hast(node, lines),
    }
}

#[cfg(test)]
#[path = "run_unit.rs"]
mod unit;
