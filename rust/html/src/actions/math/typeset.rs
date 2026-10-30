use katex::macros::MacroDefinition;
use katex::{
    KatexContext, OutputFormat, Settings, StrictMode, StrictSetting, TrustSetting,
};
use textlane_core::hast::{Element, Node, Property, PropertyItem, PropertyValue, Text};
use textlane_core::{Allocator, ArenaVec, File, Message, UnistPosition};

use crate::actions::html::parse::parse_html;
use crate::models::options::{Katex, KatexOutput, KatexStrict, KatexStrictMode, Parse};

// katex-rs recurses once per group or style switch, up to ~46 KB a level in a debug build,
// and a stack overflow aborts the host process; KaTeX.js throws a `RangeError` there instead
const LEVEL_STACK: usize = 64 * 1024;
const MAX_LEVELS: usize = 4096;
const DEFINITIONS: [&str; 8] = [
    "def",
    "gdef",
    "edef",
    "xdef",
    "newcommand",
    "renewcommand",
    "providecommand",
    "DeclareMathOperator",
];

/// KaTeX's settings for `options`, inline until a formula asks for display.
pub(crate) fn create_settings(options: &Katex) -> Settings {
    let macros = options
        .macros
        .iter()
        .map(|(name, body)| (name.clone(), MacroDefinition::String(body.clone())))
        .collect();
    Settings::builder()
        .maybe_output(options.output.map(|output| match output {
            KatexOutput::Html => OutputFormat::Html,
            KatexOutput::Mathml => OutputFormat::Mathml,
            KatexOutput::HtmlAndMathml => OutputFormat::HtmlAndMathml,
        }))
        .leqno(options.leqno)
        .fleqn(options.fleqn)
        .throw_on_error(true)
        .maybe_error_color(options.error_color.clone())
        .macros(macros)
        .maybe_min_rule_thickness(options.min_rule_thickness)
        .color_is_text_color(options.color_is_text_color)
        .maybe_strict(options.strict.as_ref().map(|strict| match strict {
            KatexStrict::Boolean(strict) => StrictSetting::Bool(*strict),
            KatexStrict::Mode(mode) => StrictSetting::Mode(match mode {
                KatexStrictMode::Ignore => StrictMode::Ignore,
                KatexStrictMode::Warn => StrictMode::Warn,
                KatexStrictMode::Error => StrictMode::Error,
            }),
        }))
        .trust(TrustSetting::Bool(options.trust))
        .maybe_max_size(options.max_size)
        .maybe_max_expand(options.max_expand)
        .global_group(options.global_group)
        .build()
}

/// One formula as rehype-katex renders it: KaTeX's markup as hast, or, when it fails, a
/// message on `file` and KaTeX's error rendering.
pub(crate) fn typeset_math<'a>(
    value: &str,
    display: bool,
    context: &KatexContext,
    settings: &mut Settings,
    place: Option<UnistPosition>,
    file: &mut File,
    alloc: &'a Allocator,
) -> ArenaVec<'a, Node<'a>> {
    let message = |rule: &str| Message {
        reason: "Could not render math with KaTeX".to_string(),
        source: Some("rehype-katex".to_string()),
        rule: Some(rule.to_string()),
        place,
    };
    let Some((levels, max_expand)) = measure_levels(value, settings) else {
        file.messages.push(message("rangeerror"));
        let title = "RangeError: Maximum call stack size exceeded";
        return create_error(value, title, settings, alloc);
    };
    let expand = std::mem::replace(&mut settings.max_expand, max_expand);
    settings.display_mode = display;
    let render = |settings: &Settings| {
        let stack = levels * LEVEL_STACK;
        stacker::maybe_grow(stack, MAX_LEVELS * LEVEL_STACK, || {
            katex::render_to_string(context, value, settings)
        })
    };
    let result = match render(settings) {
        Ok(html) => Ok(html),
        Err(_) => {
            file.messages.push(message("parseerror"));
            let strict = std::mem::replace(
                &mut settings.strict,
                StrictSetting::Mode(StrictMode::Ignore),
            );
            settings.throw_on_error = false;
            let fallback = render(settings);
            settings.throw_on_error = true;
            // katex-rs words and styles a formula's error markup unlike KaTeX.js, which
            // `create_error` matches, so the error is thrown again to make it
            let fallback = match fallback {
                Ok(html) if html.starts_with("<span class=\"katex-error\"") => {
                    render(settings).map_err(|error| error.to_string())
                }
                fallback => fallback.map_err(|error| error.to_string()),
            };
            settings.strict = strict;
            fallback
        }
    };
    settings.max_expand = expand;
    match result {
        Ok(html) => parse_markup(&html, alloc),
        Err(error) => {
            create_error(value, &format!("ParseError: {error}"), settings, alloc)
        }
    }
}

// How many levels deep KaTeX may recurse on `value`, and the expansions it may then take
// so that macros nesting groups in each other stay within `MAX_LEVELS`
fn measure_levels(value: &str, settings: &Settings) -> Option<(usize, usize)> {
    let (depth, defines) = measure_depth(value);
    let levels = depth + 64;
    let room = MAX_LEVELS.checked_sub(levels)?;
    // `\gdef`s of earlier formulas stay in `macros`, as token expansions
    let macros = settings.macros.borrow();
    if !defines && macros.is_empty() {
        return Some((levels, settings.max_expand));
    }
    let body = macros
        .values()
        .filter_map(|definition| match definition {
            MacroDefinition::String(body) => Some(measure_depth(body).0),
            MacroDefinition::Expansion(expansion) => {
                let tokens: Vec<&str> = expansion
                    .tokens
                    .iter()
                    .rev()
                    .map(|token| &*token.text)
                    .collect();
                Some(measure_depth(&tokens.join(" ")).0)
            }
            _ => None,
        })
        .fold(depth, usize::max)
        .max(1);
    let max_expand = settings.max_expand.min(room / body);
    Some((levels + max_expand * body, max_expand))
}

#[derive(Clone, Copy, PartialEq)]
enum Group {
    Brace,
    Left,
    Begin,
}

// A bound on how deep KaTeX recurses on TeX source, and whether it defines macros: a group
// is four levels, as built-in macros wrap arguments in groups of their own, and each control
// word one more until its group closes, as `\displaystyle` takes the rest as its body. A
// `[` is a group no `]` closes, and a closer in a comment or `\verb` closes nothing
fn measure_depth(source: &str) -> (usize, bool) {
    let bytes = source.as_bytes();
    let mut groups: Vec<(Group, usize)> = Vec::new();
    let (mut level, mut deepest, mut defines) = (0usize, 0usize, false);
    let open = |groups: &mut Vec<(Group, usize)>, level: &mut usize, group| {
        groups.push((group, *level));
        *level += 4;
    };
    let close = |groups: &mut Vec<(Group, usize)>, level: &mut usize, group| {
        let Some(at) = groups.iter().rposition(|(open, _)| *open == group) else {
            return;
        };
        if group == Group::Brace || at + 1 == groups.len() {
            *level = groups[at].1;
            groups.truncate(at);
        }
    };
    // Up to where a closer may be in a comment or `\verb`: nothing is skipped, since KaTeX
    // reads `%` in `\url` literally, but closers there close nothing
    let mut hidden = 0;
    // Where a comment (to `\n`, as KaTeX's lexer reads it) or a `\verb` line (to `\n` or
    // `\r`) ends, found once per line
    let mut ends = [(0, 0); 2];
    let mut find_end = |from: usize, verb: bool| {
        let (start, end) = &mut ends[usize::from(verb)];
        if from >= *end || from < *start {
            let found = match verb {
                true => memchr::memchr2(b'\n', b'\r', &bytes[from..]),
                false => memchr::memchr(b'\n', &bytes[from..]),
            };
            (*start, *end) = (from, found.map_or(bytes.len(), |found| from + found));
        }
        *end
    };
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        let shown = index >= hidden;
        index += 1;
        match byte {
            b'{' => open(&mut groups, &mut level, Group::Brace),
            b'[' => level += 4,
            b'}' if shown => close(&mut groups, &mut level, Group::Brace),
            b'%' => hidden = hidden.max(find_end(index, false)),
            b'\\' => {
                let start = index;
                while bytes.get(index).is_some_and(u8::is_ascii_alphabetic) {
                    index += 1;
                }
                match &source[start..index] {
                    "left" => open(&mut groups, &mut level, Group::Left),
                    "begin" => open(&mut groups, &mut level, Group::Begin),
                    "bgroup" | "begingroup" => {
                        open(&mut groups, &mut level, Group::Brace)
                    }
                    "right" if shown => close(&mut groups, &mut level, Group::Left),
                    "end" if shown => close(&mut groups, &mut level, Group::Begin),
                    "egroup" | "endgroup" if shown => {
                        close(&mut groups, &mut level, Group::Brace)
                    }
                    "verb" => {
                        let after = index + usize::from(bytes.get(index) == Some(&b'*'));
                        if let Some(delimiter) = source[after..].chars().next() {
                            let from = after + delimiter.len_utf8();
                            let end = find_end(from, true);
                            if let Some(at) = source[from..end].find(delimiter) {
                                hidden = hidden.max(from + at + delimiter.len_utf8());
                            }
                        }
                        level += 1;
                    }
                    "" => {
                        index += source[index..].chars().next().map_or(0, char::len_utf8)
                    }
                    word => {
                        defines |= DEFINITIONS.contains(&word);
                        level += 1;
                    }
                }
            }
            _ => {}
        }
        deepest = deepest.max(level);
    }
    (deepest, defines)
}

// KaTeX's markup through the HTML parser, as rehype-katex's `fromHtmlIsomorphic`
fn parse_markup<'a>(html: &str, alloc: &'a Allocator) -> ArenaVec<'a, Node<'a>> {
    let parse = Parse {
        fragment: true,
        ..Default::default()
    };
    let Node::Root(root) = parse_html(html, &parse, &mut File::new(), alloc) else {
        return ArenaVec::new_in(alloc);
    };
    let mut children = root.children;
    let mut stack: Vec<&mut Node<'a>> = children.iter_mut().collect();
    while let Some(node) = stack.pop() {
        match node {
            Node::Element(element) => {
                element.position = None;
                sort_attributes(element);
                stack.extend(element.children.iter_mut());
            }
            Node::Text(text) => text.position = None,
            _ => {}
        }
    }
    children
}

// katex-rs keeps attributes in a randomly seeded hash map, so their order changes from
// run to run: `class`, then `style`, then the rest by name, whatever KaTeX's order was
fn sort_attributes(element: &mut Element<'_>) {
    element
        .properties
        .sort_by_key(|property| match property.name {
            "className" => (0, ""),
            "style" => (1, ""),
            name => (2, name),
        });
}

// The markup rehype-katex makes when KaTeX cannot render even its own error
fn create_error<'a>(
    value: &str,
    title: &str,
    settings: &Settings,
    alloc: &'a Allocator,
) -> ArenaVec<'a, Node<'a>> {
    let mut classes = ArenaVec::with_capacity_in(1, alloc);
    classes.push(PropertyItem::String("katex-error"));
    let style = alloc.alloc_str(&format!("color:{}", settings.error_color));
    let mut properties = ArenaVec::with_capacity_in(3, alloc);
    properties.push(Property {
        name: "className",
        value: PropertyValue::List(classes),
    });
    properties.push(Property {
        name: "style",
        value: PropertyValue::String(style),
    });
    properties.push(Property {
        name: "title",
        value: PropertyValue::String(alloc.alloc_str(title)),
    });
    let mut children = ArenaVec::with_capacity_in(1, alloc);
    children.push(Node::Text(Text {
        value: alloc.alloc_str(value),
        position: None,
        data: None,
    }));
    let mut nodes = ArenaVec::with_capacity_in(1, alloc);
    nodes.push(Node::Element(Element {
        tag_name: "span",
        properties,
        children,
        position: None,
        data: None,
        content: None,
    }));
    nodes
}
