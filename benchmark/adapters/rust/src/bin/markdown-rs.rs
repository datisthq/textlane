use std::error::Error;

use markdown::{CompileOptions, Options, to_html_with_options};
use textlane_benchmark::{is_gfm, run_adapter};

fn main() -> Result<(), Box<dyn Error>> {
    let (parse, compile) = if is_gfm() {
        (Options::gfm().parse, CompileOptions::gfm())
    } else {
        (Options::default().parse, CompileOptions::default())
    };
    let options = Options {
        parse,
        compile: CompileOptions {
            allow_dangerous_html: true,
            allow_dangerous_protocol: true,
            ..compile
        },
    };
    run_adapter(|markdown| Ok(to_html_with_options(markdown, &options).map_err(|e| e.to_string())?))
}
