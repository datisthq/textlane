use katex::KatexContext;
use textlane_core::hast::{Element, Node, PropertyItem, PropertyValue};
use textlane_core::{Allocator, ArenaVec, File, Lines, Position, UnistPosition};

use crate::actions::attribute::read::read_attribute;
use crate::actions::math::typeset::{create_settings, typeset_math};
use crate::helpers::text::read_preformatted_text;
use crate::models::options::Katex;

/// Render each element of class `language-math`, `math-display` or `math-inline` with KaTeX
/// as rehype-katex does; a `<pre>` around a `language-math` `<code>` is replaced whole.
pub fn typeset_hast<'a>(
    node: &mut Node<'a>,
    options: &Katex,
    file: &mut File,
    alloc: &'a Allocator,
) {
    // Built at the first formula, as most documents have none
    let mut katex: Option<(KatexContext, katex::Settings)> = None;
    let mut stack: Vec<&mut Node<'a>> = vec![node];
    while let Some(node) = stack.pop() {
        let children = match node {
            Node::Root(root) => &mut root.children,
            Node::Element(element) => &mut element.children,
            _ => continue,
        };
        if !children
            .iter()
            .any(|child| read_math(child, options).is_some())
        {
            stack.extend(children.iter_mut());
            continue;
        }
        let old = std::mem::replace(children, ArenaVec::new_in(alloc));
        let mut rendered = Vec::new();
        for child in old {
            let Some(math) = read_math(&child, options) else {
                children.push(child);
                continue;
            };
            let value = read_preformatted_text(math.scope);
            let place = read_place(math.marked, file);
            let (context, settings) = katex.get_or_insert_with(|| {
                (KatexContext::default(), create_settings(options))
            });
            let start = children.len();
            let nodes =
                typeset_math(&value, math.display, context, settings, place, file, alloc);
            children.extend(nodes);
            rendered.push(start..children.len());
        }
        let mut rendered = rendered.into_iter().peekable();
        for (index, child) in children.iter_mut().enumerate() {
            while rendered.next_if(|range| range.end <= index).is_some() {}
            if rendered.peek().is_none_or(|range| !range.contains(&index)) {
                stack.push(child);
            }
        }
    }
}

// A formula: the element replaced by its rendering, the one whose class marked it, and
// whether it is display math
struct Math<'n, 'a> {
    scope: &'n Element<'a>,
    marked: &'n Element<'a>,
    display: bool,
}

fn read_math<'n, 'a>(node: &'n Node<'a>, options: &Katex) -> Option<Math<'n, 'a>> {
    let Node::Element(element) = node else {
        return None;
    };
    if (options.tag_names.iter()).any(|name| name.eq_ignore_ascii_case(element.tag_name))
    {
        let display = read_attribute(element, "display")
            .is_some_and(|value| !value.eq_ignore_ascii_case("inline"));
        return Some(Math {
            scope: element,
            marked: element,
            display,
        });
    }
    let classes = read_classes(element);
    let display = classes.contains(&"math-display");
    if display || classes.contains(&"language-math") || classes.contains(&"math-inline") {
        return Some(Math {
            scope: element,
            marked: element,
            display,
        });
    }
    if element.tag_name != "pre" {
        return None;
    }
    // A ```math fence: the `<pre>` goes, in display mode, once its `<code>` is reached
    let code = element.children.iter().find_map(|child| match child {
        Node::Element(code) if code.tag_name == "code" && is_math(code) => Some(code),
        _ => None,
    })?;
    read_classes(code)
        .contains(&"language-math")
        .then_some(Math {
            scope: element,
            marked: code,
            display: true,
        })
}

fn is_math(element: &Element<'_>) -> bool {
    read_classes(element)
        .iter()
        .any(|class| matches!(*class, "language-math" | "math-display" | "math-inline"))
}

fn read_classes<'a>(element: &Element<'a>) -> Vec<&'a str> {
    let Some(property) = element
        .properties
        .iter()
        .find(|property| property.name == "className")
    else {
        return Vec::new();
    };
    let PropertyValue::List(items) = &property.value else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| match item {
            PropertyItem::String(name) => Some(*name),
            PropertyItem::Number(_) => None,
        })
        .collect()
}

fn read_place(element: &Element<'_>, file: &File) -> Option<UnistPosition> {
    match element.position.as_ref()? {
        Position::Unist(position) => Some((*position).clone()),
        position => Some(Lines::new(file.value.as_deref()?).to_unist_position(position)),
    }
}

#[cfg(test)]
#[path = "typeset_unit.rs"]
mod unit;
