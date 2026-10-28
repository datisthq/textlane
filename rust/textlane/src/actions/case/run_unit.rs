use super::*;

#[test]
fn run_case_parses_text_into_a_tree() {
    let case = json!({ "pipeline": [["test.parseText"]], "input": "alpha" });
    assert_eq!(
        run_case(&case)["tree"],
        json!({ "type": "text", "value": "alpha" })
    );
}

#[test]
fn run_case_round_trips_text() {
    let case =
        json!({ "pipeline": [["test.parseText"], ["test.compileValue"]], "input": "a" });
    assert_eq!(run_case(&case)["txt"], "a");
}

#[test]
fn run_case_compiles_a_given_tree() {
    let case = json!({ "pipeline": [["test.compileValue"]], "tree": { "type": "root" } });
    assert_eq!(run_case(&case)["txt"], "");
}

#[test]
fn run_case_reports_an_unknown_plugin() {
    let case = json!({ "pipeline": [["test.missing"]], "input": "a" });
    assert_eq!(run_case(&case)["error"]["code"], "unknown-plugin");
}

#[test]
fn run_case_reports_input_without_a_parser() {
    let case = json!({ "pipeline": [["test.compileValue"]], "input": "a" });
    assert_eq!(run_case(&case)["error"]["code"], "missing-parser");
}

#[test]
fn run_case_rejects_a_case_without_input() {
    let case = json!({ "pipeline": [] });
    assert_eq!(run_case(&case)["error"]["code"], "invalid-case");
}

#[test]
fn run_case_finishes_an_async_stage_under_the_async_api() {
    let tree = json!({ "type": "root", "children": [] });
    let operations = json!([
        ["use", "test.trace:a", { "mode": "async" }],
        ["run", tree, { "api": "async" }],
    ]);
    let case = json!({ "pipeline": [], "operations": operations });
    assert_eq!(run_case(&case)["trace"], json!(["a"]));
}
