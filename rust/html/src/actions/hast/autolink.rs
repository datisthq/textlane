use serde_json::Value;
use textlane_core::hast::{
    Element, Node, Property, PropertyItem, PropertyValue, deserialize_hast,
};
use textlane_core::{Allocator, ArenaVec, Error};

use crate::helpers::value::{
    find_value, is_truthy, read_json_value, set_property, write_value,
};
use crate::models::options::{AutolinkBehavior, AutolinkOptions, TagNames};

/// Link each ranked heading with an `id` to itself, as rehype-autolink-headings does; a
/// linked heading's own content is not walked.
pub(crate) fn link_headings<'a>(
    node: &mut Node<'a>,
    options: &AutolinkOptions,
    alloc: &'a Allocator,
) -> Result<(), Error> {
    let linker = Linker { options, alloc };
    // The tree itself has no parent, so `before` and `after` leave it be, as upstream does
    let mut stack = match node {
        Node::Root(root) => vec![&mut root.children],
        Node::Element(element) if linker.matches(element) => {
            linker.set_heading_properties(element);
            if !matches!(
                options.behavior,
                AutolinkBehavior::Before | AutolinkBehavior::After
            ) {
                linker.link_inside(element)?;
            }
            return Ok(());
        }
        Node::Element(element) => vec![&mut element.children],
        _ => return Ok(()),
    };
    let around = matches!(
        options.behavior,
        AutolinkBehavior::Before | AutolinkBehavior::After
    );
    while let Some(children) = stack.pop() {
        // What `around` inserted is skipped, as upstream's visitor returns past it
        let mut inserted = Vec::new();
        if around
            && children
                .iter()
                .any(|child| matches!(child, Node::Element(element) if linker.matches(element)))
        {
            let mut old =
                std::mem::replace(children, ArenaVec::with_capacity_in(children.len() + 2, alloc));
            for child in old.drain(..) {
                match child {
                    Node::Element(mut element) if linker.matches(&element) => {
                        linker.set_heading_properties(&mut element);
                        let first = children.len();
                        linker.link_around(children, element)?;
                        inserted.extend(first..children.len());
                    }
                    child => children.push(child),
                }
            }
        }
        for (index, child) in children.iter_mut().enumerate() {
            let Node::Element(element) = child else {
                continue;
            };
            if inserted.contains(&index) {
                continue;
            }
            if !linker.matches(element) {
                stack.push(&mut element.children);
            } else if !around {
                linker.set_heading_properties(element);
                linker.link_inside(element)?;
            }
        }
    }
    Ok(())
}

struct Linker<'o, 'a> {
    options: &'o AutolinkOptions,
    alloc: &'a Allocator,
}

impl<'a> Linker<'_, 'a> {
    // hast-util-heading-rank, an `id` JavaScript reads as true, and `test`'s tag names
    fn matches(&self, element: &Element<'a>) -> bool {
        is_heading(element.tag_name)
            && is_truthy(find_value(element, "id"))
            && match &self.options.test {
                None => true,
                Some(TagNames::One(name)) => element.tag_name == name,
                Some(TagNames::Many(names)) => {
                    names.iter().any(|name| name == element.tag_name)
                }
            }
    }

    fn set_heading_properties(&self, element: &mut Element<'a>) {
        for (name, value) in self.options.heading_properties.iter().flatten() {
            set_property(
                element,
                self.alloc.alloc_str(name),
                read_json_value(value, self.alloc),
            );
        }
    }

    // `prepend`, `append` and `wrap`, which change the heading itself
    fn link_inside(&self, element: &mut Element<'a>) -> Result<(), Error> {
        match self.options.behavior {
            AutolinkBehavior::Wrap => {
                let mut children = std::mem::replace(
                    &mut element.children,
                    ArenaVec::with_capacity_in(1, self.alloc),
                );
                if let Some(content) = &self.options.content {
                    for node in read_nodes(content, self.alloc)? {
                        children.push(node);
                    }
                }
                let link = self.create(element, children);
                element.children.push(link);
            }
            behavior => {
                let link = self.create(element, self.content()?);
                if behavior == AutolinkBehavior::Prepend {
                    element.children.insert(0, link);
                } else {
                    element.children.push(link);
                }
            }
        }
        Ok(())
    }

    // `before` and `after`: the link beside the heading, both in `group` when it is an element
    fn link_around(
        &self,
        children: &mut ArenaVec<'a, Node<'a>>,
        element: Element<'a>,
    ) -> Result<(), Error> {
        let link = self.create(&element, self.content()?);
        let heading = Node::Element(element);
        let pair = match self.options.behavior {
            AutolinkBehavior::Before => [link, heading],
            _ => [heading, link],
        };
        let group = match &self.options.group {
            Some(group) => match deserialize_hast(group, self.alloc)? {
                Node::Element(group) => Some(group),
                _ => None,
            },
            None => None,
        };
        match group {
            Some(mut group) => {
                group.children = ArenaVec::with_capacity_in(2, self.alloc);
                group.children.extend(pair);
                children.push(Node::Element(group));
            }
            None => children.extend(pair),
        }
        Ok(())
    }

    fn content(&self) -> Result<ArenaVec<'a, Node<'a>>, Error> {
        match &self.options.content {
            Some(content) => read_nodes(content, self.alloc),
            None => {
                let mut class = ArenaVec::with_capacity_in(2, self.alloc);
                class.extend([
                    PropertyItem::String("icon"),
                    PropertyItem::String("icon-link"),
                ]);
                let mut properties = ArenaVec::with_capacity_in(1, self.alloc);
                properties.push(Property {
                    name: "className",
                    value: PropertyValue::List(class),
                });
                let mut nodes = ArenaVec::with_capacity_in(1, self.alloc);
                nodes.push(Node::Element(Element {
                    tag_name: "span",
                    properties,
                    children: ArenaVec::new_in(self.alloc),
                    position: None,
                    data: None,
                    content: None,
                }));
                Ok(nodes)
            }
        }
    }

    // `{...properties, href: '#' + id}`: the options' properties, then `href`
    fn create(
        &self,
        heading: &Element<'a>,
        children: ArenaVec<'a, Node<'a>>,
    ) -> Node<'a> {
        let mut link = Element {
            tag_name: "a",
            properties: ArenaVec::new_in(self.alloc),
            children,
            position: None,
            data: None,
            content: None,
        };
        match &self.options.properties {
            Some(properties) => {
                for (name, value) in properties {
                    set_property(
                        &mut link,
                        self.alloc.alloc_str(name),
                        read_json_value(value, self.alloc),
                    );
                }
            }
            None if matches!(
                self.options.behavior,
                AutolinkBehavior::Prepend | AutolinkBehavior::Append
            ) =>
            {
                set_property(&mut link, "ariaHidden", PropertyValue::String("true"));
                set_property(&mut link, "tabIndex", PropertyValue::Number(-1.0));
            }
            None => {}
        }
        let id = find_value(heading, "id")
            .map(write_value)
            .unwrap_or_default();
        let href = self.alloc.alloc_str(&format!("#{id}"));
        set_property(&mut link, "href", PropertyValue::String(href));
        Node::Element(link)
    }
}

fn read_nodes<'a>(
    value: &Value,
    alloc: &'a Allocator,
) -> Result<ArenaVec<'a, Node<'a>>, Error> {
    let nodes: Vec<&Value> = match value {
        Value::Array(nodes) => nodes.iter().collect(),
        node => vec![node],
    };
    let mut children = ArenaVec::with_capacity_in(nodes.len(), alloc);
    for node in nodes {
        children.push(deserialize_hast(node, alloc)?);
    }
    Ok(children)
}

fn is_heading(tag_name: &str) -> bool {
    matches!(tag_name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
}
