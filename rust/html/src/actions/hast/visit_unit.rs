use textlane_core::hast::{Element, Node, Root};
use textlane_core::{Allocator, ArenaVec, File};

use super::visit_elements;
use crate::actions::html::parse::parse_html;
use crate::models::options::Parse;
use crate::models::visit::Visit;

fn list_visits(html: &str, skip: &str, stop: &str) -> Vec<String> {
    let alloc = Allocator::new();
    let fragment = Parse {
        fragment: true,
        ..Parse::default()
    };
    let mut root = parse_html(html, &fragment, &mut File::new(), &alloc);
    let mut seen = vec![];
    visit_elements(&mut root, |element, ancestors| {
        seen.push(format!("{}>{}", ancestors.join(">"), element.tag_name));
        match element.tag_name {
            name if name == skip => Visit::Skip,
            name if name == stop => Visit::Stop,
            _ => Visit::Continue,
        }
    });
    seen
}

#[test]
fn visits_in_document_order_with_ancestors() {
    let html = "<div><p><em>a</em></p><pre><code>b</code></pre></div><hr>";
    assert_eq!(
        list_visits(html, "", ""),
        [
            ">div",
            "div>p",
            "div>p>em",
            "div>pre",
            "div>pre>code",
            ">hr"
        ]
    );
}

#[test]
fn skips_children_and_stops() {
    let html = "<div><p><em>a</em></p><pre><code>b</code></pre></div><hr>";
    assert_eq!(list_visits(html, "p", "pre"), [">div", "div>p", "div>pre"]);
}

#[test]
fn walks_deep_trees_without_recursion() {
    let alloc = Allocator::new();
    let mut node = Node::Root(Root {
        children: ArenaVec::new_in(&alloc),
        position: None,
        data: None,
        quirks_mode: None,
    });
    for _ in 0..100_000 {
        let mut children = ArenaVec::new_in(&alloc);
        children.push(node);
        node = Node::Element(Element {
            tag_name: "div",
            properties: ArenaVec::new_in(&alloc),
            children,
            position: None,
            data: None,
            content: None,
        });
    }
    let mut count = 0;
    visit_elements(&mut node, |_, _| {
        count += 1;
        Visit::Continue
    });
    assert_eq!(count, 100_000);
}
