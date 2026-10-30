use bumpalo::collections::String as ArenaString;
use textlane_core::mdast::{
    AlignKind, Blockquote, Code, ContainerDirective, DefList, DefListDescription,
    DefListTerm, Definition, FootnoteDefinition, Heading, Html, LeafDirective, List,
    ListItem, Literal, Math, MdxExpression, MdxJsxElement, MdxjsEsm, Node, Paragraph,
    Root, Table, TableCell, TableRow, Text, ThematicBreak, Toml, Yaml,
};
use textlane_core::{Allocator, ArenaVec, Point, Position};

use crate::actions::markdown::parse::ParseSettings;
use crate::helpers::decode::decode_string;
use crate::helpers::identifier::normalize_identifier;
use crate::helpers::math::create_math_data;
use crate::plugins::gfm::actions::text::autolink::autolink_text;
use crate::utils::onepass::block::{
    Block, Blocks, Kind, Line, NONE, join_lines_in, parse_blocks,
};
use std::cell::RefCell;

use textlane_core::Error;

use crate::utils::onepass::block::FlowItem;
use crate::utils::onepass::directive::DirectiveHead;
use crate::utils::onepass::inline::{InlineContext, InlineOutput, clean, parse_inline};
use crate::utils::onepass::mdx::{
    MdxBuild, MdxErrors, ScannedTag, build_attributes as build_jsx_attributes, check_tag,
    empty_data, mismatch_error, stringify_point, tail_error, unclosed_error,
};
use crate::utils::onepass::scan::{is_space_or_tab, scan_delimiter_aligns};

/// Parse Markdown into mdast in one pass over the lines and one over each leaf's text.
pub(crate) fn parse_onepass<'a>(input: &'a str, alloc: &'a Allocator) -> Node<'a> {
    let settings = ParseSettings::default();
    // Only MDX fails, and it is off here
    parse_onepass_source(
        input.strip_prefix('\u{FEFF}').unwrap_or(input),
        &settings,
        alloc,
    )
    .unwrap_or_else(|_| {
        Node::Root(Root {
            children: ArenaVec::new_in(alloc),
            position: None,
            data: None,
        })
    })
}

/// Parse a source whose byte order mark is already off, with the extensions `settings`
/// turns on; MDX's syntax errors fail it.
pub(crate) fn parse_onepass_source<'a>(
    input: &'a str,
    settings: &ParseSettings,
    alloc: &'a Allocator,
) -> Result<Node<'a>, Error> {
    let blocks = parse_blocks(input, settings);
    if let Some(error) = blocks.error.clone() {
        return Err(error);
    }
    let errors = RefCell::new(MdxErrors::default());
    let mut root = build_tree(&blocks, input, &errors, alloc);
    if let Some(error) = errors.borrow().first() {
        return Err(error);
    }
    if settings.gfm.is_some() {
        autolink_text(&mut root, alloc);
    }
    Ok(root)
}

// A JSX element a flow tag opened: children go in it until its closing tag
struct FlowElement<'a> {
    tag: ScannedTag,
    children: ArenaVec<'a, Node<'a>>,
}

struct Open<'a> {
    block: usize,
    next: u32,
    children: ArenaVec<'a, Node<'a>>,
    // Only definitions so far: a task list check may still open an item's content
    only_definitions: bool,
    checked: Option<bool>,
    elements: Vec<FlowElement<'a>>,
}

impl<'a> Open<'a> {
    fn push(&mut self, node: Node<'a>) {
        match self.elements.last_mut() {
            Some(element) => element.children.push(node),
            None => self.children.push(node),
        }
    }
}

fn build_tree<'a>(
    blocks: &Blocks,
    input: &'a str,
    errors: &RefCell<MdxErrors>,
    alloc: &'a Allocator,
) -> Node<'a> {
    let mut stack = vec![Open {
        block: 0,
        next: blocks.blocks[0].first,
        children: ArenaVec::new_in(alloc),
        only_definitions: false,
        checked: None,
        elements: vec![],
    }];
    let parser = blocks.settings.mdx.and_then(|mdx| mdx.0.as_ref());
    let mdx_on = blocks.settings.mdx.is_some();
    // The flow elements open anywhere, innermost last, with their tags' places
    let mut open_flow: Vec<(Option<String>, usize, usize)> = vec![];
    loop {
        let flow_count = open_flow.len();
        let flow_open = if mdx_on {
            open_flow.last().cloned()
        } else {
            None
        };
        let state = MdxBuild {
            parser,
            source: input,
            errors,
            flow_open,
            flow_count,
        };
        let mdx = mdx_on.then_some(&state);
        if errors.borrow().crash.is_some() {
            break;
        }
        let Some(top) = stack.last_mut() else { break };
        if top.next != NONE {
            let child = top.next as usize;
            top.next = blocks.blocks[child].next;
            match blocks.blocks[child].kind {
                Kind::BlockQuote
                | Kind::List { .. }
                | Kind::Item { .. }
                | Kind::Footnote { .. }
                | Kind::ContainerDirective { .. }
                | Kind::DefDescription { .. } => {
                    let item = matches!(blocks.blocks[child].kind, Kind::Item { .. });
                    top.only_definitions = false;
                    let mut children = ArenaVec::new_in(alloc);
                    // A container directive's label is its first child, a paragraph
                    if let Kind::ContainerDirective { index, .. } =
                        blocks.blocks[child].kind
                        && let Some(((start, end), (label_start, label_end))) =
                            blocks.directives[index as usize].label
                    {
                        children.push(Node::Paragraph(Paragraph {
                            children: build_label(blocks, input, start, end, mdx, alloc),
                            position: Some(Position::span(
                                Point::new(label_start, 0),
                                Point::new(label_end, 0),
                            )),
                            data: None,
                        }));
                    }
                    stack.push(Open {
                        block: child,
                        next: blocks.blocks[child].first,
                        children,
                        only_definitions: item,
                        checked: None,
                        elements: vec![],
                    });
                }
                Kind::MdxFlow { first, last } => {
                    top.only_definitions = false;
                    let items = &blocks.mdx_items[first as usize..last as usize];
                    let kind = token_type(blocks.blocks[top.block].kind);
                    let container =
                        (blocks.blocks[top.block].start, blocks.blocks[top.block].end);
                    let outer = state.flow_open.clone();
                    flow_items(
                        top,
                        outer,
                        items,
                        &mut open_flow,
                        kind,
                        container,
                        input,
                        errors,
                        alloc,
                    );
                }
                Kind::Esm {
                    value: (start, end),
                } => {
                    top.only_definitions = false;
                    top.push(Node::MdxjsEsm(MdxjsEsm {
                        value: clean(alloc, &input[start..end]),
                        position: Some(Position::span(
                            Point::new(start, 0),
                            Point::new(end, 0),
                        )),
                        data: Some(empty_data(alloc)),
                    }));
                }
                kind => {
                    let task = top.only_definitions
                        && blocks.settings.gfm.is_some()
                        && kind == Kind::Paragraph;
                    let (node, checked) =
                        build_leaf(blocks, child, input, task, mdx, alloc);
                    if checked.is_some() {
                        top.checked = checked;
                    }
                    if !matches!(kind, Kind::Definition { .. }) {
                        top.only_definitions = false;
                    }
                    top.push(node);
                }
            }
            continue;
        }
        let Some(mut open) = stack.pop() else { break };
        // An element still open where its container ends
        if let Some(element) = open.elements.last() {
            let kind =
                (open.block != 0).then(|| token_type(blocks.blocks[open.block].kind));
            let tag = &element.tag;
            errors.borrow_mut().compile(unclosed_error(
                tag.name.as_deref(),
                &place(input, tag.start, tag.end),
                kind,
            ));
            break;
        }
        if blocks.settings.definition_list {
            open.children = group_def_lists(open.children, input.len(), alloc);
        }
        let block = &blocks.blocks[open.block];
        let position = Some(Position::span(block.start, block.end));
        let node = match block.kind {
            Kind::BlockQuote => Node::Blockquote(Blockquote {
                children: open.children,
                position,
                data: None,
            }),
            Kind::List { ordered, start, .. } => Node::List(List {
                ordered,
                start: ordered.then_some(start),
                spread: Some(block.spread),
                children: open.children,
                position,
                data: None,
            }),
            Kind::Footnote {
                label: (start, end),
            } => {
                let raw = &input[start..end];
                Node::FootnoteDefinition(FootnoteDefinition {
                    identifier: alloc
                        .alloc_str(&normalize_identifier(raw).to_lowercase()),
                    label: Some(clean(alloc, alloc.alloc_str(&decode_string(raw)))),
                    children: open.children,
                    position,
                    data: None,
                })
            }
            Kind::DefDescription { .. } => Node::DefListDescription(DefListDescription {
                spread: block.spread,
                children: open.children,
                position,
                data: None,
            }),
            Kind::ContainerDirective { index, .. } => {
                let head = &blocks.directives[index as usize];
                Node::ContainerDirective(ContainerDirective {
                    name: &input[head.name.0..head.name.1],
                    attributes: build_attributes(head, alloc),
                    label: head.label.is_some(),
                    children: open.children,
                    position,
                    data: None,
                })
            }
            Kind::Item { .. } => Node::ListItem(ListItem {
                spread: Some(block.spread),
                checked: open.checked,
                children: open.children,
                position,
                data: None,
            }),
            _ => Node::Root(Root {
                children: open.children,
                position: Some(Position::span(Point::new(0, 0), block.end)),
                data: None,
            }),
        };
        match stack.last_mut() {
            Some(parent) => parent.push(node),
            None => return node,
        }
    }
    Node::Root(Root {
        children: ArenaVec::new_in(alloc),
        position: None,
        data: None,
    })
}

fn build_leaf<'a>(
    blocks: &Blocks,
    index: usize,
    input: &'a str,
    task: bool,
    mdx: Option<&MdxBuild>,
    alloc: &'a Allocator,
) -> (Node<'a>, Option<bool>) {
    let block = &blocks.blocks[index];
    let mut position = Some(Position::span(block.start, block.end));
    let lines = &blocks.lines[block.lines.0 as usize..block.lines.1 as usize];
    let node = match block.kind {
        Kind::Paragraph => {
            // A paragraph whose lazy lines became terms keeps its last line ending
            let mut own;
            let lines = if block.swallowed && !lines.is_empty() {
                own = lines.to_vec();
                if let Some(last) = own.last_mut() {
                    last.end = last.next;
                }
                &own[..]
            } else {
                lines
            };
            let output =
                build_phrasing(blocks, lines, input, task, "paragraph", mdx, alloc);
            // A task list check's paragraph starts where its first text now does
            if output.checked.is_some()
                && let Some(Node::Text(Text {
                    position: Some(Position::Span { start, .. }),
                    ..
                })) = output.children.first()
                && let Some(Position::Span { end, .. }) = position
            {
                position = Some(Position::span(*start, end));
            }
            let node = Node::Paragraph(Paragraph {
                children: output.children,
                position,
                data: None,
            });
            return (node, output.checked);
        }
        Kind::Heading { depth, setext } => Node::Heading(Heading {
            depth,
            children: build_phrasing(
                blocks,
                lines,
                input,
                false,
                if setext {
                    "setextHeading"
                } else {
                    "atxHeading"
                },
                mdx,
                alloc,
            )
            .children,
            position,
            data: None,
        }),
        Kind::Table => build_table(blocks, lines, input, position, mdx, alloc),
        Kind::DefTerm => Node::DefListTerm(DefListTerm {
            children: build_term(blocks, lines, input, block.swallowed, mdx, alloc),
            position,
            data: None,
        }),
        Kind::LeafDirective { index } => {
            let head = &blocks.directives[index as usize];
            let children = match head.label {
                Some(((start, end), _)) => {
                    build_label(blocks, input, start, end, mdx, alloc)
                }
                None => ArenaVec::new_in(alloc),
            };
            Node::LeafDirective(LeafDirective {
                name: &input[head.name.0..head.name.1],
                attributes: build_attributes(head, alloc),
                children,
                position,
                data: None,
            })
        }
        Kind::Frontmatter {
            matter,
            value: (start, end),
        } => {
            let value = clean(alloc, &input[start..end]);
            let kind = blocks
                .settings
                .frontmatter
                .get(matter as usize)
                .map_or("", |m| m.kind.as_str());
            match kind {
                "yaml" => Node::Yaml(Yaml {
                    value,
                    position,
                    data: None,
                }),
                "toml" => Node::Toml(Toml {
                    value,
                    position,
                    data: None,
                }),
                _ => Node::Literal(Literal {
                    kind: alloc.alloc_str(kind),
                    value,
                    position,
                    data: None,
                }),
            }
        }
        Kind::ThematicBreak => Node::ThematicBreak(ThematicBreak {
            position,
            data: None,
        }),
        Kind::Fenced {
            marker: b'$',
            info: (info_start, info_end),
            ..
        } => {
            let meta = &input[info_start..info_end];
            let meta = (!meta.is_empty()).then(|| {
                let decoded: &str = if meta.contains(['\\', '&']) {
                    alloc.alloc_str(&decode_string(meta))
                } else {
                    meta
                };
                clean(alloc, decoded)
            });
            let value = clean(alloc, fenced_value(block, input, lines, alloc));
            Node::Math(Math {
                meta,
                value,
                position,
                data: Some(create_math_data(alloc, value)),
            })
        }
        Kind::Fenced {
            info: (info_start, info_end),
            ..
        } => {
            let info = &input[info_start..info_end];
            let lang_end = info.find([' ', '\t']).unwrap_or(info.len());
            let lang = &info[..lang_end];
            let meta = info[lang_end..].trim_start_matches([' ', '\t']);
            let decode = |value: &'a str| -> Option<&'a str> {
                if value.is_empty() {
                    None
                } else if value.contains(['\\', '&']) {
                    Some(alloc.alloc_str(&decode_string(value)))
                } else {
                    Some(value)
                }
            };
            Node::Code(Code {
                lang: decode(lang).map(|lang| clean(alloc, lang)),
                meta: decode(meta).map(|meta| clean(alloc, meta)),
                value: clean(alloc, fenced_value(block, input, lines, alloc)),
                position,
                data: None,
            })
        }
        Kind::Indented => Node::Code(Code {
            lang: None,
            meta: None,
            value: clean(
                alloc,
                strip_line_ending(join_value(input, lines, false, alloc)),
            ),
            position,
            data: None,
        }),
        Kind::Html { .. } => {
            let value = join_value(input, lines, block.swallowed, alloc);
            Node::Html(Html {
                value: clean(alloc, value),
                position,
                data: None,
            })
        }
        Kind::Definition { index } => {
            let def = &blocks.defs[index as usize];
            Node::Definition(Definition {
                identifier: alloc.alloc_str(&def.identifier),
                label: Some(alloc.alloc_str(&def.label)),
                url: alloc.alloc_str(&def.url),
                title: def.title.as_deref().map(|title| &*alloc.alloc_str(title)),
                position,
                data: None,
            })
        }
        _ => Node::Paragraph(Paragraph {
            children: ArenaVec::new_in(alloc),
            position,
            data: None,
        }),
    };
    (node, None)
}

fn build_phrasing<'a>(
    blocks: &Blocks,
    lines: &[Line],
    input: &'a str,
    task: bool,
    leaf: &'static str,
    mdx: Option<&MdxBuild>,
    alloc: &'a Allocator,
) -> InlineOutput<'a> {
    if lines.is_empty() || (lines.len() == 1 && lines[0].start == lines[0].end) {
        return InlineOutput {
            children: ArenaVec::new_in(alloc),
            checked: None,
        };
    }
    let (text, segments) = join_lines_in(input, lines, alloc);
    let mut context = context(blocks, task, false, mdx);
    context.leaf = leaf;
    parse_inline(alloc, text, &segments, &context)
}

fn context<'d>(
    blocks: &'d Blocks,
    task: bool,
    in_table: bool,
    mdx: Option<&'d MdxBuild<'d>>,
) -> InlineContext<'d> {
    InlineContext {
        identifiers: &blocks.identifiers,
        longest_identifier: blocks.longest_identifier,
        gfm: blocks.settings.gfm,
        math: blocks.settings.math,
        footnotes: &blocks.footnotes,
        task,
        in_table,
        directive: blocks.settings.directive,
        keep_trailing: false,
        mdx,
        leaf: "paragraph",
    }
}

// A term's line; two spaces or more before its line ending make a hard break that takes it
fn build_term<'a>(
    blocks: &Blocks,
    lines: &[Line],
    input: &'a str,
    followed: bool,
    mdx: Option<&MdxBuild>,
    alloc: &'a Allocator,
) -> ArenaVec<'a, Node<'a>> {
    let bytes = input.as_bytes();
    let mut own = lines.to_vec();
    if let Some(line) = own.last_mut() {
        let mut spaces = 0;
        while line.end > line.start + spaces && bytes[line.end - 1 - spaces] == b' ' {
            spaces += 1;
        }
        let mut slashes = 0;
        while line.end > line.start + slashes && bytes[line.end - 1 - slashes] == b'\\' {
            slashes += 1;
        }
        // Two spaces, or an unescaped backslash, before the line ending: a hard break
        if followed && (spaces >= 2 || slashes % 2 == 1) && line.next > line.end {
            line.end = line.next;
        }
    }
    build_phrasing(blocks, &own, input, false, "defListTerm", mdx, alloc).children
}

// Runs of terms and descriptions, only blank lines between them, are definition lists
fn group_def_lists<'a>(
    children: ArenaVec<'a, Node<'a>>,
    input_len: usize,
    alloc: &'a Allocator,
) -> ArenaVec<'a, Node<'a>> {
    let is_part =
        |node: &Node| matches!(node, Node::DefListTerm(_) | Node::DefListDescription(_));
    if !children.iter().any(is_part) {
        return children;
    }
    let mut grouped = ArenaVec::with_capacity_in(children.len(), alloc);
    let mut run: Option<ArenaVec<'a, Node<'a>>> = None;
    let flush = |run: &mut Option<ArenaVec<'a, Node<'a>>>,
                 grouped: &mut ArenaVec<'a, Node<'a>>| {
        if let Some(mut items) = run.take() {
            let start = items.first().and_then(span_of).map(|span| span.0);
            // An empty description at the very end starts where its list does, as
            // mdast-util-definition-list takes the token after its prefix
            if let (Some(list_start), Some(Node::DefListDescription(last))) =
                (start, items.last_mut())
                && last.children.is_empty()
                && let Some(Position::Span { start: from, end }) = &mut last.position
                && from.offset() == end.offset()
                && end.offset() == input_len
            {
                *from = list_start;
            }
            let end = items.last().and_then(span_of).map(|span| span.1);
            let position = start
                .zip(end)
                .map(|(start, end)| Position::span(start, end));
            grouped.push(Node::DefList(DefList {
                children: items,
                position,
                data: None,
            }));
        }
    };
    for child in children {
        if is_part(&child) {
            run.get_or_insert_with(|| ArenaVec::new_in(alloc))
                .push(child);
        } else {
            flush(&mut run, &mut grouped);
            grouped.push(child);
        }
    }
    flush(&mut run, &mut grouped);
    grouped
}

fn span_of(node: &Node<'_>) -> Option<(Point, Point)> {
    let position = match node {
        Node::DefListTerm(term) => term.position,
        Node::DefListDescription(description) => description.position,
        _ => None,
    };
    match position {
        Some(Position::Span { start, end }) => Some((start, end)),
        _ => None,
    }
}

fn place(input: &str, start: usize, end: usize) -> String {
    format!(
        "{}-{}",
        stringify_point(input, Point::new(start, 0)),
        stringify_point(input, Point::new(end, 0))
    )
}

// The token type of a container, as MDX's errors name it
fn token_type(kind: Kind) -> &'static str {
    match kind {
        Kind::BlockQuote => "blockQuote",
        Kind::Item { .. } => "listItem",
        Kind::List { .. } => "listUnordered",
        Kind::Footnote { .. } => "gfmFootnoteDefinition",
        Kind::DefDescription { .. } => "defListDescription",
        Kind::ContainerDirective { .. } => "directiveContainer",
        _ => "document",
    }
}

// A flow line's tags and expressions: tags open and close elements in their container
#[allow(clippy::too_many_arguments)]
fn flow_items<'a>(
    top: &mut Open<'a>,
    outer: Option<(Option<String>, usize, usize)>,
    items: &[FlowItem],
    open_flow: &mut Vec<(Option<String>, usize, usize)>,
    kind: &str,
    container: (Point, Point),
    input: &'a str,
    errors: &RefCell<MdxErrors>,
    alloc: &'a Allocator,
) {
    let span = |start: usize, end: usize| {
        Position::span(Point::new(start, 0), Point::new(end, 0))
    };
    for item in items {
        let count = open_flow.len();
        match item {
            FlowItem::Expression {
                span: (start, end),
                value,
            } => {
                top.push(Node::MdxFlowExpression(MdxExpression {
                    value: alloc.alloc_str(value),
                    position: Some(span(*start, *end)),
                    data: Some(empty_data(alloc)),
                }));
            }
            FlowItem::Tag(tag) => {
                if let Some(error) = check_tag(tag, count > 0) {
                    errors.borrow_mut().compile(error);
                    return;
                }
                // The innermost element open anywhere must be the one a closing tag closes
                let innermost = top
                    .elements
                    .last()
                    .map(|open| (open.tag.name.clone(), open.tag.start, open.tag.end))
                    .or_else(|| outer.clone());
                if tag.close
                    && let Some((name, start, end)) = innermost
                    && name != tag.name
                {
                    let open_place = place(input, start, end);
                    errors.borrow_mut().compile(mismatch_error(
                        tag,
                        name.as_deref(),
                        &open_place,
                    ));
                    return;
                }
                if !tag.close {
                    top.elements.push(FlowElement {
                        tag: tag.clone(),
                        children: ArenaVec::new_in(alloc),
                    });
                    open_flow.push((tag.name.clone(), tag.start, tag.end));
                }
                if tag.self_closing || tag.close {
                    if top.elements.is_empty() {
                        let exit = stringify_point(input, container.1);
                        let enter = stringify_point(input, container.0);
                        errors
                            .borrow_mut()
                            .compile(tail_error(tag, kind, &exit, &enter));
                        return;
                    }
                    let Some(element) = top.elements.pop() else {
                        return;
                    };
                    open_flow.pop();
                    let open = &element.tag;
                    let name = open.name.as_deref().map(|name| &*alloc.alloc_str(name));
                    let node = Node::MdxJsxFlowElement(MdxJsxElement {
                        name,
                        attributes: build_jsx_attributes(alloc, open, input, span),
                        children: element.children,
                        position: Some(span(open.start, tag.end)),
                        data: None,
                    });
                    top.push(node);
                }
            }
        }
    }
}

// A directive label's phrasing, whitespace at its end kept
fn build_label<'a>(
    blocks: &Blocks,
    input: &'a str,
    start: usize,
    end: usize,
    mdx: Option<&MdxBuild>,
    alloc: &'a Allocator,
) -> ArenaVec<'a, Node<'a>> {
    if end <= start {
        return ArenaVec::new_in(alloc);
    }
    let mut context = context(blocks, false, false, mdx);
    context.keep_trailing = true;
    parse_inline(alloc, &input[start..end], &[(0, start)], &context).children
}

fn build_attributes<'a>(
    head: &DirectiveHead,
    alloc: &'a Allocator,
) -> ArenaVec<'a, (&'a str, &'a str)> {
    let mut attributes = ArenaVec::with_capacity_in(head.attributes.len(), alloc);
    for (key, value) in &head.attributes {
        attributes.push((&*alloc.alloc_str(key), clean(alloc, alloc.alloc_str(value))));
    }
    attributes
}

// A GFM table: its head row, then its body rows; the delimiter row gives the alignment
fn build_table<'a>(
    blocks: &Blocks,
    lines: &[Line],
    input: &'a str,
    position: Option<Position<'a>>,
    mdx: Option<&MdxBuild>,
    alloc: &'a Allocator,
) -> Node<'a> {
    let bytes = input.as_bytes();
    let mut align = ArenaVec::new_in(alloc);
    if let Some(delimiter) = lines.get(1) {
        let cells = scan_delimiter_aligns(bytes, delimiter.start, delimiter.end)
            .unwrap_or_default();
        align.extend(cells.into_iter().map(|cell| match cell {
            (true, true) => AlignKind::Center,
            (true, false) => AlignKind::Left,
            (false, true) => AlignKind::Right,
            (false, false) => AlignKind::None,
        }));
    }
    let mut rows = ArenaVec::with_capacity_in(lines.len().saturating_sub(1), alloc);
    for (index, line) in lines.iter().enumerate() {
        if index == 1 {
            continue;
        }
        let mut start = line.start;
        while start < line.end && is_space_or_tab(bytes[start]) {
            start += 1;
        }
        rows.push(Node::TableRow(TableRow {
            children: build_cells(blocks, input, start, line.end, mdx, alloc),
            position: Some(Position::span(
                Point::new(start, 0),
                Point::new(line.end, 0),
            )),
            data: None,
        }));
    }
    Node::Table(Table {
        align,
        children: rows,
        position,
        data: None,
    })
}

// A row's cells as micromark-extension-gfm-table groups them: each from its pipe to the
// next, the first from the row's start; a last cell with nothing in it goes to the one
// before
fn build_cells<'a>(
    blocks: &Blocks,
    input: &'a str,
    start: usize,
    end: usize,
    mdx: Option<&MdxBuild>,
    alloc: &'a Allocator,
) -> ArenaVec<'a, Node<'a>> {
    let bytes = input.as_bytes();
    // Each cell: its start, and where its values begin and end
    let mut cells: Vec<(usize, Option<(usize, usize)>)> = vec![(start, None)];
    let mut index = start;
    let mut first_pipe = true;
    while index < end {
        match bytes[index] {
            b' ' | b'\t' => index += 1,
            b'|' => {
                if !first_pipe {
                    cells.push((index, None));
                }
                first_pipe = false;
                index += 1;
            }
            _ => {
                first_pipe = false;
                let value_start = index;
                while index < end && !matches!(bytes[index], b' ' | b'\t' | b'|') {
                    if bytes[index] == b'\\'
                        && index + 1 < end
                        && matches!(bytes[index + 1], b'\\' | b'|')
                    {
                        index += 1;
                    }
                    index += 1;
                }
                if let Some(cell) = cells.last_mut() {
                    cell.1 = Some(match cell.1 {
                        Some((first, _)) => (first, index),
                        None => (value_start, index),
                    });
                }
            }
        }
    }
    if cells.len() > 1 && cells.last().is_some_and(|cell| cell.1.is_none()) {
        cells.pop();
    }
    let mut nodes = ArenaVec::with_capacity_in(cells.len(), alloc);
    for (position, (cell_start, values)) in cells.iter().enumerate() {
        let cell_end = cells.get(position + 1).map_or(end, |next| next.0);
        let children = match values {
            Some((value_start, value_end)) => {
                let text = &input[*value_start..*value_end];
                parse_inline(
                    alloc,
                    text,
                    &[(0, *value_start)],
                    &context(blocks, false, true, mdx),
                )
                .children
            }
            None => ArenaVec::new_in(alloc),
        };
        nodes.push(Node::TableCell(TableCell {
            children,
            position: Some(Position::span(
                Point::new(*cell_start, 0),
                Point::new(cell_end, 0),
            )),
            data: None,
        }));
    }
    nodes
}

// from-markdown drops one line ending at each end of a fence's text: an unclosed fence
// that took no line ending at the end loses its last one
fn fenced_value<'a>(
    block: &Block,
    input: &'a str,
    lines: &[Line],
    alloc: &'a Allocator,
) -> &'a str {
    let value = join_value(input, lines, false, alloc);
    if block.ended || block.swallowed {
        value
    } else {
        strip_line_ending(value)
    }
}

fn strip_line_ending(value: &str) -> &str {
    value
        .strip_suffix("\r\n")
        .or_else(|| value.strip_suffix(['\n', '\r']))
        .unwrap_or(value)
}

// A code or HTML value: the lines with their line endings, a partly taken tab as spaces
fn join_value<'a>(
    input: &'a str,
    lines: &[Line],
    with_eol: bool,
    alloc: &'a Allocator,
) -> &'a str {
    match lines {
        [] => "",
        [line] if line.virt == 0 => {
            &input[line.start..if with_eol { line.next } else { line.end }]
        }
        _ => {
            let size = lines.iter().map(|line| line.next - line.start + 3).sum();
            let mut value = ArenaString::with_capacity_in(size, alloc);
            for (index, line) in lines.iter().enumerate() {
                let mut start = line.start;
                if line.virt > 0 {
                    value.extend(std::iter::repeat_n(' ', line.pad as usize));
                    start += 1;
                }
                value.push_str(&input[start..line.end.max(start)]);
                if index + 1 < lines.len() || with_eol {
                    value.push_str(&input[line.end..line.next]);
                }
            }
            value.into_bump_str()
        }
    }
}
