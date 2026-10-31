use std::collections::BTreeMap;

use textlane_core::{Allocator, Error, File};

use super::rewrite_paths;
use crate::actions::hast::stringify::stringify_hast;
use crate::actions::html::parse::parse_html;
use crate::models::options::{Parse, PathRewrite, Stringify};

fn rewrite(input: &str, path: &str, options: &PathRewrite) -> Result<String, Error> {
    let alloc = Allocator::new();
    let parse = Parse {
        fragment: true,
        ..Default::default()
    };
    let mut node = parse_html(input, &parse, &mut File::new(), &alloc);
    rewrite_paths(&mut node, options, Some(path), &alloc);
    stringify_hast(&node, &Stringify::default())
}

#[test]
fn makes_relative_urls_root_relative_under_base() -> Result<(), Error> {
    let options = PathRewrite {
        root: Some("docs".into()),
        base: Some("/repo".into()),
        ..PathRewrite::default()
    };
    let input = "<a href=\"setup.html#flags\">a</a><img src=\"../img/a.png\" \
                 srcset=\"img/a.png 1x, https://x.dev/b.png 2x\"><a href=\"/x\">b</a>\
                 <a href=\"https://x.dev\">c</a><a href=\"#top\">d</a><a href=\"sub/\">e</a>";
    assert_eq!(
        rewrite(input, "docs/guides/build.html", &options)?,
        "<a href=\"/repo/guides/setup.html#flags\">a</a><img src=\"/repo/img/a.png\" \
         srcset=\"/repo/guides/img/a.png 1x, https://x.dev/b.png 2x\"><a href=\"/repo/x\">b</a>\
         <a href=\"https://x.dev\">c</a><a href=\"#top\">d</a><a href=\"/repo/guides/sub/\">e</a>"
    );
    Ok(())
}

#[test]
fn links_page_files_to_their_urls() -> Result<(), Error> {
    let options = PathRewrite {
        pages: BTreeMap::from([("guide.md".into(), "/guide/".into())]),
        ..PathRewrite::default()
    };
    let input = "<a href=\"guide.md#a\">a</a><img src=\"guide.md\">";
    assert_eq!(
        rewrite(input, "index.md", &options)?,
        "<a href=\"/guide/#a\">a</a><img src=\"/guide.md\">"
    );
    Ok(())
}
