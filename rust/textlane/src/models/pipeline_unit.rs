use serde_json::json;
use textlane_core::{Error, File};
use textlane_md::ShiftHeading;

use crate::models::pipeline::{Api, Pipeline};

fn shift(shift: i32) -> ShiftHeading {
    ShiftHeading {
        shift,
        ..ShiftHeading::default()
    }
}

fn process(pipeline: &mut Pipeline, input: &str) -> Result<String, Error> {
    pipeline.process(input, &mut File::new(), Api::Sync)
}

#[test]
fn runs_a_native_plugin_in_first_use_order() -> Result<(), Error> {
    let mut pipeline = Pipeline::new();
    pipeline.use_value(&json!("md.parse"), None)?;
    pipeline.use_native("shift", shift(1))?;
    pipeline.use_value(&json!(["md.toHtml", "html.stringify"]), None)?;
    assert_eq!(process(&mut pipeline, "# a")?, "<h2>a</h2>");
    Ok(())
}

#[test]
fn replaces_a_native_plugin_used_again_in_its_place() -> Result<(), Error> {
    let mut pipeline = Pipeline::new();
    pipeline.use_value(&json!("md.parse"), None)?;
    pipeline.use_native("shift", shift(1))?;
    pipeline.use_value(&json!(["md.toHtml", "html.stringify"]), None)?;
    pipeline.use_native("shift", shift(2))?;
    assert_eq!(process(&mut pipeline, "# a")?, "<h3>a</h3>");
    Ok(())
}

#[test]
fn merges_a_named_plugin_used_again_beside_native_ones() -> Result<(), Error> {
    let mut pipeline = Pipeline::new();
    pipeline.use_value(&json!("md.parse"), None)?;
    pipeline.use_native("shift", shift(1))?;
    pipeline.use_value(&json!(["md.toHtml", "html.stringify"]), None)?;
    pipeline.use_value(&json!("html.stringify"), Some(json!({ "quote": "'" })))?;
    let html = process(&mut pipeline, "# [a](b)")?;
    assert_eq!(html, "<h2><a href='b'>a</a></h2>");
    Ok(())
}

#[test]
fn refuses_a_native_plugin_once_frozen() -> Result<(), Error> {
    let mut pipeline = Pipeline::new();
    pipeline.use_value(&json!("md.parse"), None)?;
    pipeline.freeze()?;
    let error = pipeline.use_native("shift", shift(1)).err();
    assert_eq!(
        error.map(|error| error.code),
        Some("frozen-processor".to_owned())
    );
    Ok(())
}
