use serde_json::json;
use textlane_core::File;

use super::write_file;

#[test]
fn writes_value_messages_and_data() {
    let mut file = File::new();
    file.warn("too long", Some("length"));
    assert_eq!(
        write_file(&file, "a"),
        json!({
            "value": "a",
            "messages": [{ "reason": "too long", "ruleId": "length", "source": null, "place": null }],
            "data": {},
        })
    );
}
