use textlane_core::hast::{self, Node, Root, Unknown};
use textlane_core::{Allocator, ArenaVec, Error};

use super::raw_hast;
use crate::actions::hast::stringify::stringify_hast;
use crate::models::options::{Raw, Stringify};

fn create_root<'a>(alloc: &'a Allocator, values: &[&'a str]) -> Node<'a> {
    let mut children = ArenaVec::new_in(alloc);
    for value in values {
        children.push(Node::Raw(hast::Raw {
            value,
            position: None,
            data: None,
        }));
    }
    Node::Root(Root {
        children,
        position: None,
        data: None,
        quirks_mode: None,
    })
}

fn reparse(values: &[&str], options: &Raw) -> Result<String, Error> {
    let alloc = Allocator::new();
    let tree = raw_hast(&create_root(&alloc, values), options, &alloc)?;
    stringify_hast(&tree, &Stringify::default())
}

#[test]
fn escapes_disallowed_tags_with_tagfilter() -> Result<(), Error> {
    let options = Raw {
        tagfilter: true,
        ..Default::default()
    };
    assert_eq!(
        reparse(&["<script>x</SCRIPT><b>y</b><scripts>"], &options)?,
        "&#x3C;script>x&#x3C;/SCRIPT><b>y</b><scripts></scripts>"
    );
    Ok(())
}

// What hast-util-raw 9.1.0 on parse5 7.3.0 does where a lookahead hits the end of a chunk
#[test]
fn drops_a_lookahead_cut_by_the_end_of_a_raw_node() -> Result<(), Error> {
    let options = Raw::default();
    assert_eq!(reparse(&["<!-", "- x -->"], &options)?, "<!--- x ---->");
    assert_eq!(
        reparse(&["<title>a</ti", "tle>b"], &options)?,
        "<title>a&#x3C;/tle>b</title>"
    );
    assert_eq!(reparse(&["a &amp"], &options)?, "a ");
    Ok(())
}

#[test]
fn fails_on_unknown_nodes_not_passed_through() {
    let alloc = Allocator::new();
    let tree = Node::Unknown(Unknown {
        kind: "mdxjsEsm",
        fields: &[],
        children: None,
        position: None,
        data: None,
    });
    let error = raw_hast(&tree, &Raw::default(), &alloc).err();
    assert!(error.is_some_and(|error| error.code == "unknown-node"));
}

#[test]
fn passes_deeply_nested_nodes_through_without_recursing() -> Result<(), Error> {
    let alloc = Allocator::new();
    let mut node = create_root(&alloc, &["<i>x"]);
    for _ in 0..100_000 {
        let mut children = ArenaVec::new_in(&alloc);
        children.push(node);
        node = Node::Unknown(Unknown {
            kind: "custom",
            fields: &[],
            children: Some(children),
            position: None,
            data: None,
        });
    }
    let options = Raw {
        pass_through: vec!["custom".to_string()],
        ..Default::default()
    };
    let mut depth = 0;
    let mut current = raw_hast(&node, &options, &alloc)?;
    let innermost = loop {
        match current {
            Node::Unknown(Unknown {
                children: Some(mut children),
                ..
            }) if children.len() == 1 => {
                depth += 1;
                match children.pop() {
                    Some(child) => current = child,
                    None => break None,
                }
            }
            node => break Some(node),
        }
    };
    assert_eq!(depth, 100_000);
    assert!(matches!(innermost, Some(Node::Element(element)) if element.tag_name == "i"));
    Ok(())
}
