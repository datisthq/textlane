use textlane_core::mdast::{Emphasis, Node, Paragraph, Text};
use textlane_core::{Allocator, ArenaVec, Data};

use crate::models::options::AlertsOptions;

const TYPES: [&str; 5] = ["note", "tip", "important", "warning", "caution"];

/// remark-github-blockquote-alert 2.1.0: a block quote whose first matching paragraph opens
/// with `[!TYPE]` becomes GitHub's alert, through `data.hName` and `data.hProperties`.
pub(crate) fn mark_alerts<'a>(
    root: &mut Node<'a>,
    options: &AlertsOptions,
    alloc: &'a Allocator,
) {
    let mut stack: Vec<&mut Node<'a>> = vec![root];
    while let Some(node) = stack.pop() {
        if let Node::Blockquote(quote) = node
            && let Some((kind, title)) = take_marker(&mut quote.children, options, alloc)
        {
            quote.data = Some(alloc.alloc(alert_data(kind, options, alloc)));
            quote
                .children
                .insert(0, title_paragraph(kind, title, alloc));
        }
        if let Some(children) = node.children_mut() {
            stack.extend(children.iter_mut().rev());
        }
    }
}

// Upstream tries paragraphs only, in order, until one's first child is a text matching the
// marker: the marker's text is cut where the text holds a line ending, else the whole first
// child goes, with a `break` right after it
fn take_marker<'a>(
    children: &mut ArenaVec<'a, Node<'a>>,
    options: &AlertsOptions,
    alloc: &'a Allocator,
) -> Option<(&'static str, &'a str)> {
    for child in children.iter_mut() {
        let Node::Paragraph(paragraph) = child else {
            continue;
        };
        let Some(Node::Text(first)) = paragraph.children.first() else {
            continue;
        };
        let Some((kind, title, rest)) = read_marker(first.value, options.legacy_title)
        else {
            continue;
        };
        let title: &'a str = match title {
            Some(title) => alloc.alloc_str(title),
            None => alloc.alloc_str(&kind.to_ascii_uppercase()),
        };
        if first.value.contains('\n') {
            paragraph.children[0] = Node::Text(Text {
                value: rest.trim_start_matches('\n'),
                position: None,
                data: None,
            });
        } else {
            paragraph.children.remove(0);
            if matches!(paragraph.children.first(), Some(Node::Break(_))) {
                paragraph.children.remove(0);
            }
        }
        return Some((kind, title));
    }
    None
}

// `/^\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]/i`, or with `legacyTitle` the same with
// `(\/.*)?` before the bracket: `.*` is greedy to the line's last `]`; the title drops one `/`
fn read_marker(text: &str, legacy: bool) -> Option<(&'static str, Option<&str>, &str)> {
    let rest = text.strip_prefix("[!")?;
    let kind = TYPES.into_iter().find(|kind| {
        rest.get(..kind.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(kind))
    })?;
    let after = &rest[kind.len()..];
    if let Some(rest) = after.strip_prefix(']') {
        return Some((kind, None, rest));
    }
    if !legacy || !after.starts_with('/') {
        return None;
    }
    let line = after
        .split(['\n', '\r', '\u{2028}', '\u{2029}'])
        .next()
        .unwrap_or_default();
    let close = line.rfind(']')?;
    let title = &line[1..close];
    Some((kind, Some(title), &after[close + 1..]))
}

fn alert_data<'a>(kind: &str, options: &AlertsOptions, alloc: &'a Allocator) -> Data<'a> {
    let mut classes = ArenaVec::new_in(alloc);
    classes.push(Data::String("markdown-alert"));
    classes.push(Data::String(
        alloc.alloc_str(&format!("markdown-alert-{kind}")),
    ));
    for name in options
        .class_names
        .split(' ')
        .filter(|name| !name.is_empty())
    {
        classes.push(Data::String(alloc.alloc_str(name)));
    }
    let properties = object(
        alloc,
        [
            ("className", Data::Array(classes)),
            ("dir", Data::String("auto")),
        ],
    );
    object(
        alloc,
        [
            ("hName", Data::String(alloc.alloc_str(&options.tag_name))),
            ("hProperties", properties),
        ],
    )
}

fn title_paragraph<'a>(kind: &str, title: &'a str, alloc: &'a Allocator) -> Node<'a> {
    let mut children = ArenaVec::with_capacity_in(2, alloc);
    children.push(icon(kind, alloc));
    children.push(Node::Text(Text {
        value: title,
        position: None,
        data: None,
    }));
    let properties = object(
        alloc,
        [
            ("className", Data::String("markdown-alert-title")),
            ("dir", Data::String("auto")),
        ],
    );
    Node::Paragraph(Paragraph {
        children,
        position: None,
        data: Some(alloc.alloc(object(alloc, [("hProperties", properties)]))),
    })
}

// An Octicon as upstream writes it: `emphasis` nodes renamed to `svg` and `path`
fn icon<'a>(kind: &str, alloc: &'a Allocator) -> Node<'a> {
    let path = object(
        alloc,
        [
            ("hName", Data::String("path")),
            (
                "hProperties",
                object(alloc, [("d", Data::String(path_data(kind)))]),
            ),
        ],
    );
    let mut class = ArenaVec::new_in(alloc);
    class.push(Data::String("octicon"));
    let svg = object(
        alloc,
        [
            ("hName", Data::String("svg")),
            (
                "hProperties",
                object(
                    alloc,
                    [
                        ("className", Data::Array(class)),
                        ("viewBox", Data::String("0 0 16 16")),
                        ("width", Data::String("16")),
                        ("height", Data::String("16")),
                        ("ariaHidden", Data::String("true")),
                    ],
                ),
            ),
        ],
    );
    let mut children = ArenaVec::with_capacity_in(1, alloc);
    children.push(Node::Emphasis(Emphasis {
        children: ArenaVec::new_in(alloc),
        position: None,
        data: Some(alloc.alloc(path)),
    }));
    Node::Emphasis(Emphasis {
        children,
        position: None,
        data: Some(alloc.alloc(svg)),
    })
}

fn object<'a, const N: usize>(
    alloc: &'a Allocator,
    entries: [(&'a str, Data<'a>); N],
) -> Data<'a> {
    let mut fields = ArenaVec::with_capacity_in(N, alloc);
    fields.extend(entries);
    Data::Object(fields)
}

fn path_data(kind: &str) -> &'static str {
    match kind {
        "note" => {
            "M0 8a8 8 0 1 1 16 0A8 8 0 0 1 0 8Zm8-6.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13ZM6.5 7.75A.75.75 0 0 1 7.25 7h1a.75.75 0 0 1 .75.75v2.75h.25a.75.75 0 0 1 0 1.5h-2a.75.75 0 0 1 0-1.5h.25v-2h-.25a.75.75 0 0 1-.75-.75ZM8 6a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z"
        }
        "tip" => {
            "M8 1.5c-2.363 0-4 1.69-4 3.75 0 .984.424 1.625.984 2.304l.214.253c.223.264.47.556.673.848.284.411.537.896.621 1.49a.75.75 0 0 1-1.484.211c-.04-.282-.163-.547-.37-.847a8.456 8.456 0 0 0-.542-.68c-.084-.1-.173-.205-.268-.32C3.201 7.75 2.5 6.766 2.5 5.25 2.5 2.31 4.863 0 8 0s5.5 2.31 5.5 5.25c0 1.516-.701 2.5-1.328 3.259-.095.115-.184.22-.268.319-.207.245-.383.453-.541.681-.208.3-.33.565-.37.847a.751.751 0 0 1-1.485-.212c.084-.593.337-1.078.621-1.489.203-.292.45-.584.673-.848.075-.088.147-.173.213-.253.561-.679.985-1.32.985-2.304 0-2.06-1.637-3.75-4-3.75ZM5.75 12h4.5a.75.75 0 0 1 0 1.5h-4.5a.75.75 0 0 1 0-1.5ZM6 15.25a.75.75 0 0 1 .75-.75h2.5a.75.75 0 0 1 0 1.5h-2.5a.75.75 0 0 1-.75-.75Z"
        }
        "important" => {
            "M0 1.75C0 .784.784 0 1.75 0h12.5C15.216 0 16 .784 16 1.75v9.5A1.75 1.75 0 0 1 14.25 13H8.06l-2.573 2.573A1.458 1.458 0 0 1 3 14.543V13H1.75A1.75 1.75 0 0 1 0 11.25Zm1.75-.25a.25.25 0 0 0-.25.25v9.5c0 .138.112.25.25.25h2a.75.75 0 0 1 .75.75v2.19l2.72-2.72a.749.749 0 0 1 .53-.22h6.5a.25.25 0 0 0 .25-.25v-9.5a.25.25 0 0 0-.25-.25Zm7 2.25v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 9a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z"
        }
        "warning" => {
            "M6.457 1.047c.659-1.234 2.427-1.234 3.086 0l6.082 11.378A1.75 1.75 0 0 1 14.082 15H1.918a1.75 1.75 0 0 1-1.543-2.575Zm1.763.707a.25.25 0 0 0-.44 0L1.698 13.132a.25.25 0 0 0 .22.368h12.164a.25.25 0 0 0 .22-.368Zm.53 3.996v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 11a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z"
        }
        _ => {
            "M4.47.22A.749.749 0 0 1 5 0h6c.199 0 .389.079.53.22l4.25 4.25c.141.14.22.331.22.53v6a.749.749 0 0 1-.22.53l-4.25 4.25A.749.749 0 0 1 11 16H5a.749.749 0 0 1-.53-.22L.22 11.53A.749.749 0 0 1 0 11V5c0-.199.079-.389.22-.53Zm.84 1.28L1.5 5.31v5.38l3.81 3.81h5.38l3.81-3.81V5.31L10.69 1.5ZM8 4a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-1.5 0v-3.5A.75.75 0 0 1 8 4Zm0 8a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z"
        }
    }
}
