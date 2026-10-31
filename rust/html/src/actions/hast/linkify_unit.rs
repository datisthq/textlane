use textlane_core::{Allocator, Error, File};

use super::link_urls;
use crate::actions::hast::stringify::stringify_hast;
use crate::actions::html::parse::parse_html;
use crate::models::options::{Linkify, Parse, Stringify};

fn link(input: &str) -> Result<String, Error> {
    let alloc = Allocator::new();
    let parse = Parse {
        fragment: true,
        ..Default::default()
    };
    let mut node = parse_html(input, &parse, &mut File::new(), &alloc);
    link_urls(&mut node, &Linkify::default(), &alloc);
    stringify_hast(&node, &Stringify::default())
}

#[test]
fn links_urls_www_and_emails() -> Result<(), Error> {
    assert_eq!(
        link("<p>See https://a.dev/x, www.b.dev or c@d.dev.</p>")?,
        "<p>See <a href=\"https://a.dev/x\">https://a.dev/x</a>, \
         <a href=\"https://www.b.dev\">www.b.dev</a> or \
         <a href=\"mailto:c@d.dev\">c@d.dev</a>.</p>"
    );
    Ok(())
}

#[test]
fn leaves_bare_domains_links_and_code_alone() -> Result<(), Error> {
    let input =
        "<p>index.html</p><a href=\"/\">https://a.dev</a><code>https://b.dev</code>";
    assert_eq!(link(input)?, input);
    Ok(())
}
