use textlane_core::Error;
use textlane_core::mdast::{MdxJsxAttribute, MdxJsxAttributeValue, MdxJsxElement, Node};

use crate::models::options::Mdx;
use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::sink::Layer;
use crate::utils::serializer::state::{Info, Parent, Serializer};

const INDENT: &str = "  ";

/// Write a JSX element as mdast-util-mdx-jsx does: a flow element's attributes go on lines
/// of their own past `printWidth`, its children indented by how deep it nests.
pub(crate) fn stringify_mdx_jsx_element(
    state: &mut Serializer,
    node: &Node,
) -> Result<(), Error> {
    let (element, flow) = match node {
        Node::MdxJsxFlowElement(element) => (element, true),
        Node::MdxJsxTextElement(element) => (element, false),
        _ => return Ok(()),
    };
    let options = state.settings.mdx.clone().unwrap_or_default();
    let (quote, alternative) = read_quotes(&options);
    let name = element.name.unwrap_or("");
    let self_closing = !name.is_empty() && element.children.is_empty();
    let depth = infer_depth(state);
    let current_indent = INDENT.repeat(depth);
    let prefix = format!("{}<{name}", if flow { current_indent.as_str() } else { "" });
    let construct = if flow {
        Construct::MdxJsxFlowElement
    } else {
        Construct::MdxJsxTextElement
    };
    let column = infer_column(state) + prefix.chars().count();
    state.enter(construct);
    let attributes = match serialize_attributes(element, quote, alternative, &options) {
        Ok(attributes) => attributes,
        Err(error) => {
            state.exit();
            return Err(error);
        }
    };
    let one_line = attributes.join(" ");
    let closing_size = match (self_closing, options.tight_self_closing) {
        (true, true) => 2,
        (true, false) => 3,
        (false, _) => 1,
    };
    let print_width = options.print_width.unwrap_or(f64::INFINITY);
    let own_lines = flow
        && (one_line.contains(['\n', '\r'])
            || (column + one_line.chars().count() + closing_size) as f64 > print_width);
    let mut value = prefix;
    if own_lines {
        value.push('\n');
        let lines: Vec<String> = attributes
            .iter()
            .map(|attribute| format!("{current_indent}{INDENT}{attribute}"))
            .collect();
        value.push_str(&lines.join("\n"));
        value.push('\n');
        value.push_str(&current_indent);
    } else if !one_line.is_empty() {
        value.push(' ');
        value.push_str(&one_line);
    }
    if self_closing {
        if !options.tight_self_closing && !own_lines {
            value.push(' ');
        }
        value.push('/');
    }
    value.push('>');
    if !element.children.is_empty() {
        if flow {
            value.push('\n');
            state.write(&value);
            value.clear();
            if let Err(error) = write_flow(state, node, element) {
                state.exit();
                return Err(error);
            }
            value.push('\n');
        } else {
            let phrasing = state.container_phrasing(
                Parent::of(node),
                Info {
                    before: ">",
                    after: "<",
                },
            );
            match phrasing {
                Ok(phrasing) => value.push_str(&phrasing),
                Err(error) => {
                    state.exit();
                    return Err(error);
                }
            }
        }
    }
    if !self_closing {
        if flow {
            value.push_str(&current_indent);
        }
        value.push_str("</");
        value.push_str(name);
        value.push('>');
    }
    state.exit();
    state.write(&value);
    Ok(())
}

fn serialize_attributes(
    element: &MdxJsxElement<Node>,
    quote: char,
    alternative: char,
    options: &Mdx,
) -> Result<Vec<String>, Error> {
    if !element.attributes.is_empty() && element.name.is_none_or(str::is_empty) {
        return Err(Error::new(
            "invalid-tree",
            "Cannot serialize fragment w/ attributes",
        ));
    }
    let mut results = Vec::with_capacity(element.attributes.len());
    for attribute in element.attributes.iter() {
        let result = match attribute {
            MdxJsxAttribute::Expression { value, .. } => format!("{{{value}}}"),
            MdxJsxAttribute::Attribute { name, value, .. } => {
                if name.is_empty() {
                    return Err(Error::new(
                        "invalid-tree",
                        "Cannot serialize attribute w/o name",
                    ));
                }
                match value {
                    None => name.to_string(),
                    Some(MdxJsxAttributeValue::Expression { value, .. }) => {
                        format!("{name}={{{value}}}")
                    }
                    Some(MdxJsxAttributeValue::Literal(value)) => {
                        let applied = if options.quote_smart
                            && value.matches(quote).count()
                                > value.matches(alternative).count()
                        {
                            alternative
                        } else {
                            quote
                        };
                        let encoded = value
                            .replace(applied, &format!("&#x{:X};", u32::from(applied)));
                        format!("{name}={applied}{encoded}{applied}")
                    }
                }
            }
        };
        results.push(result);
    }
    Ok(results)
}

// Children each go on their own, a blank line apart; all but JSX flow elements, which
// indent themselves, get the indent of this element's depth
fn write_flow(
    state: &mut Serializer,
    node: &Node,
    element: &MdxJsxElement<Node>,
) -> Result<(), Error> {
    let current_indent = INDENT.repeat(infer_depth(state));
    let parent = Parent::of(node);
    state.index_stack.push(0);
    for (index, child) in element.children.iter().enumerate() {
        if let Some(top) = state.index_stack.last_mut() {
            *top = index;
        }
        let info = Info {
            before: "\n",
            after: "\n",
        };
        let result = if matches!(child, Node::MdxJsxFlowElement(_)) {
            state.handle(child, Some(parent), info)
        } else {
            state.open_layer(Layer::Indent(current_indent.clone()));
            let result = state.handle(child, Some(parent), info);
            state.close_layer();
            result
        };
        if let Err(error) = result {
            state.index_stack.pop();
            return Err(error);
        }
        if !matches!(child, Node::List(_)) {
            state.bullet_last_used = None;
        }
        if index + 1 < element.children.len() {
            state.write("\n\n");
        }
    }
    state.index_stack.pop();
    Ok(())
}

// How many JSX flow elements this one is in, up to the nearest block quote or list item
fn infer_depth(state: &Serializer) -> usize {
    state
        .stack
        .iter()
        .rev()
        .take_while(|construct| {
            !matches!(construct, Construct::Blockquote | Construct::ListItem)
        })
        .filter(|construct| **construct == Construct::MdxJsxFlowElement)
        .count()
}

// The column a flow element starts at, as upstream's tracker counts it: one, shifted by
// each container around it (a list item as if bulleted `* `)
fn infer_column(state: &Serializer) -> usize {
    1 + state
        .stack
        .iter()
        .map(|construct| match construct {
            Construct::Blockquote
            | Construct::ListItem
            | Construct::MdxJsxFlowElement => 2,
            Construct::DefListDescription => 4,
            _ => 0,
        })
        .sum::<usize>()
}

/// The opening and closing tag of a JSX text element with children, `<a b="c">` and
/// `</a>`, its children going between them.
pub(crate) fn serialize_mdx_jsx_tags(
    state: &Serializer,
    element: &MdxJsxElement<Node>,
) -> Result<(String, String), Error> {
    let options = state.settings.mdx.clone().unwrap_or_default();
    let (quote, alternative) = read_quotes(&options);
    let attributes = serialize_attributes(element, quote, alternative, &options)?;
    let name = element.name.unwrap_or("");
    let mut open = format!("<{name}");
    if !attributes.is_empty() {
        open.push(' ');
        open.push_str(&attributes.join(" "));
    }
    open.push('>');
    Ok((open, format!("</{name}>")))
}

fn read_quotes(options: &Mdx) -> (char, char) {
    match options.quote.as_deref() {
        Some("'") => ('\'', '"'),
        _ => ('"', '\''),
    }
}
