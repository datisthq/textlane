use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use textlane::{Allocator, File, Pipeline, Tree};

// The direct writer must write what md.toHtml and html.stringify write, byte for byte,
// for every Markdown input of the conformance suite it does not give up on
#[test]
fn writes_what_the_hast_path_writes() -> Result<(), String> {
    let inputs = read_inputs()?;
    let mut written = 0;
    let mut total = 0;
    let mut failures = vec![];
    for gfm in [false, true] {
        for dangerous in [false, true] {
            let mut pipeline = Pipeline::new();
            pipeline
                .use_value(&json!("md.parse"), None)
                .map_err(|e| e.message)?;
            if gfm {
                pipeline
                    .use_value(&json!("md.gfm"), None)
                    .map_err(|e| e.message)?;
            }
            let processor = pipeline.freeze().map_err(|e| e.message)?;
            let to_html = textlane::md::ToHtml {
                allow_dangerous_html: dangerous,
                ..Default::default()
            };
            let stringify = textlane::html::Stringify {
                allow_dangerous_html: dangerous,
                ..Default::default()
            };
            for input in &inputs {
                let alloc = Allocator::new();
                let Ok(Tree::Mdast(tree)) =
                    processor.parse(input, &mut File::new(), &alloc)
                else {
                    continue;
                };
                total += 1;
                let Some(direct) = textlane::md::write_html(&tree, &to_html, dangerous)
                else {
                    continue;
                };
                written += 1;
                let hast = textlane::md::convert_mdast(&tree, &alloc, &to_html);
                let expected = textlane::html::stringify_hast(&hast, &stringify)
                    .map_err(|e| e.message)?;
                if direct != expected {
                    failures.push(format!(
                        "gfm {gfm}, dangerous {dangerous}: {input:?}\n  direct: {direct:?}\n  hast:   {expected:?}"
                    ));
                }
            }
        }
    }
    println!("the writer wrote {written} of {total} documents");
    if !failures.is_empty() {
        return Err(format!(
            "{} differ:\n{}",
            failures.len(),
            failures.join("\n")
        ));
    }
    // A writer that gives up on everything would pass the comparison above
    if written * 2 < total {
        return Err(format!(
            "the writer wrote only {written} of {total} documents"
        ));
    }
    Ok(())
}

fn read_inputs() -> Result<Vec<String>, String> {
    let mut inputs = vec![];
    let directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../conformance/tests/md");
    for file in list_files(&directory)? {
        let text =
            fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        let groups: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        for group in groups.as_array().into_iter().flatten() {
            for test in group["tests"].as_array().into_iter().flatten() {
                if let Some(input) = test["input"].as_str() {
                    inputs.push(input.to_string());
                }
            }
        }
    }
    Ok(inputs)
}

fn list_files(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = vec![];
    let entries =
        fs::read_dir(directory).map_err(|e| format!("{}: {e}", directory.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}
