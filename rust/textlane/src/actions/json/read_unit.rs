use super::read_json;

#[test]
fn reads_json_nested_past_serde_jsons_limit() {
    let text = "[".repeat(2000) + &"]".repeat(2000);
    assert!(read_json(&text).is_ok());
}

#[test]
fn refuses_json_nested_past_a_trees_depth() {
    let text = "[".repeat(100_000) + &"]".repeat(100_000);
    let error = read_json(&text).err().map(|error| error.code);
    assert_eq!(error.as_deref(), Some("tree-too-deep"));
}

#[test]
fn counts_no_brackets_inside_strings() {
    let text = format!("[\"{}\\\"\"]", "[".repeat(10_000));
    assert!(read_json(&text).is_ok());
}

#[test]
fn rejects_trailing_text() {
    let error = read_json("{} x").err().map(|error| error.code);
    assert_eq!(error.as_deref(), Some("invalid-json"));
}
