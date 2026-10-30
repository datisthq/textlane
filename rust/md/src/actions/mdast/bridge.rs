use textlane_core::mdast::Node;
use textlane_core::nlcst::{self, Literal, Parent};
use textlane_core::{
    Allocator, ArenaVec, Location, NaturalLanguageParser, Position, UnistPoint,
};

use crate::models::options::ToNaturalLanguage;
use crate::utils::serializer::state::read_node_type;

const DEFAULT_IGNORE: [&str; 3] = ["table", "tableRow", "tableCell"];
const DEFAULT_SOURCE: [&str; 1] = ["inlineCode"];

/// mdast into nlcst, as mdast-util-to-nlcst does: the document's text as one paragraph's
/// content, split into paragraphs at blank lines, with `source` the text the tree was
/// parsed from.
pub fn bridge_mdast<'a>(
    tree: &Node<'a>,
    source: &str,
    parser: &dyn NaturalLanguageParser,
    options: &ToNaturalLanguage,
    alloc: &'a Allocator,
) -> nlcst::Node<'a> {
    let state = State {
        location: Location::new(source),
        parser,
        options,
        alloc,
    };
    let mut content = ArenaVec::new_in(alloc);
    let mut stack = Vec::new();
    if let Some(children) = state.one(tree, &mut content) {
        stack.push(Frame {
            children,
            index: 0,
            end: None,
        });
    }
    while let Some(frame) = stack.last_mut() {
        let Some(child) = frame.children.get(frame.index) else {
            stack.pop();
            continue;
        };
        frame.index += 1;
        if let (Some(end), Some(start)) = (frame.end, state.point_start(child))
            && start.line != end.line
        {
            content.push(state.line_ending(start.line, end));
        }
        frame.end = state.point_end(child);
        if let Some(children) = state.one(child, &mut content) {
            stack.push(Frame {
                children,
                index: 0,
                end: None,
            });
        }
    }
    state.to_root(content)
}

struct State<'s, 'a> {
    location: Location<'s>,
    parser: &'s dyn NaturalLanguageParser,
    options: &'s ToNaturalLanguage,
    alloc: &'a Allocator,
}

// One parent being walked: `end` is where its last child ended, for line endings between
struct Frame<'n, 'a> {
    children: &'n [Node<'a>],
    index: usize,
    end: Option<UnistPoint>,
}

impl<'s, 'a> State<'s, 'a> {
    // A leaf's nodes go into `content`; a parent's children come back to be walked
    fn one<'n>(
        &self,
        node: &'n Node<'a>,
        content: &mut ArenaVec<'a, nlcst::Node<'a>>,
    ) -> Option<&'n [Node<'a>]> {
        let kind = read_node_type(node);
        if is_listed(&DEFAULT_IGNORE, &self.options.ignore, kind) {
            return None;
        }
        let mut start = node
            .position()
            .and_then(|position| self.unist(position).0.offset);
        let mut results = if is_listed(&DEFAULT_SOURCE, &self.options.source, kind) {
            let end = node
                .position()
                .and_then(|position| self.unist(position).1.offset);
            let (Some(from), Some(to)) = (start, end) else {
                return None;
            };
            let mut results = ArenaVec::new_in(self.alloc);
            results.push(source_node(self.slice(from, to)));
            results
        } else if let Some(children) = node.children() {
            return Some(children);
        } else {
            match node {
                Node::Image(image) => self.tokenize_alt(image.alt, &mut start)?,
                Node::ImageReference(image) => {
                    self.tokenize_alt(image.alt, &mut start)?
                }
                Node::Break(_) => {
                    let mut results = ArenaVec::new_in(self.alloc);
                    results.push(white_space("\n"));
                    results
                }
                Node::Text(text) => self.parser.tokenize(text.value, 0, self.alloc),
                _ => return None,
            }
        };
        self.location.place_nlcst(&mut results, start, self.alloc);
        content.extend(results);
        None
    }

    fn tokenize_alt(
        &self,
        alt: Option<&'a str>,
        start: &mut Option<usize>,
    ) -> Option<ArenaVec<'a, nlcst::Node<'a>>> {
        let alt = alt.filter(|alt| !alt.is_empty())?;
        *start = start.map(|start| start + 2);
        Some(self.parser.tokenize(alt, 0, self.alloc))
    }

    // As many line feeds as lines passed, placed after the previous node, but always two:
    // one would not end a paragraph
    fn line_ending(&self, line: usize, end: UnistPoint) -> nlcst::Node<'a> {
        let value = "\n".repeat(line.saturating_sub(end.line));
        let mut nodes = [white_space(self.alloc.alloc_str(&value))];
        self.location
            .place_nlcst(&mut nodes, end.offset, self.alloc);
        let [mut node] = nodes;
        if let nlcst::Node::WhiteSpace(literal) = &mut node
            && value.len() < 2
        {
            literal.value = "\n\n";
        }
        node
    }

    fn to_root(&self, content: ArenaVec<'a, nlcst::Node<'a>>) -> nlcst::Node<'a> {
        let position = match (content.first(), content.last()) {
            (Some(first), Some(last)) => match (first.position(), last.position()) {
                (Some(start), Some(end)) => {
                    Some(self.location.join(start, end, self.alloc))
                }
                _ => None,
            },
            _ => {
                return nlcst::Node::Root(Parent {
                    children: content,
                    position: None,
                    data: None,
                });
            }
        };
        let paragraph = self
            .parser
            .tokenize_paragraph(content, position, self.alloc);
        self.parser.tokenize_root(paragraph, position, self.alloc)
    }

    // The source between two UTF-16 offsets, as JavaScript's `slice` cuts it
    fn slice(&self, from: usize, to: usize) -> &'a str {
        let to = to.min(self.location.len());
        let from = from.min(to);
        let source = self.location.source();
        let text = match (self.location.to_byte(from), self.location.to_byte(to)) {
            (Some(from), Some(to)) => std::borrow::Cow::Borrowed(&source[from..to]),
            _ => {
                let units: Vec<u16> =
                    source.encode_utf16().skip(from).take(to - from).collect();
                std::borrow::Cow::Owned(String::from_utf16_lossy(&units))
            }
        };
        self.alloc.alloc_str(&text)
    }

    fn point_start(&self, node: &Node<'_>) -> Option<UnistPoint> {
        node.position()
            .map(|position| self.unist(position).0)
            .filter(is_valid)
    }

    fn point_end(&self, node: &Node<'_>) -> Option<UnistPoint> {
        node.position()
            .map(|position| self.unist(position).1)
            .filter(is_valid)
    }

    fn unist(&self, position: &Position<'_>) -> (UnistPoint, UnistPoint) {
        match position {
            Position::Span { start, end } => (
                self.location.to_unist_point(*start),
                self.location.to_unist_point(*end),
            ),
            Position::Unist(position) => (position.start, position.end),
        }
    }
}

fn is_listed(defaults: &[&str], extra: &[String], kind: &str) -> bool {
    defaults.contains(&kind) || extra.iter().any(|item| item == kind)
}

// unist-util-position reads a point only with a line and column from 1
fn is_valid(point: &UnistPoint) -> bool {
    point.line > 0 && point.column > 0
}

fn white_space(value: &str) -> nlcst::Node<'_> {
    nlcst::Node::WhiteSpace(Literal {
        value,
        position: None,
        data: None,
    })
}

fn source_node(value: &str) -> nlcst::Node<'_> {
    nlcst::Node::Source(Literal {
        value,
        position: None,
        data: None,
    })
}
