use super::*;

#[test]
fn run_case_parses_text_into_a_tree() {
    let case = json!({ "pipeline": [["test.parseText"]], "input": "alpha" });
    assert_eq!(
        run_case(&case),
        json!({ "tree": { "type": "text", "value": "alpha" } })
    );
}

#[test]
fn run_case_round_trips_text() {
    let case =
        json!({ "pipeline": [["test.parseText"], ["test.compileValue"]], "input": "a" });
    assert_eq!(run_case(&case), json!({ "txt": "a" }));
}

#[test]
fn run_case_compiles_a_given_tree() {
    let case = json!({ "pipeline": [["test.compileValue"]], "tree": { "type": "root" } });
    assert_eq!(run_case(&case), json!({ "txt": "" }));
}

#[test]
fn run_case_reports_an_unknown_plugin() {
    let case = json!({ "pipeline": [["test.missing"]], "input": "a" });
    assert_eq!(run_case(&case)["error"]["code"], "unknown-plugin");
}

#[test]
fn run_case_reports_a_plugin_out_of_order() {
    let case = json!({ "pipeline": [["test.compileValue"]], "input": "a" });
    assert_eq!(run_case(&case)["error"]["code"], "invalid-stage");
}

#[test]
fn run_case_rejects_a_case_without_input() {
    let case = json!({ "pipeline": [] });
    assert_eq!(run_case(&case)["error"]["code"], "invalid-case");
}

#[test]
fn run_case_reports_operations_as_unsupported() {
    let case = json!({ "pipeline": [], "operations": [["freeze"]] });
    assert_eq!(run_case(&case)["error"]["code"], "unsupported-operations");
}
