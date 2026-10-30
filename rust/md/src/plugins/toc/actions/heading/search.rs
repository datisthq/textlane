use regex::Regex;
use textlane_core::mdast::Node;
use textlane_core::{Allocator, ArenaVec, Data, Slugger};

use crate::models::options::ParentTest;
use crate::plugins::toc::actions::node::copy::{CopyMode, copy_nodes};
use crate::plugins::toc::actions::parent::check::check_parent;
use crate::plugins::toc::actions::text::collect::collect_text;

/// What `search_headings` looks for.
pub(crate) struct SearchSettings<'s> {
    pub(crate) heading: &'s Regex,
    pub(crate) skip: Option<&'s Regex>,
    pub(crate) parents: Option<&'s ParentTest>,
    pub(crate) min_depth: u8,
    pub(crate) max_depth: u8,
}

/// One heading the table lists.
pub(crate) struct Entry<'a> {
    pub(crate) depth: u8,
    pub(crate) children: ArenaVec<'a, Node<'a>>,
    pub(crate) id: String,
}

/// Where the table goes: after the child `index` of the root names, up to `end_index`.
pub(crate) struct Search<'a> {
    pub(crate) index: Option<usize>,
    pub(crate) end_index: usize,
    pub(crate) map: Vec<Entry<'a>>,
}

/// mdast-util-toc's `search`: every heading slugged in document order, those in `parents`
/// after the closing heading collected.
pub(crate) fn search_headings<'a>(
    root: &Node<'a>,
    settings: &SearchSettings<'_>,
    alloc: &'a Allocator,
) -> Search<'a> {
    let max = root.children().map_or(0, |children| children.len());
    let mut slugger = Slugger::default();
    let mut map = Vec::new();
    // Zero is JavaScript's falsy: an opening is stored one past its index, and a closing
    // heading found at index 0 does not count as found
    let mut index = 0;
    let mut end_index = 0;
    let mut opening_depth = None;
    let mut stack = Vec::new();
    if let Some(children) = root.children() {
        stack.push((root, children.iter().enumerate()));
    }
    while let Some((parent, children)) = stack.last_mut() {
        let parent: &Node<'a> = parent;
        let Some((position, node)) = children.next() else {
            stack.pop();
            continue;
        };
        if let Node::Heading(heading) = node {
            let value = collect_text(&heading.children);
            let slug = match read_id(heading.data) {
                Some(id) => slugger.slug(id),
                None => slugger.slug(&value),
            };
            let in_parents = match settings.parents {
                Some(test) => check_parent(test, parent),
                None => std::ptr::eq(parent, root),
            };
            if in_parents {
                if index == 0 && settings.heading.is_match(&value) {
                    index = position + 1;
                    opening_depth = Some(heading.depth);
                } else {
                    if opening_depth
                        .is_some_and(|depth| end_index == 0 && heading.depth <= depth)
                    {
                        end_index = position;
                    }
                    if end_index != 0
                        && (settings.min_depth == 0
                            || heading.depth >= settings.min_depth)
                        && (settings.max_depth == 0
                            || heading.depth <= settings.max_depth)
                        && !settings.skip.is_some_and(|skip| skip.is_match(&value))
                    {
                        map.push(Entry {
                            depth: heading.depth,
                            children: copy_nodes(
                                &heading.children,
                                CopyMode::Phrasing,
                                alloc,
                            ),
                            id: slug,
                        });
                    }
                }
            }
        }
        if let Some(children) = node.children() {
            stack.push((node, children.iter().enumerate()));
        }
    }
    Search {
        index: (index != 0).then_some(index),
        end_index: if end_index == 0 { max } else { end_index },
        map,
    }
}

// `data.hProperties.id` when truthy; one that is no string slugs as the empty string
fn read_id<'a>(data: Option<&'a Data<'a>>) -> Option<&'a str> {
    let Some(Data::Object(entries)) = data else {
        return None;
    };
    let (_, properties) = entries
        .iter()
        .rev()
        .find(|(key, _)| *key == "hProperties")?;
    let Data::Object(properties) = properties else {
        return None;
    };
    let (_, id) = properties.iter().rev().find(|(key, _)| *key == "id")?;
    match id {
        Data::Null | Data::Boolean(false) => None,
        Data::String("") => None,
        Data::Number(id) if *id == 0.0 || id.is_nan() => None,
        Data::String(id) => Some(id),
        _ => Some(""),
    }
}
