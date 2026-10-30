use serde_json::{Value, json};
use textlane_core::{Error, File};

use crate::models::pipeline::{Api, Pipeline};
use crate::models::run::{Outcome, Pause, Progress};

fn pipeline(plugins: Value) -> Result<Pipeline, Error> {
    let mut pipeline = Pipeline::new();
    pipeline.use_value(&plugins, None)?;
    Ok(pipeline)
}

fn paused(progress: Progress) -> Result<Pause, Error> {
    match progress {
        Progress::Paused(pause) => Ok(pause),
        Progress::Finished(_) => Err(Error::new("test", "expected a pause")),
    }
}

fn text(progress: Progress) -> Result<String, Error> {
    match progress {
        Progress::Finished(finished) => match finished.output {
            Outcome::Text(text) => Ok(text),
            Outcome::Tree(tree) => Ok(tree.to_string()),
        },
        Progress::Paused(_) => Err(Error::new("test", "expected the end")),
    }
}

#[test]
fn pauses_before_host_stages_and_resumes_with_their_tree() -> Result<(), Error> {
    let mut pipeline = pipeline(json!([
        "md.parse",
        { "host": 7 },
        "md.toHtml",
        { "host": 8 },
        "html.stringify",
    ]))?;
    let pause = paused(pipeline.start_process("*a*", File::new(), Api::Async)?)?;
    assert_eq!(pause.hosts(), &[7]);
    assert_eq!(pause.tree()["type"], "root");
    assert_eq!(pause.tree()["position"]["end"]["offset"], 3);
    let mut tree = pause.tree().clone();
    tree["children"][0]["children"][0]["type"] = json!("strong");
    let pause = paused(pause.resume(&tree)?)?;
    assert_eq!(pause.hosts(), &[8]);
    assert_eq!(pause.tree()["children"][0]["tagName"], "p");
    let tree = pause.tree().clone();
    assert_eq!(text(pause.resume(&tree)?)?, "<p><strong>a</strong></p>");
    Ok(())
}

#[test]
fn batches_consecutive_host_stages() -> Result<(), Error> {
    let mut pipeline =
        pipeline(json!(["md.parse", { "host": 1 }, { "host": 2 }, "md.stringify"]))?;
    let pause = paused(pipeline.start_process("a", File::new(), Api::Sync)?)?;
    assert_eq!(pause.hosts(), &[1, 2]);
    let tree = pause.tree().clone();
    assert_eq!(text(pause.resume(&tree)?)?, "a\n");
    Ok(())
}

#[test]
fn skips_disabled_host_plugins() -> Result<(), Error> {
    let mut pipeline =
        pipeline(json!(["md.parse", { "host": 1 }, { "host": 2 }, "md.stringify"]))?;
    pipeline.use_value(&json!({ "host": 1 }), Some(json!(false)))?;
    assert_eq!(pipeline.hosts(), vec![2]);
    let pause = paused(pipeline.start_process("a", File::new(), Api::Async)?)?;
    assert_eq!(pause.hosts(), &[2]);
    Ok(())
}

#[test]
fn keeps_messages_host_stages_report() -> Result<(), Error> {
    let mut pipeline = pipeline(json!(["md.parse", { "host": 1 }, "md.stringify"]))?;
    let mut pause = paused(pipeline.start_process("a", File::new(), Api::Async)?)?;
    pause.file_mut().warn("from a host", Some("rule"));
    let tree = pause.tree().clone();
    match pause.resume(&tree)? {
        Progress::Finished(finished) => assert_eq!(finished.file.messages.len(), 1),
        Progress::Paused(_) => return Err(Error::new("test", "expected the end")),
    }
    Ok(())
}

#[test]
fn runs_a_tree_up_to_a_host_stage() -> Result<(), Error> {
    let mut pipeline = pipeline(json!(["md.toHtml", { "host": 1 }]))?;
    let tree = json!({ "type": "root", "children": [{ "type": "text", "value": "a" }] });
    let pause = paused(pipeline.start_run(&tree, File::new(), Api::Async)?)?;
    assert_eq!(
        pause.tree()["children"][0],
        json!({ "type": "text", "value": "a" })
    );
    let tree = pause.tree().clone();
    match pause.resume(&tree)? {
        Progress::Finished(finished) => match finished.output {
            Outcome::Tree(tree) => assert_eq!(tree["type"], "root"),
            Outcome::Text(_) => return Err(Error::new("test", "expected a tree")),
        },
        Progress::Paused(_) => return Err(Error::new("test", "expected the end")),
    }
    Ok(())
}

#[test]
fn fails_to_process_host_stages_without_a_binding() -> Result<(), Error> {
    let mut pipeline = pipeline(json!(["md.parse", { "host": 3 }, "md.stringify"]))?;
    let error = pipeline.process("a", &mut File::new(), Api::Async).err();
    assert_eq!(
        error.map(|error| error.code),
        Some("missing-host".to_string())
    );
    Ok(())
}

#[test]
fn still_checks_the_kinds_around_host_stages() -> Result<(), Error> {
    let mut pipeline = pipeline(json!(["md.parse", { "host": 3 }, "html.stringify"]))?;
    let error = pipeline.freeze().err();
    assert_eq!(
        error.map(|error| error.code),
        Some("invalid-stage".to_string())
    );
    Ok(())
}
