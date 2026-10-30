use textlane_core::hast::{Element, Node, Raw};
use textlane_core::{Allocator, ArenaVec, Error};

use super::stringify_hast;
use crate::models::options::Stringify;

#[test]
fn escapes_raw_in_script_unless_dangerous_html_is_allowed() -> Result<(), Error> {
    let alloc = Allocator::new();
    let mut children = ArenaVec::new_in(&alloc);
    children.push(Node::Raw(Raw {
        value: "</script><img onerror=x>",
        position: None,
        data: None,
    }));
    let script = Node::Element(Element {
        tag_name: "script",
        properties: ArenaVec::new_in(&alloc),
        children,
        position: None,
        content: None,
        data: None,
    });
    assert_eq!(
        stringify_hast(&script, &Stringify::default())?,
        "<script>&#x3C;/script>&#x3C;img onerror=x></script>"
    );
    let options = Stringify {
        allow_dangerous_html: true,
        ..Default::default()
    };
    assert_eq!(
        stringify_hast(&script, &options)?,
        "<script></script><img onerror=x></script>"
    );
    Ok(())
}
