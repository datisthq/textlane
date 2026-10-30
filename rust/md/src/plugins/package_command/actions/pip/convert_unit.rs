use super::*;

#[test]
fn adds_packages_for_uv() {
    assert_eq!(convert_pip("pip install a b", PipManager::Uv), "uv add a b");
}

#[test]
fn keeps_a_line_without_an_equivalent() {
    let line = "pip install --user a";
    assert_eq!(convert_pip(line, PipManager::Poetry), line);
}

#[test]
fn keeps_pip_lines_for_pip() {
    assert_eq!(convert_pip("python -m a", PipManager::Pip), "python -m a");
    assert_eq!(convert_pip("uvx ruff", PipManager::Pip), "pipx run ruff");
}

#[test]
fn reads_a_missing_flag_value_as_no_equivalent() {
    assert_eq!(
        convert_pip("pip install -r", PipManager::Uv),
        "pip install -r"
    );
}
