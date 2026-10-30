use std::env;
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
    let mut stringify = dangerous;
    // What any HTML plugin forces, as a no-op: a non-default option takes the hast path
    if env::var_os("TEXTLANE_HAST").is_some() {
        stringify["quote"] = json!("\"");
    }
    pipeline.use_value(&json!("html.stringify"), Some(stringify))?;
    let processor = pipeline.freeze()?;
    run_adapter(|markdown| Ok(processor.process(markdown)?.value))
}
