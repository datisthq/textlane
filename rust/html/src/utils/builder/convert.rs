use std::collections::HashMap;

use textlane_core::hast::{
    Comment, Doctype, Element, Node, Property, PropertyItem, PropertyValue, Root, Text,
};
use textlane_core::{Allocator, ArenaVec, Point, Position};

use crate::helpers::property::{Item, Value, find_property, parse_property_value};
use crate::helpers::schema::{HTML, Info, SVG};
use crate::models::namespace::Namespace;
use crate::utils::builder::dom::{DocumentMode, Dom, Location, NodeData, NodeId};

/// Turn the parser's tree into hast, as hast-util-from-parse5 does with hastscript.
pub(crate) fn convert_dom<'a>(
    dom: &Dom,
    root: NodeId,
    source_length: usize,
    alloc: &'a Allocator,
) -> Node<'a> {
    let mut converter = Converter {
        dom,
        alloc,
        source_length,
        location_seen: false,
    };
    converter.convert(root)
}

struct Converter<'d, 'a> {
    dom: &'d Dom,
    alloc: &'a Allocator,
    source_length: usize,
    // hast-util-from-parse5 gives a root a position once any node before it had one
    location_seen: bool,
}

struct Frame<'a> {
    node: NodeId,
    // A template's children are its content's
    parent_of_children: NodeId,
    cursor: Option<NodeId>,
    children: ArenaVec<'a, Node<'a>>,
}

impl<'a> Converter<'_, 'a> {
    fn convert(&mut self, root: NodeId) -> Node<'a> {
        let mut frames = vec![self.open(root)];
        loop {
            let Some(top) = frames.last_mut() else {
                return Node::Root(self.create_root(None));
            };
            if let Some(child) = top.cursor {
                top.cursor = self.dom.nodes[child].next;
                match &self.dom.nodes[child].data {
                    NodeData::Element(_)
                    | NodeData::Fragment
                    | NodeData::Document { .. } => {
                        frames.push(self.open(child));
                    }
                    _ => {
                        let node = self.convert_leaf(child);
                        top.children.push(node);
                    }
                }
                continue;
            }
            let Some(frame) = frames.pop() else {
                continue;
            };
            let node = self.close(frame);
            match frames.last_mut() {
                Some(parent) => parent.children.push(node),
                None => return node,
            }
        }
    }

    fn open(&self, node: NodeId) -> Frame<'a> {
        let parent_of_children = self.dom.template_content(node).unwrap_or(node);
        Frame {
            node,
            parent_of_children,
            cursor: self.dom.nodes[parent_of_children].first_child,
            children: ArenaVec::new_in(self.alloc),
        }
    }

    fn close(&mut self, frame: Frame<'a>) -> Node<'a> {
        let node = &self.dom.nodes[frame.node];
        match &node.data {
            NodeData::Document { quirks } => {
                let mut root = self.create_root(Some(frame.children));
                root.quirks_mode = Some(*quirks != DocumentMode::NoQuirks);
                Node::Root(root)
            }
            NodeData::Element(element) => {
                let schema: &'static [Info] = if element.namespace == Namespace::Svg {
                    &SVG
                } else {
                    &HTML
                };
                let (tag_name, selector) = parse_selector(&element.tag_name);
                let tag_name = if element.namespace == Namespace::Svg {
                    adjust_svg_tag_name(&tag_name)
                } else {
                    tag_name.to_lowercase()
                };
                let properties = self.create_properties(schema, selector, element);
                let is_template = frame.parent_of_children != frame.node;
                let (children, content) = if is_template {
                    // upstream converts the content after the template's own position
                    self.location_seen = self.location_seen || node.location.is_some();
                    let mut content = self.create_root(Some(frame.children));
                    if let Some(location) = node.location
                        && let (Some(start_tag), Some(end_tag)) =
                            (location.start_tag, location.end_tag)
                    {
                        content.position = Some(Position::span(
                            Point::new(start_tag.end, 0),
                            Point::new(end_tag.start, 0),
                        ));
                    }
                    (
                        ArenaVec::new_in(self.alloc),
                        Some(&*self.alloc.alloc(content)),
                    )
                } else {
                    (frame.children, None)
                };
                let position = node.location.map(|location| {
                    self.location_seen = true;
                    let mut end = location.end;
                    if location.end_tag.is_none()
                        && let Some(Position::Span { end: tail, .. }) =
                            children.last().and_then(Node::position)
                    {
                        end = tail.offset();
                    }
                    Position::span(Point::new(location.start, 0), Point::new(end, 0))
                });
                Node::Element(Element {
                    tag_name: self.alloc.alloc_str(&tag_name),
                    properties,
                    children,
                    position,
                    content,
                })
            }
            _ => Node::Root(self.create_root(Some(frame.children))),
        }
    }

    fn create_root(&self, children: Option<ArenaVec<'a, Node<'a>>>) -> Root<'a> {
        Root {
            children: children.unwrap_or_else(|| ArenaVec::new_in(self.alloc)),
            position: self.location_seen.then(|| {
                Position::span(Point::new(0, 0), Point::new(self.source_length, 0))
            }),
            quirks_mode: Some(false),
        }
    }

    fn convert_leaf(&mut self, node: NodeId) -> Node<'a> {
        let dom_node = &self.dom.nodes[node];
        let position = dom_node
            .location
            .map(|location| self.create_position(location));
        match &dom_node.data {
            NodeData::Text(value) => Node::Text(Text {
                value: self.alloc.alloc_str(value),
                position,
            }),
            NodeData::Comment(value) => Node::Comment(Comment {
                value: self.alloc.alloc_str(value),
                position,
            }),
            _ => Node::Doctype(Doctype { position }),
        }
    }

    fn create_position(&mut self, location: Location) -> Position<'a> {
        self.location_seen = true;
        Position::span(Point::new(location.start, 0), Point::new(location.end, 0))
    }

    fn create_properties(
        &self,
        schema: &'static [Info],
        selector: Selector,
        element: &crate::utils::builder::dom::Element,
    ) -> ArenaVec<'a, Property<'a>> {
        let mut values = Values::default();
        if let Some(id) = selector.id {
            values.set("id".to_string(), Value::String(id));
        }
        if !selector.class_names.is_empty() {
            let items = selector.class_names.into_iter().map(Item::String).collect();
            values.set("className".to_string(), Value::List(items));
        }
        for attribute in &element.attrs {
            let name = match &attribute.prefix {
                Some(prefix) if !prefix.is_empty() => {
                    format!("{prefix}:{}", attribute.name)
                }
                _ => attribute.name.clone(),
            };
            // hast-util-from-parse5 skips names on `Object.prototype`
            if name == "constructor" || name == "__proto__" {
                continue;
            }
            let found = find_property(schema, &name);
            let mut value = parse_property_value(&found, &attribute.value);
            let property = found.property.to_string();
            if property == "className"
                && let (Some(Value::List(old)), Value::List(new)) =
                    (values.get(&property), &mut value)
            {
                let mut joined = old.clone();
                joined.append(new);
                value = Value::List(joined);
            }
            values.set(property, value);
        }
        let values = values.entries;
        let mut properties = ArenaVec::with_capacity_in(values.len(), self.alloc);
        for (name, value) in values {
            properties.push(Property {
                name: self.alloc.alloc_str(&name),
                value: self.create_value(value),
            });
        }
        properties
    }

    fn create_value(&self, value: Value) -> PropertyValue<'a> {
        match value {
            Value::Boolean(value) => PropertyValue::Boolean(value),
            Value::Number(value) => PropertyValue::Number(value),
            Value::String(value) => PropertyValue::String(self.alloc.alloc_str(&value)),
            Value::List(items) => {
                let mut list = ArenaVec::with_capacity_in(items.len(), self.alloc);
                for item in items {
                    list.push(match item {
                        Item::Number(value) => PropertyItem::Number(value),
                        Item::String(value) => {
                            PropertyItem::String(self.alloc.alloc_str(&value))
                        }
                    });
                }
                PropertyValue::List(list)
            }
        }
    }
}

// Properties in insertion order; a later attribute with the same property replaces it
#[derive(Default)]
struct Values {
    entries: Vec<(String, Value)>,
    indexes: HashMap<String, usize>,
}

impl Values {
    fn get(&self, name: &str) -> Option<&Value> {
        self.indexes.get(name).map(|index| &self.entries[*index].1)
    }

    fn set(&mut self, name: String, value: Value) {
        match self.indexes.get(&name) {
            Some(index) => self.entries[*index].1 = value,
            None => {
                self.indexes.insert(name.clone(), self.entries.len());
                self.entries.push((name, value));
            }
        }
    }
}

#[derive(Default)]
struct Selector {
    id: Option<String>,
    class_names: Vec<String>,
}

// hastscript reads a tag name as a selector, so `a.b` is an `a` with class `b`
fn parse_selector(value: &str) -> (String, Selector) {
    let mut selector = Selector::default();
    let mut tag_name = None;
    let mut previous = None;
    let mut rest = value;
    while !rest.is_empty() {
        let end = rest.find(['#', '.']).unwrap_or(rest.len());
        let part = &rest[..end];
        if !part.is_empty() {
            match previous {
                None => tag_name = Some(part.to_string()),
                Some('#') => selector.id = Some(part.to_string()),
                Some(_) => selector.class_names.push(part.to_string()),
            }
        }
        if end < rest.len() {
            previous = rest[end..].chars().next();
            rest = &rest[end + 1..];
        } else {
            rest = "";
        }
    }
    (tag_name.unwrap_or_else(|| "div".to_string()), selector)
}

fn adjust_svg_tag_name(tag_name: &str) -> String {
    let lower = tag_name.to_lowercase();
    match SVG_TAG_NAMES.binary_search_by(|(each, _)| each.cmp(&lower.as_str())) {
        Ok(index) => SVG_TAG_NAMES[index].1.to_string(),
        Err(_) => lower,
    }
}

// hastscript's case-sensitive SVG tag names, by lowercase name
static SVG_TAG_NAMES: [(&str, &str); 39] = [
    ("altglyph", "altGlyph"),
    ("altglyphdef", "altGlyphDef"),
    ("altglyphitem", "altGlyphItem"),
    ("animatecolor", "animateColor"),
    ("animatemotion", "animateMotion"),
    ("animatetransform", "animateTransform"),
    ("clippath", "clipPath"),
    ("feblend", "feBlend"),
    ("fecolormatrix", "feColorMatrix"),
    ("fecomponenttransfer", "feComponentTransfer"),
    ("fecomposite", "feComposite"),
    ("feconvolvematrix", "feConvolveMatrix"),
    ("fediffuselighting", "feDiffuseLighting"),
    ("fedisplacementmap", "feDisplacementMap"),
    ("fedistantlight", "feDistantLight"),
    ("fedropshadow", "feDropShadow"),
    ("feflood", "feFlood"),
    ("fefunca", "feFuncA"),
    ("fefuncb", "feFuncB"),
    ("fefuncg", "feFuncG"),
    ("fefuncr", "feFuncR"),
    ("fegaussianblur", "feGaussianBlur"),
    ("feimage", "feImage"),
    ("femerge", "feMerge"),
    ("femergenode", "feMergeNode"),
    ("femorphology", "feMorphology"),
    ("feoffset", "feOffset"),
    ("fepointlight", "fePointLight"),
    ("fespecularlighting", "feSpecularLighting"),
    ("fespotlight", "feSpotLight"),
    ("fetile", "feTile"),
    ("feturbulence", "feTurbulence"),
    ("foreignobject", "foreignObject"),
    ("glyphref", "glyphRef"),
    ("lineargradient", "linearGradient"),
    ("radialgradient", "radialGradient"),
    ("solidcolor", "solidColor"),
    ("textarea", "textArea"),
    ("textpath", "textPath"),
];
