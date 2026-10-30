use std::sync::LazyLock;

use regex::Regex;
use textlane_core::mdast::{Heading, Node};
use textlane_core::{
    Allocator, ArenaVec, Attach, Data, Error, File, Plugin, Transform, Tree, TreeKind,
};

use crate::plugins::heading_id::actions::data::copy::copy_data;

/// `md.headingId`: `{#id}` at the end of a heading, after a space, as its id, as
/// remark-heading-id reads it; headings without one keep none.
#[derive(Clone, Debug, Default)]
pub struct HeadingId;

static CUSTOM_ID: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?s) \{#(.+?)\}$").ok());

impl Plugin for HeadingId {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(HeadingId));
        Ok(())
    }
}

impl Transform for HeadingId {
    fn input(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn output(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn run<'a>(
        &self,
        tree: Tree<'a>,
        _file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        let Tree::Mdast(mut root) = tree else {
            return Err(Error::new("invalid-stage", "`md.headingId` takes mdast"));
        };
        let mut stack = root.children_mut().into_iter().collect::<Vec<_>>();
        while let Some(children) = stack.pop() {
            for child in children.iter_mut() {
                if let Node::Heading(heading) = child {
                    read_custom_id(heading, alloc);
                }
                if let Some(children) = child.children_mut() {
                    stack.push(children);
                }
            }
        }
        Ok(Tree::Mdast(root))
    }
}

// The last text, its trailing spaces dropped, loses ` {#id}`, which becomes `data.id` and
// `data.hProperties.id`
fn read_custom_id<'a>(heading: &mut Heading<'a>, alloc: &'a Allocator) {
    let Some(Node::Text(text)) = heading.children.last_mut() else {
        return;
    };
    let trimmed = text.value.trim_end_matches(' ');
    let Some(captures) = CUSTOM_ID
        .as_ref()
        .and_then(|custom| custom.captures(trimmed))
    else {
        return;
    };
    let (Some(whole), Some(id)) = (captures.get(0), captures.get(1)) else {
        return;
    };
    let id = alloc.alloc_str(id.as_str());
    text.value = &trimmed[..whole.start()];
    heading.data = Some(alloc.alloc(write_id(heading.data, id, alloc)));
}

fn write_id<'a>(
    data: Option<&'a Data<'a>>,
    id: &'a str,
    alloc: &'a Allocator,
) -> Data<'a> {
    let mut entries = ArenaVec::new_in(alloc);
    let mut properties = ArenaVec::new_in(alloc);
    if let Some(Data::Object(existing)) = data {
        for (key, value) in existing.iter() {
            match (*key, value) {
                ("hProperties", Data::Object(fields)) => {
                    for (name, value) in fields.iter().filter(|(name, _)| *name != "id") {
                        properties.push((*name, copy_data(value, alloc)));
                    }
                }
                ("id" | "hProperties", _) => {}
                (key, value) => entries.push((key, copy_data(value, alloc))),
            }
        }
    }
    properties.push(("id", Data::String(id)));
    entries.push(("hProperties", Data::Object(properties)));
    entries.push(("id", Data::String(id)));
    Data::Object(entries)
}
