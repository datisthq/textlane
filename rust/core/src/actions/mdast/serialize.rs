use serde_json::{Map, Value};

use crate::actions::hast::serialize::{insert_fields, serialize_data};
use crate::actions::mdx::serialize::insert_mdx_jsx;
use crate::actions::position::serialize::serialize_position;
use crate::models::error::Error;
use crate::models::lines::Lines;
use crate::models::mdast::AlignKind;
use crate::models::mdast::Node;

// JSON values nest and drop recursively, so trees deeper than this fail instead of
// overflowing the stack; serde_json itself stops parsing at 128 levels
pub(crate) const MAX_JSON_DEPTH: usize = 1000;

/// Serialize an mdast node to the JSON unified produces; spans need the source's `lines`.
pub fn serialize_mdast(
    node: &Node<'_>,
    lines: Option<&Lines<'_>>,
) -> Result<Value, Error> {
    serialize_node(node, lines, 0)
}

fn serialize_node(
    node: &Node<'_>,
    lines: Option<&Lines<'_>>,
    depth: usize,
) -> Result<Value, Error> {
    if depth > MAX_JSON_DEPTH {
        return Err(Error::new(
            "tree-too-deep",
            format!("trees nest at most {MAX_JSON_DEPTH} levels in JSON"),
        ));
    }
    let mut object = Map::new();
    let kind = match node {
        Node::Root(_) => "root",
        Node::Paragraph(_) => "paragraph",
        Node::Heading(heading) => {
            object.insert("depth".into(), heading.depth.into());
            "heading"
        }
        Node::ThematicBreak(_) => "thematicBreak",
        Node::Emphasis(_) => "emphasis",
        Node::Strong(_) => "strong",
        Node::Break(_) => "break",
        Node::Code(code) => {
            object.insert("lang".into(), code.lang.into());
            object.insert("meta".into(), code.meta.into());
            object.insert("value".into(), code.value.into());
            "code"
        }
        Node::Blockquote(_) => "blockquote",
        Node::List(list) => {
            object.insert("ordered".into(), list.ordered.into());
            object.insert("start".into(), list.start.into());
            if let Some(spread) = list.spread {
                object.insert("spread".into(), spread.into());
            }
            "list"
        }
        Node::ListItem(item) => {
            if let Some(spread) = item.spread {
                object.insert("spread".into(), spread.into());
            }
            object.insert("checked".into(), item.checked.into());
            "listItem"
        }
        Node::Link(link) => {
            object.insert("url".into(), link.url.into());
            object.insert("title".into(), link.title.into());
            "link"
        }
        Node::Image(image) => {
            object.insert("url".into(), image.url.into());
            object.insert("title".into(), image.title.into());
            object.insert("alt".into(), image.alt.into());
            "image"
        }
        Node::LinkReference(reference) => {
            object.insert("identifier".into(), reference.identifier.into());
            object.insert("label".into(), reference.label.into());
            object.insert(
                "referenceType".into(),
                reference.reference_type.as_str().into(),
            );
            "linkReference"
        }
        Node::ImageReference(reference) => {
            object.insert("identifier".into(), reference.identifier.into());
            object.insert("label".into(), reference.label.into());
            object.insert(
                "referenceType".into(),
                reference.reference_type.as_str().into(),
            );
            object.insert("alt".into(), reference.alt.into());
            "imageReference"
        }
        Node::Definition(definition) => {
            object.insert("identifier".into(), definition.identifier.into());
            object.insert("label".into(), definition.label.into());
            object.insert("url".into(), definition.url.into());
            object.insert("title".into(), definition.title.into());
            "definition"
        }
        Node::Html(html) => {
            object.insert("value".into(), html.value.into());
            "html"
        }
        Node::InlineCode(code) => {
            object.insert("value".into(), code.value.into());
            "inlineCode"
        }
        Node::Text(text) => {
            object.insert("value".into(), text.value.into());
            "text"
        }
        Node::Delete(_) => "delete",
        Node::Table(table) => {
            let align = table
                .align
                .iter()
                .map(|align| match align {
                    AlignKind::Left => Value::from("left"),
                    AlignKind::Right => Value::from("right"),
                    AlignKind::Center => Value::from("center"),
                    AlignKind::None => Value::Null,
                })
                .collect();
            object.insert("align".into(), Value::Array(align));
            "table"
        }
        Node::TableRow(_) => "tableRow",
        Node::TableCell(_) => "tableCell",
        Node::FootnoteDefinition(definition) => {
            object.insert("identifier".into(), definition.identifier.into());
            object.insert("label".into(), definition.label.into());
            "footnoteDefinition"
        }
        Node::FootnoteReference(reference) => {
            object.insert("identifier".into(), reference.identifier.into());
            object.insert("label".into(), reference.label.into());
            "footnoteReference"
        }
        Node::DefList(_) => "defList",
        Node::DefListTerm(_) => "defListTerm",
        Node::DefListDescription(description) => {
            object.insert("spread".into(), description.spread.into());
            "defListDescription"
        }
        Node::Math(math) => {
            object.insert("meta".into(), math.meta.into());
            object.insert("value".into(), math.value.into());
            "math"
        }
        Node::InlineMath(math) => {
            object.insert("value".into(), math.value.into());
            "inlineMath"
        }
        Node::ContainerDirective(directive) => {
            insert_directive(&mut object, directive.name, &directive.attributes);
            "containerDirective"
        }
        Node::LeafDirective(directive) => {
            insert_directive(&mut object, directive.name, &directive.attributes);
            "leafDirective"
        }
        Node::TextDirective(directive) => {
            insert_directive(&mut object, directive.name, &directive.attributes);
            "textDirective"
        }
        Node::Yaml(yaml) => {
            object.insert("value".into(), yaml.value.into());
            "yaml"
        }
        Node::Toml(toml) => {
            object.insert("value".into(), toml.value.into());
            "toml"
        }
        Node::Literal(literal) => {
            object.insert("value".into(), literal.value.into());
            literal.kind
        }
        Node::Unknown(unknown) => {
            insert_fields(&mut object, unknown.fields);
            unknown.kind
        }
        Node::MdxJsxFlowElement(element) => {
            insert_mdx_jsx(&mut object, element.name, &element.attributes, lines);
            "mdxJsxFlowElement"
        }
        Node::MdxJsxTextElement(element) => {
            insert_mdx_jsx(&mut object, element.name, &element.attributes, lines);
            "mdxJsxTextElement"
        }
        Node::MdxFlowExpression(expression) => {
            object.insert("value".into(), expression.value.into());
            "mdxFlowExpression"
        }
        Node::MdxTextExpression(expression) => {
            object.insert("value".into(), expression.value.into());
            "mdxTextExpression"
        }
        Node::MdxjsEsm(esm) => {
            object.insert("value".into(), esm.value.into());
            "mdxjsEsm"
        }
    };
    object.insert("type".into(), kind.into());
    if let Some(children) = node.children() {
        let mut children: Vec<Value> = children
            .iter()
            .map(|child| serialize_node(child, lines, depth + 1))
            .collect::<Result<_, _>>()?;
        if let Node::ContainerDirective(directive) = node
            && directive.label
            && let Some(Value::Object(label)) = children.first_mut()
        {
            let data = label
                .entry("data")
                .or_insert_with(|| Value::Object(Map::new()));
            if let Value::Object(data) = data {
                data.insert("directiveLabel".into(), true.into());
            }
        }
        object.insert("children".into(), Value::Array(children));
    }
    if let Some(data) = node.data() {
        object.insert("data".into(), serialize_data(data));
    }
    serialize_position(&mut object, node.position(), lines);
    Ok(Value::Object(object))
}

fn insert_directive(
    object: &mut Map<String, Value>,
    name: &str,
    attributes: &[(&str, &str)],
) {
    object.insert("name".into(), name.into());
    let mut map = Map::new();
    for (key, value) in attributes {
        map.insert((*key).into(), (*value).into());
    }
    object.insert("attributes".into(), Value::Object(map));
}
