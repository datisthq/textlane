use super::parse_expression;
use textlane_core::{ScriptError, ScriptExpression};

#[test]
fn reads_where_an_expression_ends() -> Result<(), ScriptError> {
    assert_eq!(
        parse_expression("a // b")?,
        ScriptExpression {
            end: 1,
            object: None
        }
    );
    Ok(())
}

#[test]
fn lists_the_properties_of_an_object() -> Result<(), ScriptError> {
    let expression = parse_expression("({...a, b})")?;
    let kinds = expression.object.map(|properties| {
        properties
            .into_iter()
            .map(|(kind, _)| kind)
            .collect::<Vec<_>>()
    });
    assert_eq!(
        kinds,
        Some(vec!["SpreadElement".to_string(), "Property".to_string()])
    );
    Ok(())
}

#[test]
fn rejects_more_than_one_expression() {
    assert!(parse_expression("a)(b").is_err());
    assert!(parse_expression("a b").is_err());
}

#[test]
fn reports_errors_the_source_may_still_fix_at_its_end() {
    // A cover grammar error is only found once all is read
    let error = parse_expression("({c=d})").err();
    assert_eq!(error.map(|error| error.end), Some(7));
    let error = parse_expression("(}").err();
    assert_eq!(error.map(|error| error.end), Some(2));
}

#[test]
fn survives_deep_nesting() {
    let deep = format!("{}a{}", "(".repeat(1_000), ")".repeat(1_000));
    assert!(parse_expression(&deep).is_ok());
    let deeper = format!("{}a{}", "(".repeat(100_000), ")".repeat(100_000));
    assert!(parse_expression(&deeper).is_ok());
    let separated = format!("{}a", "new\u{a0}".repeat(20_000));
    let _ = parse_expression(&separated);
    let unchecked = format!("{}a", "!".repeat(300_000));
    assert!(parse_expression(&unchecked).is_err());
}
