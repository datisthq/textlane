use super::*;

#[test]
fn fails_without_a_prose_parser() {
    let mut context = Attach::default();
    let result = ToProse.attach(&mut context);
    assert!(matches!(result, Err(error) if error.code == "missing-prose-parser"));
}
