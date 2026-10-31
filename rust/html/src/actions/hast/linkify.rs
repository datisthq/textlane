use linkify::{LinkFinder, LinkKind};
use textlane_core::hast::{Element, Node, Property, PropertyValue};
use textlane_core::{Allocator, ArenaVec};

use crate::helpers::prose::{create_text, replace_prose};
use crate::models::options::Linkify;

/// Turn bare URLs and email addresses in text into links: an `http`, `https` or `ftp` one
/// as written, a `www.` one under `https://`, an email under `mailto:`; any other scheme,
/// and a bare domain, which may name a file, stay text.
pub fn link_urls<'a>(node: &mut Node<'a>, options: &Linkify, alloc: &'a Allocator) {
    let mut finder = LinkFinder::new();
    finder.url_must_have_scheme(false);
    replace_prose(
        node,
        &options.ignore,
        &options.ignore_attributes,
        alloc,
        |text, out| {
            let mut last = 0;
            let mut changed = false;
            for link in finder.links(text) {
                let url = link.as_str();
                let href = match link.kind() {
                    LinkKind::Email => format!("mailto:{url}"),
                    _ if has_safe_scheme(url) => url.to_owned(),
                    _ if url.starts_with("www.") => format!("https://{url}"),
                    _ => continue,
                };
                if link.start() > last {
                    out.push(create_text(&text[last..link.start()], alloc));
                }
                out.push(create_link(&href, url, alloc));
                last = link.end();
                changed = true;
            }
            if changed && last < text.len() {
                out.push(create_text(&text[last..], alloc));
            }
            changed
        },
    );
}

// Any other scheme stays text: `javascript://%0aalert(1)` in prose must not become a link
fn has_safe_scheme(url: &str) -> bool {
    url.split_once("://").is_some_and(|(scheme, _)| {
        ["http", "https", "ftp"]
            .iter()
            .any(|safe| scheme.eq_ignore_ascii_case(safe))
    })
}

fn create_link<'a>(href: &str, text: &str, alloc: &'a Allocator) -> Node<'a> {
    let mut properties = ArenaVec::with_capacity_in(1, alloc);
    properties.push(Property {
        name: "href",
        value: PropertyValue::String(alloc.alloc_str(href)),
    });
    let mut children = ArenaVec::with_capacity_in(1, alloc);
    children.push(create_text(text, alloc));
    Node::Element(Element {
        tag_name: "a",
        properties,
        children,
        position: None,
        data: None,
        content: None,
    })
}

#[cfg(test)]
#[path = "linkify_unit.rs"]
mod unit;
