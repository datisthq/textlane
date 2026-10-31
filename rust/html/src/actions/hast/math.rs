use textlane_core::hast::{Element, Node, Property, PropertyItem, PropertyValue};
use textlane_core::{Allocator, ArenaVec};

use crate::helpers::prose::{create_text, replace_prose};
use crate::models::options::Math;

/// Turn `$…$` and `$$…$$` in text into the `<code class="language-math math-inline">` and
/// `math-display` elements `md.math` makes, for `html.katex` to render.
pub fn mark_math<'a>(node: &mut Node<'a>, options: &Math, alloc: &'a Allocator) {
    replace_prose(
        node,
        &options.ignore,
        &options.ignore_attributes,
        alloc,
        |text, out| {
            if !text.contains('$') {
                return false;
            }
            let segments = split_math(text, options.single_dollar_text_math);
            if !segments
                .iter()
                .any(|segment| !matches!(segment, Segment::Text(_)))
            {
                return false;
            }
            let mut plain = String::new();
            for segment in segments {
                match segment {
                    Segment::Text(text) => plain.push_str(&text),
                    Segment::Dollar => plain.push('$'),
                    Segment::Math(latex, display) => {
                        if !plain.is_empty() {
                            out.push(create_text(&std::mem::take(&mut plain), alloc));
                        }
                        out.push(create_math(&latex, display, alloc));
                    }
                }
            }
            if !plain.is_empty() {
                out.push(create_text(&plain, alloc));
            }
            true
        },
    );
}

enum Segment {
    Text(String),
    Math(String, bool),
    Dollar,
}

fn split_math(text: &str, single: bool) -> Vec<Segment> {
    let mut segments = vec![];
    let mut plain = String::new();
    let mut rest = text;
    while let Some(at) = rest.find('$') {
        plain.push_str(&rest[..at]);
        let after = &rest[at..];
        if plain.ends_with('\\') {
            plain.pop();
            segments.push(Segment::Text(std::mem::take(&mut plain)));
            segments.push(Segment::Dollar);
            rest = &after[1..];
            continue;
        }
        let found = match after.strip_prefix("$$") {
            Some(inner) => inner.find("$$").and_then(|end| {
                let latex = inner[..end].trim();
                (!latex.is_empty()).then(|| (latex.to_owned(), true, end + 4))
            }),
            None if single => find_inline(&after[1..])
                .map(|(latex, end)| (latex.to_owned(), false, end + 1)),
            None => None,
        };
        match found {
            Some((latex, display, length)) => {
                segments.push(Segment::Text(std::mem::take(&mut plain)));
                segments.push(Segment::Math(latex, display));
                rest = &after[length..];
            }
            None => {
                plain.push('$');
                rest = &after[1..];
            }
        }
    }
    plain.push_str(rest);
    segments.push(Segment::Text(plain));
    segments
}

// The LaTeX of an inline `$…$` starting right after its opening `$`, and how far its
// closing `$` reaches
fn find_inline(text: &str) -> Option<(&str, usize)> {
    if text.is_empty() || text.starts_with(char::is_whitespace) {
        return None;
    }
    let mut search = 0;
    while let Some(offset) = text[search..].find('$') {
        let end = search + offset;
        let before = text[..end].chars().next_back();
        let after = text[end + 1..].chars().next();
        let closes = before.is_some_and(|char| !char.is_whitespace() && char != '\\')
            && !after.is_some_and(|char| char.is_ascii_digit());
        if closes && end > 0 {
            return Some((&text[..end], end + 1));
        }
        search = end + 1;
    }
    None
}

fn create_math<'a>(latex: &str, display: bool, alloc: &'a Allocator) -> Node<'a> {
    let mut classes = ArenaVec::with_capacity_in(2, alloc);
    classes.push(PropertyItem::String("language-math"));
    classes.push(PropertyItem::String(match display {
        true => "math-display",
        false => "math-inline",
    }));
    let mut properties = ArenaVec::with_capacity_in(1, alloc);
    properties.push(Property {
        name: "className",
        value: PropertyValue::List(classes),
    });
    let mut children = ArenaVec::with_capacity_in(1, alloc);
    children.push(create_text(latex, alloc));
    Node::Element(Element {
        tag_name: "code",
        properties,
        children,
        position: None,
        data: None,
        content: None,
    })
}

#[cfg(test)]
#[path = "math_unit.rs"]
mod unit;
