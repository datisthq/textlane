use std::error::Error;

use comrak::{Options, markdown_to_html};
use textlane_benchmark::run_adapter;

fn main() -> Result<(), Box<dyn Error>> {
    let mut options = Options::default();
    options.render.r#unsafe = true;
    run_adapter(|markdown| Ok(markdown_to_html(markdown, &options)))
}
