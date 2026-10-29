use textlane_core::hast::{
    Element, MdxJsxAttribute, MdxJsxAttributeValue, Node, PropertyItem, PropertyValue,
};
use textlane_core::{Error, Lines, Position, UnistPoint};

use crate::actions::comment::attach::attach_comments;
use crate::actions::expression::read::read_expression;
use crate::actions::script::parse::parse_script;
use crate::helpers::identifier::is_identifier_name;
use crate::helpers::json::format_number;
use crate::helpers::property::{find_property, react_property};
use crate::helpers::style::parse_style;
use crate::models::estree::{Id, Store, Value};
use crate::models::name::{Key, Kind};
use crate::models::options::CompileOptions;

const STACK_RED_ZONE: usize = 64 * 1024;
const STACK_GROWTH: usize = 1024 * 1024;

/// One JavaScript source read from an MDX node: where its offsets start in the store and
/// where it starts in the document, so a node's place can be told in lines and columns.
pub(crate) struct Snippet {
    pub(crate) base: usize,
    pub(crate) value: String,
    pub(crate) start: Option<UnistPoint>,
}

/// An attribute of an MDX JSX element, as core's `MdxJsxAttribute` holds it.
pub(crate) enum JsxAttribute<'a> {
    Boolean(&'a str),
    Literal(&'a str, &'a str),
    Expression(&'a str, &'a str),
    Spread(&'a str),
}

/// hast-util-to-estree 3.1.3's state: the store the program is built in, the comments and
/// ESM collected on the way, and the snippets read.
pub(crate) struct Convert<'o, 's> {
    pub(crate) store: Store,
    pub(crate) comments: Vec<Id>,
    pub(crate) esm: Vec<Id>,
    pub(crate) snippets: Vec<Snippet>,
    options: &'o CompileOptions,
    lines: Option<Lines<'s>>,
    svg: bool,
    next_base: usize,
}

/// hast-util-to-estree: a hast tree into an ESTree program of its ESM and one JSX
/// expression statement; the program's `comments` are every comment met on the way.
pub(crate) fn convert_hast<'o, 's>(
    tree: &Node<'_>,
    source: Option<&'s str>,
    options: &'o CompileOptions,
) -> Result<(Convert<'o, 's>, Id), Error> {
    let mut state = Convert {
        store: Store::default(),
        comments: Vec::new(),
        esm: Vec::new(),
        snippets: Vec::new(),
        options,
        lines: source.map(Lines::new),
        svg: false,
        next_base: 2,
    };
    let mut body = Vec::new();
    if let Some(mut result) = state.handle(tree)? {
        let kind = state.store.kind(result);
        if kind != Kind::JSX_FRAGMENT && kind != Kind::JSX_ELEMENT {
            result = state.store.fragment(vec![result]);
        }
        body.push(state.store.expression_statement(result));
    }
    let mut statements: Vec<Id> = std::mem::take(&mut state.esm);
    statements.extend(body);
    let comments = Value::nodes(state.comments.clone());
    let program = state.store.add(
        Kind::PROGRAM,
        [
            (Key::BODY, Value::nodes(statements)),
            (Key::SOURCE_TYPE, "module".into()),
            (Key::COMMENTS, comments),
        ],
    );
    Ok((state, program))
}

impl Convert<'_, '_> {
    fn handle(&mut self, node: &Node<'_>) -> Result<Option<Id>, Error> {
        stacker::maybe_grow(STACK_RED_ZONE, STACK_GROWTH, || match node {
            Node::Root(root) => self.root(&root.children).map(Some),
            Node::Element(element) => self.element(element).map(Some),
            Node::Text(text) => Ok(self.text(text.value)),
            Node::Comment(comment) => Ok(Some(self.comment(comment.value))),
            Node::Doctype(_) => Ok(None),
            Node::Raw(_) => Err(unknown("raw")),
            Node::Unknown(unknown_node) => Err(unknown(unknown_node.kind)),
            Node::MdxJsxFlowElement(element) | Node::MdxJsxTextElement(element) => {
                let attributes: Vec<JsxAttribute<'_>> = element
                    .attributes
                    .iter()
                    .map(|attribute| match attribute {
                        MdxJsxAttribute::Attribute { name, value, .. } => match value {
                            None => JsxAttribute::Boolean(name),
                            Some(MdxJsxAttributeValue::Literal(value)) => {
                                JsxAttribute::Literal(name, value)
                            }
                            Some(MdxJsxAttributeValue::Expression { value, .. }) => {
                                JsxAttribute::Expression(name, value)
                            }
                        },
                        MdxJsxAttribute::Expression { value, .. } => {
                            JsxAttribute::Spread(value)
                        }
                    })
                    .collect();
                self.mdx_element(element.name, &attributes, &element.children)
                    .map(Some)
            }
            Node::MdxFlowExpression(expression) | Node::MdxTextExpression(expression) => {
                self.mdx_expression(expression.value).map(Some)
            }
            Node::MdxjsEsm(esm) => self
                .mdx_esm(esm.value, esm.position.as_ref())
                .map(|()| None),
        })
    }

    // `all`: line endings between table parts are dropped, as react-dom warns on them
    fn all(&mut self, children: &[Node<'_>], table: bool) -> Result<Vec<Id>, Error> {
        let mut results = Vec::with_capacity(children.len());
        for child in children {
            if table && matches!(child, Node::Text(text) if text.value == "\n") {
                continue;
            }
            if let Some(result) = self.handle(child)? {
                results.push(result);
            }
        }
        Ok(results)
    }

    fn root(&mut self, children: &[Node<'_>]) -> Result<Id, Error> {
        let children = self.all(children, false)?;
        let mut clean = Vec::with_capacity(children.len());
        let mut queue: Option<Vec<Id>> = None;
        for child in children {
            if self.is_whitespace_text(child) {
                if let Some(queue) = &mut queue {
                    queue.push(child);
                }
            } else {
                if let Some(queue) = queue.take() {
                    clean.extend(queue);
                }
                clean.push(child);
                queue = Some(Vec::new());
            }
        }
        Ok(self.store.fragment(clean))
    }

    // hast-util-whitespace: an expression container of a literal of only HTML white space
    fn is_whitespace_text(&self, id: Id) -> bool {
        self.store.is(id, Kind::JSX_EXPRESSION_CONTAINER)
            && self
                .store
                .child(id, Key::EXPRESSION)
                .is_some_and(|expression| {
                    self.store.is(expression, Kind::LITERAL)
                        && match self.store.get(expression, Key::VALUE) {
                            Some(Value::Str(value)) => value.bytes().all(|byte| {
                                matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b'\x0c')
                            }),
                            _ => false,
                        }
                })
    }

    fn text(&mut self, value: &str) -> Option<Id> {
        if value.is_empty() {
            return None;
        }
        let literal = self.store.literal(value);
        Some(self.store.add(
            Kind::JSX_EXPRESSION_CONTAINER,
            [(Key::EXPRESSION, literal.into())],
        ))
    }

    fn comment(&mut self, value: &str) -> Id {
        let comment = self.store.add(Kind::BLOCK, [(Key::VALUE, value.into())]);
        self.comments.push(comment);
        let attached = self.store.copy(comment);
        self.store.set(attached, Key::LEADING, false);
        self.store.set(attached, Key::TRAILING, true);
        let expression = self.store.add(
            Kind::JSX_EMPTY_EXPRESSION,
            [(Key::COMMENTS, Value::nodes([attached]))],
        );
        self.store.add(
            Kind::JSX_EXPRESSION_CONTAINER,
            [(Key::EXPRESSION, expression.into())],
        )
    }

    fn element(&mut self, node: &Element<'_>) -> Result<Id, Error> {
        let parent_svg = self.svg;
        if !parent_svg && node.tag_name.eq_ignore_ascii_case("svg") {
            self.svg = true;
        }
        let table = !self.svg
            && matches!(
                node.tag_name.to_ascii_lowercase().as_str(),
                "table" | "tbody" | "thead" | "tfoot" | "tr"
            );
        let children = self.all(&node.children, table)?;
        let mut attributes = Vec::new();
        let mut align = None;
        let mut style: Option<Id> = None;
        for property in &node.properties {
            let info = find_property(self.svg, property.name);
            let value = &property.value;
            let falsy = match value {
                PropertyValue::Null | PropertyValue::Boolean(false) => true,
                PropertyValue::Number(number) if number.is_nan() => true,
                PropertyValue::Number(number) => *number == 0.0,
                PropertyValue::String(text) => text.is_empty(),
                _ => false,
            };
            if matches!(value, PropertyValue::Null | PropertyValue::Boolean(false))
                || matches!(value, PropertyValue::Number(number) if number.is_nan())
                || (falsy && info.boolean)
            {
                continue;
            }
            let name = if self.options.element_attribute_name_case.as_deref()
                != Some("html")
                && info.space.is_some()
            {
                react_property(&info.property)
                    .map_or_else(|| info.property.to_string(), str::to_string)
            } else {
                info.attribute.to_string()
            };
            let text = match value {
                PropertyValue::List(items) => {
                    let items: Vec<String> = items
                        .iter()
                        .map(|item| match item {
                            PropertyItem::Number(number) => format_number(*number),
                            PropertyItem::String(text) => text.to_string(),
                        })
                        .collect();
                    Some(if info.comma_separated {
                        stringify_commas(&items)
                    } else {
                        items.join(" ").trim_matches(is_js_whitespace).to_string()
                    })
                }
                PropertyValue::String(text) => Some(text.to_string()),
                PropertyValue::Number(number) => Some(format_number(*number)),
                _ => None,
            };
            let attribute_value = if name == "style" {
                let declarations = parse_style(text.as_deref().unwrap_or_default())
                    .map_err(|_| {
                        Error::new(
                            "mdx-failed",
                            format!(
                                "Could not parse `style` attribute on `{}`",
                                node.tag_name
                            ),
                        )
                    })?;
                let css = self.options.style_property_name_case.as_deref() == Some("css");
                let mut properties = Vec::with_capacity(declarations.len());
                for (property, value) in declarations {
                    let key = if css { css_casing(&property) } else { property };
                    properties.push(self.style_property(&key, &value));
                }
                let object = self.store.object(properties);
                style = Some(object);
                Some(self.store.add(
                    Kind::JSX_EXPRESSION_CONTAINER,
                    [(Key::EXPRESSION, object.into())],
                ))
            } else if matches!(value, PropertyValue::Boolean(true)) {
                None
            } else if self.options.table_cell_align_to_style != Some(false)
                && matches!(node.tag_name, "td" | "th")
                && name == "align"
            {
                align = text;
                continue;
            } else {
                Some(self.store.literal(text.unwrap_or_default().as_str()))
            };
            attributes.push(self.attribute(&name, attribute_value));
        }
        if let Some(align) = align {
            let object = match style {
                Some(object) => object,
                None => {
                    let object = self.store.object(Vec::new());
                    let container = self.store.add(
                        Kind::JSX_EXPRESSION_CONTAINER,
                        [(Key::EXPRESSION, object.into())],
                    );
                    let name = self.store.jsx_identifier("style");
                    attributes.push(self.store.add(
                        Kind::JSX_ATTRIBUTE,
                        [(Key::NAME, name.into()), (Key::VALUE, container.into())],
                    ));
                    object
                }
            };
            let key = if self.options.style_property_name_case.as_deref() == Some("css") {
                css_casing("textAlign")
            } else {
                "textAlign".to_string()
            };
            let property = self.style_property(&key, &align);
            self.store
                .list_mut(object, Key::PROPERTIES)
                .push(property.into());
        }
        self.svg = parent_svg;
        Ok(self.jsx_element(node.tag_name, attributes, children, false))
    }

    fn style_property(&mut self, key: &str, value: &str) -> Id {
        let key = if is_identifier_name(key, false) {
            self.store.identifier(key)
        } else {
            self.store.literal(key)
        };
        let value = self.store.literal(value);
        self.store.property(key, value, false)
    }

    // A name JSX cannot spell becomes a spread of an object with that key
    fn attribute(&mut self, name: &str, value: Option<Id>) -> Id {
        if is_identifier_name(name, true) {
            let name = self.store.jsx_identifier(name);
            return self.store.add(
                Kind::JSX_ATTRIBUTE,
                [(Key::NAME, name.into()), (Key::VALUE, value.into())],
            );
        }
        let key = self.store.literal(name);
        let value = value.unwrap_or_else(|| self.store.literal(true));
        let property = self.store.property(key, value, false);
        let argument = self.store.object(vec![property]);
        self.store.add(
            Kind::JSX_SPREAD_ATTRIBUTE,
            [(Key::ARGUMENT, argument.into())],
        )
    }

    /// A JSX element named `name`, self-closing when it has no children; `explicit` marks
    /// JSX written in MDX, which `_components` never replaces.
    pub(crate) fn jsx_element(
        &mut self,
        name: &str,
        attributes: Vec<Id>,
        children: Vec<Id>,
        explicit: bool,
    ) -> Id {
        let opening_name = self.jsx_name(name);
        let opening = self.store.add(
            Kind::JSX_OPENING_ELEMENT,
            [
                (Key::ATTRIBUTES, Value::nodes(attributes)),
                (Key::NAME, opening_name.into()),
                (Key::SELF_CLOSING, children.is_empty().into()),
            ],
        );
        let closing = if children.is_empty() {
            None
        } else {
            let closing_name = self.jsx_name(name);
            Some(self.store.add(
                Kind::JSX_CLOSING_ELEMENT,
                [(Key::NAME, closing_name.into())],
            ))
        };
        let element = self.store.add(
            Kind::JSX_ELEMENT,
            [
                (Key::OPENING_ELEMENT, opening.into()),
                (Key::CLOSING_ELEMENT, closing.into()),
                (Key::CHILDREN, Value::nodes(children)),
            ],
        );
        if explicit {
            self.store.set(element, Key::EXPLICIT, true);
        }
        element
    }

    // `createJsxNameFromString`
    fn jsx_name(&mut self, name: &str) -> Id {
        if name.contains('.') {
            let mut parts = name.split('.');
            let mut node = self.store.jsx_identifier(parts.next().unwrap_or_default());
            for part in parts.take_while(|part| !part.is_empty()) {
                let property = self.store.jsx_identifier(part);
                node = self.store.add(
                    Kind::JSX_MEMBER_EXPRESSION,
                    [(Key::OBJECT, node.into()), (Key::PROPERTY, property.into())],
                );
            }
            return node;
        }
        if let Some((namespace, local)) = name.split_once(':') {
            let local = local.split(':').next().unwrap_or_default();
            let namespace = self.store.jsx_identifier(namespace);
            let local = self.store.jsx_identifier(local);
            return self.store.add(
                Kind::JSX_NAMESPACED_NAME,
                [
                    (Key::NAMESPACE, namespace.into()),
                    (Key::NAME, local.into()),
                ],
            );
        }
        self.store.jsx_identifier(name)
    }

    /// `mdxJsxFlowElement` and `mdxJsxTextElement`: JSX written in MDX, a fragment when it
    /// has no name; children first, then attributes, as upstream collects comments.
    pub(crate) fn mdx_element(
        &mut self,
        name: Option<&str>,
        attributes: &[JsxAttribute<'_>],
        children: &[Node<'_>],
    ) -> Result<Id, Error> {
        let parent_svg = self.svg;
        if !parent_svg && name.is_some_and(|name| name.eq_ignore_ascii_case("svg")) {
            self.svg = true;
        }
        let children = self.all(children, false)?;
        let mut results = Vec::with_capacity(attributes.len());
        for attribute in attributes {
            let result = match attribute {
                JsxAttribute::Boolean(name) => {
                    let name = self.jsx_name(name);
                    self.store.add(
                        Kind::JSX_ATTRIBUTE,
                        [(Key::NAME, name.into()), (Key::VALUE, Value::Null)],
                    )
                }
                JsxAttribute::Literal(name, value) => {
                    let name = self.jsx_name(name);
                    let value = self.store.literal(*value);
                    self.store.add(
                        Kind::JSX_ATTRIBUTE,
                        [(Key::NAME, name.into()), (Key::VALUE, value.into())],
                    )
                }
                JsxAttribute::Expression(name, source) => {
                    let expression = self.expression(source, false)?;
                    let value = self.store.add(
                        Kind::JSX_EXPRESSION_CONTAINER,
                        [(Key::EXPRESSION, expression.into())],
                    );
                    let name = self.jsx_name(name);
                    self.store.add(
                        Kind::JSX_ATTRIBUTE,
                        [(Key::NAME, name.into()), (Key::VALUE, value.into())],
                    )
                }
                JsxAttribute::Spread(source) => {
                    let argument = match self.parse_expression(source, true)? {
                        Some(argument) => argument,
                        None => self.store.object(Vec::new()),
                    };
                    self.store.add(
                        Kind::JSX_SPREAD_ATTRIBUTE,
                        [(Key::ARGUMENT, argument.into())],
                    )
                }
            };
            results.push(result);
        }
        self.svg = parent_svg;
        Ok(match name {
            Some(name) => self.jsx_element(name, results, children, true),
            None => self.store.fragment(children),
        })
    }

    /// `mdxFlowExpression` and `mdxTextExpression`: the expression in a container, or an
    /// empty one when the source holds only comments.
    pub(crate) fn mdx_expression(&mut self, source: &str) -> Result<Id, Error> {
        let first = self.store.len();
        let expression = self.expression(source, false)?;
        self.mark_explicit(first);
        Ok(self.store.add(
            Kind::JSX_EXPRESSION_CONTAINER,
            [(Key::EXPRESSION, expression.into())],
        ))
    }

    /// `mdxjsEsm`: its statements join the program's, its comments the program's comments.
    pub(crate) fn mdx_esm(
        &mut self,
        source: &str,
        position: Option<&Position<'_>>,
    ) -> Result<(), Error> {
        let base = self.reserve(source, position);
        let first = self.store.len();
        let (program, comments) = parse_script(&mut self.store, source, base)?;
        self.mark_explicit(first);
        self.comments.extend(comments.iter().copied());
        attach_comments(&mut self.store, program, &comments);
        self.esm.extend(self.store.children(program, Key::BODY));
        Ok(())
    }

    // remark-mark-and-unravel marks JSX in expressions and ESM as written by hand, but
    // not JSX in attribute values, which `_components` still replaces
    fn mark_explicit(&mut self, first: usize) {
        for index in first..self.store.len() {
            if self.store.is(Id(index), Kind::JSX_ELEMENT) {
                self.store.set(Id(index), Key::EXPLICIT, true);
            }
        }
    }

    fn expression(&mut self, source: &str, spread: bool) -> Result<Id, Error> {
        match self.parse_expression(source, spread)? {
            Some(expression) => Ok(expression),
            None => Ok(self.store.add(Kind::JSX_EMPTY_EXPRESSION, [])),
        }
    }

    // Upstream attaches an expression's comments to a program wrapping it, which spans
    // the whole source, so comments before or after the expression print nowhere
    fn parse_expression(
        &mut self,
        source: &str,
        spread: bool,
    ) -> Result<Option<Id>, Error> {
        let base = self.reserve(source, None);
        let (expression, comments) =
            read_expression(&mut self.store, source, base, spread)?;
        self.comments.extend(comments.iter().copied());
        let Some(expression) = expression else {
            return Ok(None);
        };
        let start = Value::Number(base as f64);
        let end = Value::Number((base + source.len()) as f64);
        let statement = self.store.add(
            Kind::EXPRESSION_STATEMENT,
            [
                (Key::EXPRESSION, expression.into()),
                (Key::START, start.clone()),
                (Key::END, end.clone()),
            ],
        );
        let program = self.store.add(
            Kind::PROGRAM,
            [
                (Key::BODY, Value::nodes([statement])),
                (Key::START, start),
                (Key::END, end),
            ],
        );
        attach_comments(&mut self.store, program, &comments);
        if !spread {
            return Ok(Some(expression));
        }
        Ok(self
            .store
            .children(expression, Key::PROPERTIES)
            .first()
            .copied()
            .filter(|property| self.store.is(*property, Kind::SPREAD_ELEMENT))
            .and_then(|property| self.store.child(property, Key::ARGUMENT)))
    }

    fn reserve(&mut self, source: &str, position: Option<&Position<'_>>) -> usize {
        let base = self.next_base;
        self.next_base += source.len() + 4;
        let start = position.and_then(|position| match position {
            Position::Span { start, .. } => self
                .lines
                .as_ref()
                .map(|lines| lines.to_unist_point(*start)),
            Position::Unist(position) => Some(position.start),
        });
        self.snippets.push(Snippet {
            base,
            value: source.to_string(),
            start,
        });
        base
    }
}

fn unknown(kind: &str) -> Error {
    Error::new("mdx-failed", format!("Cannot handle unknown node `{kind}`"))
}

// comma-separated-tokens' `stringify`
fn stringify_commas(items: &[String]) -> String {
    let mut items = items.to_vec();
    if items.last().is_some_and(String::is_empty) {
        items.push(String::new());
    }
    items.join(", ").trim_matches(is_js_whitespace).to_string()
}

// `textAlign` becomes `text-align`, and `msX` becomes `-ms-x`
fn css_casing(from: &str) -> String {
    let mut to = String::with_capacity(from.len() + 2);
    for character in from.chars() {
        if character.is_ascii_uppercase() {
            to.push('-');
            to.push(character.to_ascii_lowercase());
        } else {
            to.push(character);
        }
    }
    if to.starts_with("ms-") {
        to.insert(0, '-');
    }
    to
}

fn is_js_whitespace(character: char) -> bool {
    crate::utils::printer::state::is_js_whitespace(character)
}
