use linkify::{LinkFinder, LinkKind};
use textlane_core::hast::{Element, Node, Property, PropertyValue};
use textlane_core::{Allocator, ArenaVec};

use crate::helpers::prose::{create_text, replace_prose};
use crate::models::options::Linkify;

/// Turn bare URLs and email addresses in text into links: one with `://` as written, a
/// `www.` one under `https://`, an email under `mailto:`; a bare domain stays text, as it
/// may name a file.
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
                    _ if url.contains("://") => url.to_owned(),
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
