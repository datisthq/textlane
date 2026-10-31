use textlane_core::hast::{Node, Property, PropertyValue};
use textlane_core::{Allocator, ArenaVec, Slugger};

use crate::models::options::Slug;

// Headings nested in each other share text, and slugging each one's whole text is
// quadratic; past this much more text than the headings hold, later ones are cut short
const SLACK: usize = 64 * 1024;

/// Give headings without an `id` one made from their text, as rehype-slug does.
pub fn slug_hast<'a>(node: &mut Node<'a>, options: &Slug, alloc: &'a Allocator) {
    let (text, ranges) = collect_heading_text(node);
    let mut budget = text.len() + SLACK;
    let mut ranges = ranges.into_iter();
    let mut slugger = Slugger::default();
    if options.reserve_existing {
        reserve_ids(node, &options.prefix, &mut slugger);
    }
    let mut stack: Vec<&mut Node<'a>> = vec![node];
    while let Some(node) = stack.pop() {
        let children = match node {
            Node::Root(root) => &mut root.children,
            Node::Element(element) => {
                if is_heading(element.tag_name) {
                    let (start, end) = ranges.next().unwrap_or_default();
                    if !has_id(&element.properties)
                        && is_ranked(element.tag_name, options)
                    {
                        let mut end = end.min(start + budget);
                        while !text.is_char_boundary(end) {
                            end -= 1;
                        }
                        budget -= end - start;
                        let id = format!(
                            "{}{}",
                            options.prefix,
                            slugger.slug(&text[start..end])
                        );
                        element.properties.push(Property {
                            name: "id",
                            value: PropertyValue::String(alloc.alloc_str(&id)),
                        });
                    }
                }
                &mut element.children
            }
            _ => continue,
        };
        // Visited in document order, which decides which repeat gets `-1`
        stack.extend(children.iter_mut().rev());
    }
}

// The text inside headings, as hast-util-to-string reads it, and each heading's range of
// it in document order
fn collect_heading_text(node: &Node) -> (String, Vec<(usize, usize)>) {
    let mut collector = Collector::default();
    collector.open(node);
    while let Some((children, next, heading)) = collector.frames.last_mut() {
        if let Some(child) = children.get(*next) {
            *next += 1;
            match child {
                Node::Text(value) if collector.open_headings > 0 => {
                    collector.text.push_str(value.value);
                }
                Node::Element(_) | Node::Root(_) => collector.open(child),
                _ => {}
            }
            continue;
        }
        if let Some(index) = *heading {
            collector.open_headings -= 1;
            collector.ranges[index].1 = collector.text.len();
        }
        collector.frames.pop();
    }
    (collector.text, collector.ranges)
}

#[derive(Default)]
struct Collector<'n, 'a> {
    text: String,
    ranges: Vec<(usize, usize)>,
    open_headings: usize,
    // Each open node's children, where the walk is in them, and its range if a heading
    frames: Vec<(&'n [Node<'a>], usize, Option<usize>)>,
}

impl<'n, 'a> Collector<'n, 'a> {
    fn open(&mut self, node: &'n Node<'a>) {
        let (children, heading) = match node {
            Node::Element(element) => {
                (&element.children[..], is_heading(element.tag_name))
            }
            Node::Root(root) => (&root.children[..], false),
            _ => return,
        };
        let index = heading.then(|| {
            self.ranges.push((self.text.len(), self.text.len()));
            self.open_headings += 1;
            self.ranges.len() - 1
        });
        self.frames.push((children, 0, index));
    }
}

// Only an id carrying the prefix can collide with a generated one
fn reserve_ids(node: &Node, prefix: &str, slugger: &mut Slugger) {
    let mut stack: Vec<&Node> = vec![node];
    while let Some(node) = stack.pop() {
        let children = match node {
            Node::Root(root) => &root.children,
            Node::Element(element) => {
                let id = element.properties.iter().rev().find(|p| p.name == "id");
                if let Some(Property {
                    value: PropertyValue::String(id),
                    ..
                }) = id
                    && let Some(rest) = id.strip_prefix(prefix)
                {
                    slugger.reserve(rest);
                }
                &element.children
            }
            _ => continue,
        };
        stack.extend(children.iter());
    }
}

fn is_ranked(tag_name: &str, options: &Slug) -> bool {
    let rank = tag_name.as_bytes().get(1).map_or(0, |digit| digit - b'0');
    options
        .ranks
        .as_ref()
        .is_none_or(|ranks| ranks.contains(&rank))
}

fn is_heading(tag_name: &str) -> bool {
    matches!(tag_name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
}

// rehype-slug checks `!properties.id`, so an empty or false id counts as none
fn has_id(properties: &ArenaVec<'_, Property<'_>>) -> bool {
    properties
        .iter()
        .rev()
        .find(|property| property.name == "id")
        .is_some_and(|property| match &property.value {
            PropertyValue::Null | PropertyValue::Boolean(false) => false,
            PropertyValue::String(value) => !value.is_empty(),
            PropertyValue::Number(value) => *value != 0.0 && !value.is_nan(),
            _ => true,
        })
}
