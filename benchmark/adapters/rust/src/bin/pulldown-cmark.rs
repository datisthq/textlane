use std::error::Error;

use pulldown_cmark::{Options, Parser, html};
use textlane_benchmark::run_adapter;

fn main() -> Result<(), Box<dyn Error>> {
    run_adapter(|markdown| {
        let mut output = String::with_capacity(markdown.len() * 3 / 2);
        html::push_html(&mut output, Parser::new_ext(markdown, Options::empty()));
        Ok(output)
    })
}
