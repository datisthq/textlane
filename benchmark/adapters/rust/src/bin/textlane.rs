use std::error::Error;

use serde_json::json;
use textlane::Pipeline;
use textlane_benchmark::{is_gfm, run_adapter};

fn main() -> Result<(), Box<dyn Error>> {
    let mut pipeline = Pipeline::new();
    let dangerous = json!({ "allowDangerousHtml": true });
    pipeline.use_value(&json!("md.parse"), None)?;
    if is_gfm() {
        pipeline.use_value(&json!("md.gfm"), None)?;
    }
    pipeline.use_value(&json!("md.toHtml"), Some(dangerous.clone()))?;
    pipeline.use_value(&json!("html.stringify"), Some(dangerous))?;
    let processor = pipeline.freeze()?;
    run_adapter(|markdown| Ok(processor.process(markdown)?.value))
}
