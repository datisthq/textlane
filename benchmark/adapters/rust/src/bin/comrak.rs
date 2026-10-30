use std::error::Error;

use comrak::{Options, markdown_to_html};
use textlane_benchmark::{is_gfm, run_adapter};

fn main() -> Result<(), Box<dyn Error>> {
    let mut options = Options::default();
    options.render.r#unsafe = true;
    if is_gfm() {
        options.extension.table = true;
        options.extension.strikethrough = true;
        options.extension.tasklist = true;
        options.extension.autolink = true;
        options.extension.footnotes = true;
    }
    run_adapter(|markdown| Ok(markdown_to_html(markdown, &options)))
}
