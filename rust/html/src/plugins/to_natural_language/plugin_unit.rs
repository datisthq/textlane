use super::*;

#[test]
fn fails_without_a_natural_language_parser() {
    let mut context = Attach::default();
    let result = ToNaturalLanguage.attach(&mut context);
    assert!(
        matches!(result, Err(error) if error.code == "missing-natural-language-parser")
    );
}
