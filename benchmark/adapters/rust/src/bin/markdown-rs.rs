use std::error::Error;

use markdown::{CompileOptions, Options, to_html_with_options};
use textlane_benchmark::run_adapter;

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options {
        compile: CompileOptions {
            allow_dangerous_html: true,
            allow_dangerous_protocol: true,
            ..CompileOptions::default()
        },
        ..Options::default()
    };
    run_adapter(|markdown| Ok(to_html_with_options(markdown, &options).map_err(|e| e.to_string())?))
}
