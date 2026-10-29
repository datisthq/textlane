use super::parse_module;
use textlane_core::ScriptError;

#[test]
fn lists_statements_and_imports() -> Result<(), ScriptError> {
    let statements = parse_module("import a, {b as c} from 'd'\nexport const e = 1")?;
    let kinds: Vec<&str> = statements
        .iter()
        .map(|statement| statement.kind.as_str())
        .collect();
    assert_eq!(kinds, ["ImportDeclaration", "ExportNamedDeclaration"]);
    assert_eq!(statements[0].imports, ["a", "c"]);
    Ok(())
}

#[test]
fn rejects_what_acorn_rejects_early() {
    assert!(parse_module("export {a}").is_err());
    assert!(parse_module("var a\nimport a from 'b'").is_err());
}
