use textlane_core::hast::{Element, Node, PropertyValue};
use textlane_core::nlcst::{self, Literal, Parent};
use textlane_core::{
    Allocator, ArenaString, ArenaVec, Location, Point, Position, ProseParser,
};

use crate::helpers::phrasing::{is_embedded, is_phrasing};
use crate::helpers::value::find_value;

/// hast into nlcst, as hast-util-to-nlcst does: each paragraph or heading, and each run of
/// phrasing content in an element that accepts flow, becomes a paragraph, with `source` the
/// text the tree was parsed from.
pub fn bridge_hast<'a>(
    tree: &Node<'a>,
    source: &str,
    parser: &dyn ProseParser,
    alloc: &'a Allocator,
) -> nlcst::Node<'a> {
    let state = State {
        location: Location::new(source),
        parser,
        alloc,
    };
    let mut paragraphs = ArenaVec::new_in(alloc);
    let mut tasks = vec![Task::Find(tree)];
    while let Some(task) = tasks.pop() {
        match task {
            Task::Find(node) => state.find(node, &mut tasks),
            Task::Add(nodes) => {
                if let Some(paragraph) = state.add(&nodes) {
                    paragraphs.push(paragraph);
                }
            }
        }
    }
    nlcst::Node::Root(Parent {
        children: paragraphs,
        position: Some(Position::span(
            Point::new(0, 0),
            Point::new(source.len(), 0),
        )),
        data: None,
    })
}

struct State<'s, 'a> {
    location: Location<'s>,
    parser: &'s dyn ProseParser,
    alloc: &'a Allocator,
}

// Work left, last first: `find` pushes what it would do next in reverse
enum Task<'n, 'a> {
    Find(&'n Node<'a>),
    Add(Vec<&'n Node<'a>>),
}

impl<'s, 'a> State<'s, 'a> {
    fn find<'n>(&self, node: &'n Node<'a>, tasks: &mut Vec<Task<'n, 'a>>) {
        let children = match node {
            Node::Root(root) => &root.children,
            Node::Element(element) if !is_ignored(element) => {
                if is_explicit(element) {
                    tasks.push(Task::Add(vec![node]));
                    return;
                }
                if is_flow_accepting(element) {
                    implicit(flatten(&element.children), tasks);
                    return;
                }
                &element.children
            }
            _ => return,
        };
        tasks.extend(children.iter().rev().map(Task::Find));
    }

    fn add(&self, nodes: &[&Node<'a>]) -> Option<nlcst::Node<'a>> {
        let mut content = ArenaVec::new_in(self.alloc);
        let mut stack: Vec<std::slice::Iter<'_, Node<'a>>> = Vec::new();
        for node in nodes {
            if let Some(children) = self.one(node, &mut content) {
                stack.push(children.iter());
            }
            while let Some(iter) = stack.last_mut() {
                match iter.next() {
                    Some(child) => {
                        if let Some(children) = self.one(child, &mut content) {
                            stack.push(children.iter());
                        }
                    }
                    None => {
                        stack.pop();
                    }
                }
            }
        }
        let position = match (content.first()?.position(), content.last()?.position()) {
            (Some(start), Some(end)) => Some(self.location.join(start, end, self.alloc)),
            _ => None,
        };
        Some(
            self.parser
                .tokenize_paragraph(content, position, self.alloc),
        )
    }

    // A leaf's nodes go into `content`; an element's children come back to be walked
    fn one<'n>(
        &self,
        node: &'n Node<'a>,
        content: &mut ArenaVec<'a, nlcst::Node<'a>>,
    ) -> Option<&'n [Node<'a>]> {
        let mut results = match node {
            Node::Text(text) => self.parser.tokenize(text.value, 0, self.alloc),
            Node::Element(element) if !is_ignored(element) => {
                let replacement = match element.tag_name {
                    "wbr" => white_space(" "),
                    "br" => white_space("\n"),
                    _ if is_source(element) => nlcst::Node::Source(Literal {
                        value: to_string(element, self.alloc),
                        position: None,
                        data: None,
                    }),
                    _ => return Some(&element.children),
                };
                let mut results = ArenaVec::new_in(self.alloc);
                results.push(replacement);
                results
            }
            _ => return None,
        };
        // Without a start, upstream keeps the parser's positions, which count from the
        // text rather than the document: none is closer
        let start = node.position().and_then(|position| self.offset(position));
        self.location.place_nlcst(&mut results, start, self.alloc);
        content.extend(results);
        None
    }

    // `toOffset(pointStart(node))`: the UTF-16 offset of a start's line and column
    fn offset(&self, position: &Position<'_>) -> Option<usize> {
        let point = match position {
            Position::Span { start, .. } => self.location.to_unist_point(*start),
            Position::Unist(position) => position.start,
        };
        if point.line == 0 || point.column == 0 {
            return None;
        }
        self.location.to_offset(point.line, point.column)
    }
}

// hast-util-to-nlcst's `implicit`: runs of phrasing content become paragraphs when they
// hold more than white space and embedded content; anything else is searched
fn implicit<'n, 'a>(children: Vec<&'n Node<'a>>, tasks: &mut Vec<Task<'n, 'a>>) {
    let mut work = Vec::new();
    let mut run = Vec::new();
    let mut viable = false;
    for child in children {
        if is_phrasing(child) {
            if !viable && !is_embedded_node(child) && !is_white_space(child) {
                viable = true;
            }
            run.push(child);
        } else {
            flush(&mut run, viable, &mut work);
            viable = false;
            work.push(Task::Find(child));
        }
    }
    flush(&mut run, viable, &mut work);
    tasks.extend(work.into_iter().rev());
}

fn flush<'n, 'a>(
    run: &mut Vec<&'n Node<'a>>,
    viable: bool,
    work: &mut Vec<Task<'n, 'a>>,
) {
    if run.is_empty() {
        return;
    }
    if viable {
        work.push(Task::Add(std::mem::take(run)));
    } else {
        work.extend(run.drain(..).map(Task::Find));
    }
}

// `flattenAll`: links, insertions, deletions and maps give way to their children
fn flatten<'n, 'a>(children: &'n [Node<'a>]) -> Vec<&'n Node<'a>> {
    let mut results = Vec::new();
    let mut stack = vec![children.iter()];
    while let Some(iter) = stack.last_mut() {
        match iter.next() {
            Some(Node::Element(element))
                if matches!(element.tag_name, "a" | "ins" | "del" | "map") =>
            {
                stack.push(element.children.iter());
            }
            Some(child) => results.push(child),
            None => {
                stack.pop();
            }
        }
    }
    results
}

// hast-util-to-string: the text of every descendant
fn to_string<'a>(element: &Element<'a>, alloc: &'a Allocator) -> &'a str {
    let mut value = ArenaString::new_in(alloc);
    let mut stack = vec![element.children.iter()];
    while let Some(iter) = stack.last_mut() {
        match iter.next() {
            Some(Node::Text(text)) => value.push_str(text.value),
            Some(Node::Element(element)) => stack.push(element.children.iter()),
            Some(Node::Root(root)) => stack.push(root.children.iter()),
            Some(Node::Unknown(unknown)) => {
                if let Some(children) = &unknown.children {
                    stack.push(children.iter());
                }
            }
            Some(Node::MdxJsxFlowElement(element) | Node::MdxJsxTextElement(element)) => {
                stack.push(element.children.iter());
            }
            Some(_) => {}
            None => {
                stack.pop();
            }
        }
    }
    value.into_bump_str()
}

fn is_ignored(element: &Element<'_>) -> bool {
    matches!(
        element.tag_name,
        "script" | "style" | "svg" | "math" | "del"
    ) || has_data_nlcst(element, "ignore")
}

fn is_source(element: &Element<'_>) -> bool {
    element.tag_name == "code" || has_data_nlcst(element, "source")
}

fn is_explicit(element: &Element<'_>) -> bool {
    matches!(
        element.tag_name,
        "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
    )
}

fn is_flow_accepting(element: &Element<'_>) -> bool {
    matches!(
        element.tag_name,
        "body"
            | "article"
            | "section"
            | "blockquote"
            | "nav"
            | "aside"
            | "header"
            | "footer"
            | "address"
            | "li"
            | "dt"
            | "dd"
            | "figure"
            | "figcaption"
            | "div"
            | "main"
            | "caption"
            | "td"
            | "th"
            | "form"
            | "fieldset"
            | "details"
            | "dialog"
    )
}

fn has_data_nlcst(element: &Element<'_>, expected: &str) -> bool {
    matches!(find_value(element, "dataNlcst"), Some(PropertyValue::String(value)) if *value == expected)
}

fn is_embedded_node(node: &Node<'_>) -> bool {
    matches!(node, Node::Element(element) if is_embedded(element))
}

// hast-util-whitespace: text of only spaces, tabs, line feeds, form feeds and returns
fn is_white_space(node: &Node<'_>) -> bool {
    matches!(node, Node::Text(text) if text.value.bytes().all(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\x0C' | b'\r')))
}

fn white_space(value: &str) -> nlcst::Node<'_> {
    nlcst::Node::WhiteSpace(Literal {
        value,
        position: None,
        data: None,
    })
}
