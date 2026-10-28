use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use textlane::run_case;

// "<area>/<file>#<group>" to the open issue whose fix unblocks it; nothing else skips
const KNOWN_DIVERGENCES: &[(&str, &str)] = &[];

#[test]
fn conformance() -> Result<(), String> {
    let mut failures = Vec::new();
    let mut passed = 0;
    for file in list_files(&suite_path().join("tests"))? {
        let id = case_id(&file)?;
        let text = fs::read_to_string(&file).map_err(|e| format!("{id}: {e}"))?;
        let groups: Value =
            serde_json::from_str(&text).map_err(|e| format!("{id}: {e}"))?;
        for group in groups.as_array().into_iter().flatten() {
            let description = group["description"].as_str().unwrap_or_default();
            let group_id = format!("{id}#{description}");
            if KNOWN_DIVERGENCES
                .iter()
                .any(|(skipped, _)| *skipped == group_id)
            {
                continue;
            }
            for test in group["tests"].as_array().into_iter().flatten() {
                let mut case = json!({ "pipeline": group["pipeline"] });
                for key in ["input", "tree"] {
                    if let Some(value) = test.get(key) {
                        case[key] = value.clone();
                    }
                }
                let actual = run_case(&case);
                if matches_expected(&actual, &test["expected"]) {
                    passed += 1;
                } else {
                    let name = test["description"].as_str().unwrap_or_default();
                    failures.push(format!(
                        "{group_id} / {name}\n  expected {}\n  actual   {actual}",
                        test["expected"]
                    ));
                }
            }
        }
    }
    if passed == 0 {
        return Err("the suite ran no tests".into());
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

// An expected error matches on its code, and on its message only as a substring
fn matches_expected(actual: &Value, expected: &Value) -> bool {
    match (expected.get("error"), actual.get("error")) {
        (Some(want), Some(got)) => {
            let message = got["message"].as_str().unwrap_or_default();
            want["code"] == got["code"]
                && want["message"]
                    .as_str()
                    .is_none_or(|part| message.contains(part))
        }
        _ => actual == expected,
    }
}

fn suite_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../conformance")
}

fn list_files(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let entries =
        fs::read_dir(directory).map_err(|e| format!("{}: {e}", directory.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            files.extend(list_files(&path)?);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn case_id(file: &Path) -> Result<String, String> {
    let tests = suite_path().join("tests");
    let relative = file.strip_prefix(&tests).map_err(|e| e.to_string())?;
    Ok(relative
        .with_extension("")
        .to_string_lossy()
        .replace('\\', "/"))
}
