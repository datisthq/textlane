use serde_json::json;
use textlane_core::{Error, UnistPoint};

use super::read_messages;

#[test]
fn reads_a_point_as_an_empty_position() -> Result<(), Error> {
    let messages = read_messages(&json!([
        { "reason": "a", "ruleId": "b", "source": "c", "place": { "line": 1, "column": 2 } },
        { "reason": "d" },
    ]))?;
    let point = UnistPoint {
        line: 1,
        column: 2,
        offset: None,
    };
    assert_eq!(messages[0].rule.as_deref(), Some("b"));
    assert_eq!(
        messages[0].place.as_ref().map(|place| place.end),
        Some(point)
    );
    assert_eq!(messages[1].place, None);
    Ok(())
}

#[test]
fn fails_on_a_message_without_reason() {
    let error = read_messages(&json!([{ "ruleId": "b" }])).err();
    assert_eq!(
        error.map(|error| error.code),
        Some("invalid-message".to_string())
    );
}
