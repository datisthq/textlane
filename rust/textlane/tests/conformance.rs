use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use textlane::run_case;

// "<area>/<file>#<group>" to the open issue whose fix unblocks it in this runner alone;
// what the crate does not implement yet is conformance/pending.json, shared by every runner
const KNOWN_DIVERGENCES: &[(&str, &str)] = &[];

#[test]
fn conformance() -> Result<(), String> {
    let pending = read_pending()?;
    let mut report = Report::default();
    for file in list_files(&suite_path().join("tests"))? {
        let id = case_id(&file)?;
        let text = fs::read_to_string(&file).map_err(|e| format!("{id}: {e}"))?;
        let groups: Value =
            serde_json::from_str(&text).map_err(|e| format!("{id}: {e}"))?;
        for group in groups.as_array().into_iter().flatten() {
            let description = group["description"].as_str().unwrap_or_default();
            let group_id = format!("{id}#{description}");
            if !KNOWN_DIVERGENCES
                .iter()
                .any(|(skipped, _)| *skipped == group_id)
            {
                run_group(group, &id, &group_id, &pending, &mut report);
            }
        }
    }
    report.finish()
}

#[derive(Default)]
struct Report {
    passed: usize,
    failures: Vec<String>,
    matched_entries: Vec<String>,
    failing_entries: Vec<String>,
}

impl Report {
    // A pending entry whose cases all pass is stale and fails the run
    fn finish(mut self) -> Result<(), String> {
        for entry in &self.matched_entries {
            if !self.failing_entries.contains(entry) {
                let message = format!(
                    "{entry} passes now; remove it from conformance/pending.json"
                );
                self.failures.push(message);
            }
        }
        if self.passed == 0 && self.failing_entries.is_empty() {
            return Err("the suite ran no tests".into());
        }
        if self.failures.is_empty() {
            Ok(())
        } else {
            Err(self.failures.join("\n"))
        }
    }
}

fn run_group(
    group: &Value,
    file_id: &str,
    group_id: &str,
    pending: &[String],
    report: &mut Report,
) {
    for test in group["tests"].as_array().into_iter().flatten() {
        let name = test["description"].as_str().unwrap_or_default();
        let test_id = format!("{group_id}#{name}");
        let entry = (pending.iter())
            .find(|entry| matches_entry(entry, file_id, group_id, &test_id));
        if let Some(entry) = entry.filter(|entry| !report.matched_entries.contains(entry))
        {
            report.matched_entries.push(entry.clone());
        }
        let mut case = json!({ "pipeline": group["pipeline"] });
        for key in ["input", "tree", "operations"] {
            if let Some(value) = test.get(key) {
                case[key] = value.clone();
            }
        }
        let mut actual = run_case(&case);
        if group["positions"] == Value::Bool(false) {
            strip_positions(&mut actual);
        }
        if matches_expected(&actual, &test["expected"]) {
            report.passed += 1;
        } else if let Some(entry) = entry {
            report.failing_entries.push(entry.clone());
        } else {
            let expected = &test["expected"];
            let message =
                format!("{test_id}\n  expected {expected}\n  actual   {actual}");
            report.failures.push(message);
        }
    }
}

// A group with `positions: false` compares trees without them
fn strip_positions(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("position");
            map.values_mut().for_each(strip_positions);
        }
        Value::Array(items) => items.iter_mut().for_each(strip_positions),
        _ => {}
    }
}

// A run may report several outcomes; the one `expected` names is compared, and an unexpected
// error always fails. An expected error matches on its code, its message only as a substring
fn matches_expected(actual: &Value, expected: &Value) -> bool {
    let Some((key, want)) = expected.as_object().and_then(|map| map.iter().next()) else {
        return false;
    };
    match (key.as_str(), actual.get("error")) {
        ("error", Some(got)) => {
            let message = got["message"].as_str().unwrap_or_default();
            want["code"] == got["code"]
                && want["message"]
                    .as_str()
                    .is_none_or(|part| message.contains(part))
        }
        (_, Some(_)) | ("error", None) => false,
        _ => actual.get(key) == Some(want),
    }
}

// An entry names a file, a group, one test as `<group>#<test>`, or an area as `<area>/*`
fn matches_entry(entry: &str, file_id: &str, group_id: &str, test_id: &str) -> bool {
    match entry.strip_suffix('*') {
        Some(prefix) => file_id.starts_with(prefix),
        None => entry == file_id || entry == group_id || entry == test_id,
    }
}

fn read_pending() -> Result<Vec<String>, String> {
    let path = suite_path().join("pending.json");
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let pending: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let entries = pending.as_object().into_iter().flatten();
    Ok(entries.map(|(entry, _)| entry.clone()).collect())
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
