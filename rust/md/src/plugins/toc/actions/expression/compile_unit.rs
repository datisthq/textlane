use super::compile_expression;
use textlane_core::Error;

fn matches(source: &str, value: &str) -> Result<bool, Error> {
    Ok(compile_expression(source)?.is_match(value))
}

#[test]
fn matches_the_default_heading_whole_and_case_insensitively() -> Result<(), Error> {
    let source = "(table[ -]of[ -])?contents?|toc";
    assert!(matches(source, "Table of Contents")?);
    assert!(matches(source, "TOC")?);
    assert!(matches(source, "table-of-content")?);
    assert!(!matches(source, "Contents here")?);
    assert!(!matches(source, "my toc")?);
    Ok(())
}

#[test]
fn reads_classes_as_javascript_does() -> Result<(), Error> {
    assert!(matches(r"\d+", "123")?);
    assert!(!matches(r"\d", "٣")?);
    assert!(!matches(r"\w", "é")?);
    assert!(matches(r"a\sb", "a\u{FEFF}b")?);
    assert!(!matches(r"a\sb", "a\u{85}b")?);
    assert!(!matches(r".", "\r")?);
    assert!(matches(r"[]", "").is_ok_and(|matched| !matched));
    assert!(matches(r"a[^]", "a\n")?);
    assert!(matches(r"[\d-z]", "-")?);
    assert!(matches(r"[a-c]+", "abc")?);
    assert!(matches(r"[]a]", "]a").is_ok_and(|matched| !matched));
    Ok(())
}

#[test]
fn reads_escapes_and_braces_as_javascript_does() -> Result<(), Error> {
    assert!(matches(r"A\x42", "ab")?);
    assert!(matches(r"😀", "😀")?);
    assert!(matches(r"a{", "a{")?);
    assert!(matches(r"a{2}", "aa")?);
    assert!(matches(r"\/\.", "/.")?);
    assert!(matches(r"\a", "a")?);
    assert!(matches(r"a\bb", "a b").is_ok_and(|matched| !matched));
    assert!(matches(r"a\b b", "a b")?);
    Ok(())
}

#[test]
fn fails_on_what_rust_cannot_match() {
    assert!(compile_expression(r"(a)\1").is_err());
    assert!(compile_expression(r"a(?=b)").is_err());
    assert!(compile_expression(r"(?<!a)b").is_err());
    assert!(compile_expression(r"[b-a]").is_err());
    assert!(compile_expression(r"(").is_err());
}

#[test]
fn closes_the_group_the_source_is_put_in() -> Result<(), Error> {
    assert!(matches("a)|(b", "a and more")?);
    Ok(())
}
