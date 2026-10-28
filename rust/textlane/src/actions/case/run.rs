use serde_json::{Map, Value, json};

/// Run one conformance case, `{pipeline, input | tree}`, through the stub test plugins.
///
/// The result has one key, as a test's `expected` does: `txt`, `tree` or `error`.
pub fn run_case(case: &Value) -> Value {
    match run_pipeline(case) {
        Ok(result) => result,
        Err((code, message)) => json!({ "error": { "code": code, "message": message } }),
    }
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

#[cfg(test)]
#[path = "run_unit.rs"]
mod unit;
