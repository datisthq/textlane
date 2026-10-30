use serde_json::json;

use super::*;
use crate::utils::yaml::number::format_number;

fn error_of(source: &str) -> Option<Error> {
    parse_yaml(source).err()
}

#[test]
fn reads_core_schema_scalars() -> Result<(), Error> {
    let value =
        parse_yaml("a: 1\nb: 1.0\nc: 0x1f\nd: .inf\ne: ~\nf: True\ng: 2024-01-02")?;
    let expected = json!({"a": 1, "b": 1, "c": 31, "d": null, "e": null, "f": true, "g": "2024-01-02"});
    assert_eq!(value, expected);
    Ok(())
}

#[test]
fn formats_numbers_as_javascript() {
    let cases = [
        (1e21, "1e+21"),
        (1e20, "100000000000000000000"),
        (1.5, "1.5"),
        (1e-7, "1e-7"),
        (0.000001, "0.000001"),
        (-0.0, "0"),
        (6.02e23, "6.02e+23"),
        (f64::NEG_INFINITY, "-Infinity"),
    ];
    for (number, text) in cases {
        assert_eq!(format_number(number), text);
    }
}

#[test]
fn stops_exponential_aliases() {
    let mut source = String::from("a0: &a0 [x, x, x, x, x, x, x, x, x, x]\n");
    for level in 1..30 {
        let previous = format!("*a{}", level - 1);
        let items = vec![previous; 10].join(", ");
        source.push_str(&format!("a{level}: &a{level} [{items}]\n"));
    }
    let error = error_of(&source);
    assert!(
        matches!(error, Some(error) if error.code == "invalid-frontmatter"
        && error.message.contains("resource exhaustion"))
    );
}

#[test]
fn stops_deep_nesting_without_overflowing() {
    let deep = format!("{}{}", "[".repeat(100_000), "]".repeat(100_000));
    assert!(
        matches!(error_of(&deep), Some(error) if error.code == "invalid-frontmatter")
    );
    let block = (0..2_000).fold(String::new(), |text, level| {
        format!("{text}{}- \n", "  ".repeat(level))
    });
    assert!(
        matches!(error_of(&block), Some(error) if error.code == "invalid-frontmatter")
    );
}

#[test]
fn stops_aliases_that_nest_too_deep() {
    let half = format!("{}x{}", "[".repeat(300), "]".repeat(300));
    let source = format!("a: &a {half}\nb: {}*a{}", "[".repeat(300), "]".repeat(300));
    assert!(
        matches!(error_of(&source), Some(error) if error.code == "invalid-frontmatter")
    );
}

#[test]
fn fails_on_circular_aliases() {
    assert!(
        matches!(error_of("&a [*a]"), Some(error) if error.message == "Circular alias")
    );
}

#[test]
fn fails_on_collection_keys() {
    assert!(
        matches!(error_of("? [a]\n: b"), Some(error) if error.code == "invalid-frontmatter")
    );
}

#[test]
fn fails_on_more_than_one_document() {
    assert!(error_of("a: 1\n...\n---\nb: 2").is_some());
}

#[test]
fn caps_aliases_of_empty_collections() {
    let mut source = String::from("a0: &a0 []\n");
    for level in 1..30 {
        let previous = format!("*a{}", level - 1);
        let items = [previous.as_str(); 10].join(", ");
        source.push_str(&format!("a{level}: &a{level} [{items}]\n"));
    }
    let started = std::time::Instant::now();
    let error = error_of(&source).map(|error| error.code);
    assert_eq!(error.as_deref(), Some("invalid-frontmatter"));
    assert!(started.elapsed().as_secs() < 5);
}
