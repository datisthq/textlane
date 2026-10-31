use textlane_core::hast::Node;
use textlane_core::{Allocator, File};

use super::read_attribute;
use crate::actions::html::parse::parse_html;
use crate::models::options::Parse;

fn read_first(alloc: &Allocator, html: &str, name: &str) -> Option<String> {
    let fragment = Parse {
        fragment: true,
        ..Parse::default()
    };
    let root = parse_html(html, &fragment, &mut File::new(), alloc);
    let Node::Root(root) = root else {
        return None;
    };
    let Some(Node::Element(element)) = root.children.first() else {
        return None;
    };
    read_attribute(element, name).map(|value| value.into_owned())
}

#[test]
fn reads_by_html_name() {
    let alloc = Allocator::new();
    let html = r#"<pre class="a  b" data-title="x.js" data-line-numbers hidden></pre>"#;
    assert_eq!(read_first(&alloc, html, "class").as_deref(), Some("a b"));
    assert_eq!(
        read_first(&alloc, html, "data-title").as_deref(),
        Some("x.js")
    );
    assert_eq!(
        read_first(&alloc, html, "data-line-numbers").as_deref(),
        Some("")
    );
    assert_eq!(read_first(&alloc, html, "hidden").as_deref(), Some(""));
    assert_eq!(read_first(&alloc, html, "id"), None);
}
