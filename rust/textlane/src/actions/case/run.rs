use serde_json::{Value, json};
use textlane_core::{Allocator, Error, File, Lines};

use crate::models::pipeline::{Api, Pipeline, read_tree, write_tree};
use crate::utils::testing::plugins::take_trace;

/// Run one conformance case, `{pipeline, input | tree | operations}`, through the crate.
///
/// The result holds every outcome the case produced (`tree`, the output keyed by its
/// extension, `data`, `messages`, `trace`), or only `error`.
pub fn run_case(case: &Value) -> Value {
    take_trace();
    let mut file = File::new();
    let result = match read_case(case) {
        Ok((pipeline, Some(operations))) => {
            run_operations(pipeline, operations, &mut file)
        }
        Ok((pipeline, None)) => run_document(case, pipeline, &mut file),
        Err(error) => Err(error),
    };
    match result {
        Ok(mut result) => {
            result["messages"] = write_messages(&file);
            result["trace"] = json!(take_trace());
            result
        }
        Err(error) => {
            json!({ "error": { "code": error.code, "message": error.message } })
        }
    }
}

fn read_case(case: &Value) -> Result<(Pipeline, Option<&Vec<Value>>), Error> {
    let entries = case
        .get("pipeline")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::new("invalid-case", "a case needs a `pipeline` array"))?;
    let mut pipeline = Pipeline::new();
    for entry in entries {
        let name = entry.get(0).cloned().unwrap_or(Value::Null);
        pipeline.use_value(&name, entry.get(1).cloned())?;
    }
    Ok((pipeline, case.get("operations").and_then(Value::as_array)))
}

// A pipeline case parses its input when there is a parser, or reads its tree, then compiles
// when there is a compiler and otherwise gives the tree
fn run_document(
    case: &Value,
    mut pipeline: Pipeline,
    file: &mut File,
) -> Result<Value, Error> {
    let processor = pipeline.freeze()?;
    let alloc = Allocator::new();
    let input = case.get("input").and_then(Value::as_str);
    let lines = input.map(|input| Lines::new(processor.source(input)));
    let tree = match (input, case.get("tree")) {
        (Some(input), _) if processor.has_parser() => {
            processor.parse(input, file, &alloc)?
        }
        (_, Some(tree)) => read_tree(tree, processor.input_kind(), &alloc)?,
        (Some(_), None) => {
            return Err(Error::new(
                "missing-parser",
                "Cannot `parse` without `parser`",
            ));
        }
        (None, None) => {
            return Err(Error::new(
                "invalid-case",
                "a case needs either `input` or `tree`",
            ));
        }
    };
    let tree = processor.run_sync(tree, file, &alloc)?;
    let mut result = match processor.extension() {
        Some(extension) => json!({ extension: processor.stringify(&tree, file)? }),
        None => json!({ "tree": write_tree(&tree, lines.as_ref())? }),
    };
    result["data"] = pipeline.data(None);
    Ok(result)
}

// Each operation is one call on the processor; the last one's output is the case's
fn run_operations(
    mut pipeline: Pipeline,
    operations: &[Value],
    file: &mut File,
) -> Result<Value, Error> {
    let mut result = json!({});
    for operation in operations {
        let name = operation.get(0).and_then(Value::as_str).unwrap_or_default();
        let argument = |index: usize| operation.get(index).cloned();
        let api = match operation.get(2).and_then(|options| options.get("api")) {
            Some(Value::String(api)) if api == "async" => Api::Async,
            _ => Api::Sync,
        };
        result = match name {
            "use" => {
                let value = argument(1).unwrap_or(Value::Null);
                pipeline.use_value(&value, argument(2))?;
                json!({})
            }
            "data" => match (argument(1), argument(2)) {
                (Some(Value::String(key)), Some(value)) => {
                    pipeline.set_data(&key, value)?;
                    json!({})
                }
                (Some(Value::String(key)), None) => {
                    json!({ "data": pipeline.data(Some(&key)) })
                }
                _ => json!({ "data": pipeline.data(None) }),
            },
            "freeze" => {
                pipeline.freeze()?;
                json!({})
            }
            "parse" => {
                let input = argument(1).unwrap_or(Value::Null);
                json!({ "tree": pipeline.parse(input.as_str().unwrap_or_default(), file)? })
            }
            "run" => {
                let tree = argument(1).unwrap_or(Value::Null);
                json!({ "tree": pipeline.run(&tree, file, api)? })
            }
            "stringify" => {
                let tree = argument(1).unwrap_or(Value::Null);
                let value = pipeline.stringify(&tree, file)?;
                json!({ extension_of(&mut pipeline)?: value })
            }
            "process" => {
                let input = argument(1).unwrap_or(Value::Null);
                let value =
                    pipeline.process(input.as_str().unwrap_or_default(), file, api)?;
                json!({ extension_of(&mut pipeline)?: value })
            }
            _ => {
                return Err(Error::new(
                    "invalid-case",
                    format!("not an operation: {operation}"),
                ));
            }
        };
    }
    if result.get("data").is_none() {
        result["data"] = pipeline.data(None);
    }
    Ok(result)
}

fn extension_of(pipeline: &mut Pipeline) -> Result<&'static str, Error> {
    Ok(pipeline.freeze()?.extension().unwrap_or("txt"))
}

// Messages in the order they were reported, compared on their rule, source and place only
fn write_messages(file: &File) -> Value {
    let messages = file.messages.iter();
    let write_point = |point: &textlane_core::UnistPoint| json!({ "line": point.line, "column": point.column, "offset": point.offset });
    Value::Array(
        messages
            .map(|message| {
                json!({
                    "ruleId": message.rule,
                    "source": message.source,
                    "place": message.place.as_ref().map(|place| json!({
                        "start": write_point(&place.start),
                        "end": write_point(&place.end),
                    })),
                })
            })
            .collect(),
    )
}

#[cfg(test)]
#[path = "run_unit.rs"]
mod unit;
