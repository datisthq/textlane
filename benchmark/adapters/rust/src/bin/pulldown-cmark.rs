use std::error::Error;

use pulldown_cmark::{Options, Parser, html};
use textlane_benchmark::{is_gfm, run_adapter};

fn main() -> Result<(), Box<dyn Error>> {
    let options = if is_gfm() {
        Options::ENABLE_GFM
            | Options::ENABLE_TABLES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_FOOTNOTES
    } else {
        Options::empty()
    };
    run_adapter(|markdown| {
        let mut output = String::with_capacity(markdown.len() * 3 / 2);
        html::push_html(&mut output, Parser::new_ext(markdown, options));
        Ok(output)
    })
}
