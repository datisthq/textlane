use std::sync::LazyLock;

use regex::Regex;
use textlane_core::mdast::{Heading, Node};
use textlane_core::{
    Allocator, ArenaVec, Attach, Data, Error, File, Plugin, Transform, Tree, TreeKind,
};

use crate::models::options::HeadingId;
use crate::plugins::heading_id::actions::data::copy::copy_data;
use crate::utils::onepass::directive::scan_attributes;

static CUSTOM_ID: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?s) \{#(.+?)\}$").ok());

impl Plugin for HeadingId {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(self.clone()));
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
                    match self.attributes {
                        true => read_attributes(heading, alloc),
                        false => read_custom_id(heading, alloc),
                    }
                }
                if let Some(children) = child.children_mut() {
                    stack.push(children);
                }
            }
        }
        Ok(Tree::Mdast(root))
    }
}

// The last text, its trailing spaces dropped, loses its last ` {…}` when those braces hold a
// valid attribute list ending the text; only the last are tried, which keeps a run of braces
// linear, so a quoted value holding ` {` leaves the list unread
fn read_attributes<'a>(heading: &mut Heading<'a>, alloc: &'a Allocator) {
    let Some(Node::Text(text)) = heading.children.last_mut() else {
        return;
    };
    let trimmed = text.value.trim_end_matches(' ');
    let Some(brace) = trimmed.rfind(" {") else {
        return;
    };
    let bytes = trimmed.as_bytes();
    let Some((list, end)) =
        scan_attributes(bytes, trimmed, brace + 1, bytes.len(), false)
    else {
        return;
    };
    if end != bytes.len() {
        return;
    }
    let list: Vec<(String, String)> = list
        .into_iter()
        .filter(|(key, _)| is_allowed(key))
        .collect();
    text.value = &trimmed[..brace];
    heading.data = Some(alloc.alloc(write_attributes(heading.data, &list, alloc)));
}

// Event handlers never come from Markdown, nor names an attribute cannot have; the rest is
// for `html.sanitize` to judge on untrusted input
fn is_allowed(key: &str) -> bool {
    !key.is_empty()
        && !key
            .get(..2)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("on"))
        && key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_:.".contains(&byte))
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
    write_attributes(data, &[("id".to_owned(), id.to_owned())], alloc)
}

// `data.hProperties` gets each attribute, classes as the `className` list, and `data.id`
// the id, beside whatever data the heading already had
fn write_attributes<'a>(
    data: Option<&'a Data<'a>>,
    list: &[(String, String)],
    alloc: &'a Allocator,
) -> Data<'a> {
    let names: Vec<&str> = list
        .iter()
        .map(|(key, _)| {
            if key == "class" {
                "className"
            } else {
                key.as_str()
            }
        })
        .collect();
    let mut entries = ArenaVec::new_in(alloc);
    let mut properties = ArenaVec::new_in(alloc);
    if let Some(Data::Object(existing)) = data {
        for (key, value) in existing.iter() {
            match (*key, value) {
                ("hProperties", Data::Object(fields)) => {
                    for (name, value) in
                        fields.iter().filter(|(name, _)| !names.contains(name))
                    {
                        properties.push((*name, copy_data(value, alloc)));
                    }
                }
                ("id" | "hProperties", _) => {}
                (key, value) => entries.push((key, copy_data(value, alloc))),
            }
        }
    }
    let mut id = None;
    for ((key, value), name) in list.iter().zip(&names) {
        let value = alloc.alloc_str(value);
        let written = match key.as_str() {
            "class" => {
                let mut classes = ArenaVec::new_in(alloc);
                classes.extend(value.split_whitespace().map(Data::String));
                Data::Array(classes)
            }
            "id" => {
                id = Some(&*value);
                Data::String(value)
            }
            _ => Data::String(value),
        };
        properties.push((alloc.alloc_str(name), written));
    }
    entries.push(("hProperties", Data::Object(properties)));
    if let Some(id) = id {
        entries.push(("id", Data::String(id)));
    }
    Data::Object(entries)
}
