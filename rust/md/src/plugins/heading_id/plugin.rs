use std::sync::LazyLock;

use regex::Regex;
use textlane_core::mdast::{Heading, Node};
use textlane_core::{
    Allocator, ArenaVec, Attach, Data, Error, File, Plugin, Transform, Tree, TreeKind,
};

use std::collections::BTreeMap;

use crate::models::options::{HeadingId, MarkerValue};
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
        // longest first, so `[!toc]` is never read as a shorter marker it ends with
        let mut markers: Vec<(&String, &BTreeMap<String, MarkerValue>)> =
            self.markers.iter().collect();
        markers.sort_by_key(|(key, _)| std::cmp::Reverse(key.len()));
        let mut stack = root.children_mut().into_iter().collect::<Vec<_>>();
        while let Some(children) = stack.pop() {
            for child in children.iter_mut() {
                if let Node::Heading(heading) = child {
                    read_heading(heading, self.attributes, &markers, alloc);
                }
                if let Some(children) = child.children_mut() {
                    stack.push(children);
                }
            }
        }
        Ok(Tree::Mdast(root))
    }
}

type Found = Vec<(String, Option<String>)>;

// The last text loses, from its end, any markers and one attribute list (or `{#id}` without
// `attributes`), in any order; the list is read once, which keeps a run of braces linear
fn read_heading<'a>(
    heading: &mut Heading<'a>,
    attributes: bool,
    markers: &[(&String, &BTreeMap<String, MarkerValue>)],
    alloc: &'a Allocator,
) {
    let Some(Node::Text(text)) = heading.children.last_mut() else {
        return;
    };
    let mut value = text.value;
    let mut found: Found = Vec::new();
    let mut listed = false;
    loop {
        let trimmed = value.trim_end_matches(' ');
        if let Some((key, fields)) =
            markers.iter().find(|(key, _)| trimmed.ends_with(*key))
        {
            value = trimmed[..trimmed.len() - key.len()].trim_end();
            for (name, field) in fields.iter().filter(|(name, _)| is_allowed(name)) {
                let field = match field {
                    MarkerValue::Flag(false) => continue,
                    MarkerValue::Flag(true) => None,
                    MarkerValue::Text(text) => Some(text.clone()),
                };
                found.push((name.clone(), field));
            }
            continue;
        }
        if listed {
            break;
        }
        listed = true;
        let rest = match attributes {
            true => read_attribute_list(trimmed, &mut found),
            false => read_custom_id(trimmed, &mut found),
        };
        match rest {
            Some(rest) => value = rest,
            None => break,
        }
    }
    if found.is_empty() {
        return;
    }
    text.value = value;
    heading.data = Some(alloc.alloc(write_attributes(heading.data, &found, alloc)));
}

// Its last ` {…}` when those braces hold a valid attribute list ending the text; only the
// last are tried, so a quoted value holding ` {` leaves the list unread
fn read_attribute_list<'v>(trimmed: &'v str, found: &mut Found) -> Option<&'v str> {
    let brace = trimmed.rfind(" {")?;
    let bytes = trimmed.as_bytes();
    let (list, end) = scan_attributes(bytes, trimmed, brace + 1, bytes.len(), false)?;
    if end != bytes.len() {
        return None;
    }
    found.extend(
        list.into_iter()
            .filter(|(key, _)| is_allowed(key))
            .map(|(key, value)| (key, Some(value))),
    );
    Some(&trimmed[..brace])
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

// remark-heading-id's ` {#id}`, which becomes `data.id` and `data.hProperties.id`
fn read_custom_id<'v>(trimmed: &'v str, found: &mut Found) -> Option<&'v str> {
    let captures = CUSTOM_ID.as_ref()?.captures(trimmed)?;
    let (whole, id) = (captures.get(0)?, captures.get(1)?);
    found.push(("id".to_owned(), Some(id.as_str().to_owned())));
    Some(&trimmed[..whole.start()])
}

// `data.hProperties` gets each attribute, classes as the `className` list, and `data.id`
// the id, beside whatever data the heading already had
fn write_attributes<'a>(
    data: Option<&'a Data<'a>>,
    list: &[(String, Option<String>)],
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
        // the first of a name wins: the one nearest the heading's end
        if properties.iter().any(|(existing, _)| existing == name) {
            continue;
        }
        let Some(value) = value else {
            properties.push((alloc.alloc_str(name), Data::Boolean(true)));
            continue;
        };
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
