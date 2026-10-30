use std::collections::{BTreeMap, HashMap, HashSet};

use textlane_core::hast::{
    Comment, Doctype, Element, Node, Property, PropertyItem, PropertyValue, Root, Text,
};
use textlane_core::{Allocator, ArenaVec};

use crate::models::options::{AttributeDefinition, Primitive, Sanitize};

/// Keep only what a schema allows of a hast tree, as hast-util-sanitize does.
pub fn sanitize_hast<'a>(
    node: &Node<'a>,
    options: &Sanitize,
    alloc: &'a Allocator,
) -> Node<'a> {
    let schema = Schema::from_options(options);
    let mut sanitizer = Sanitizer {
        schema: &schema,
        alloc,
        ancestors: HashMap::new(),
    };
    match sanitizer.transform(node) {
        Output::One(node) => node,
        Output::Many(mut nodes) if nodes.len() == 1 => {
            nodes.pop().unwrap_or_else(|| Node::Root(empty_root(alloc)))
        }
        Output::Many(nodes) => {
            let mut children = ArenaVec::with_capacity_in(nodes.len(), alloc);
            children.extend(nodes);
            Node::Root(Root {
                children,
                ..empty_root(alloc)
            })
        }
        Output::None => Node::Root(empty_root(alloc)),
    }
}

fn empty_root(alloc: &Allocator) -> Root<'_> {
    Root {
        children: ArenaVec::new_in(alloc),
        position: None,
        data: None,
        quirks_mode: None,
    }
}

// What a sanitized node becomes: itself, the content of an unwrapped element, or nothing
enum Output<'a> {
    None,
    One(Node<'a>),
    Many(Vec<Node<'a>>),
}

#[derive(Clone, Debug)]
enum Allowed {
    Value(Primitive),
    // hast-util-sanitize's default `/^language-./` for `code` classes
    Language,
}

#[derive(Clone, Debug)]
struct Definition {
    name: String,
    allowed: Vec<Allowed>,
}

struct Schema {
    allow_comments: bool,
    allow_doctypes: bool,
    ancestors: HashMap<String, Vec<String>>,
    attributes: HashMap<String, Vec<Definition>>,
    clobber: Vec<String>,
    clobber_prefix: String,
    protocols: HashMap<String, Vec<String>>,
    required: HashMap<String, BTreeMap<String, Primitive>>,
    strip: Option<Vec<String>>,
    tag_names: Option<HashSet<String>>,
}

impl Schema {
    fn from_options(options: &Sanitize) -> Schema {
        let owned = |list: &[&str]| list.iter().map(|item| item.to_string()).collect();
        let attributes = match &options.attributes {
            Some(attributes) => attributes
                .iter()
                .map(|(tag, definitions)| {
                    (
                        tag.clone(),
                        definitions.iter().map(read_definition).collect(),
                    )
                })
                .collect(),
            None => create_default_attributes(),
        };
        Schema {
            allow_comments: options.allow_comments.unwrap_or(false),
            allow_doctypes: options.allow_doctypes.unwrap_or(false),
            ancestors: options.ancestors.clone().unwrap_or_else(|| {
                ["tbody", "td", "th", "thead", "tfoot", "tr"]
                    .iter()
                    .map(|tag| (tag.to_string(), vec!["table".to_string()]))
                    .collect()
            }),
            attributes,
            clobber: options.clobber.clone().unwrap_or_else(|| {
                owned(&["ariaDescribedBy", "ariaLabelledBy", "id", "name"])
            }),
            clobber_prefix: options
                .clobber_prefix
                .clone()
                .unwrap_or_else(|| "user-content-".to_string()),
            protocols: options.protocols.clone().unwrap_or_else(|| {
                let web = || owned(&["http", "https"]);
                HashMap::from([
                    ("cite".to_string(), web()),
                    (
                        "href".to_string(),
                        owned(&["http", "https", "irc", "ircs", "mailto", "xmpp"]),
                    ),
                    ("longDesc".to_string(), web()),
                    ("src".to_string(), web()),
                ])
            }),
            required: options.required.clone().unwrap_or_else(|| {
                HashMap::from([(
                    "input".to_string(),
                    BTreeMap::from([
                        ("disabled".to_string(), Primitive::Boolean(true)),
                        (
                            "type".to_string(),
                            Primitive::String("checkbox".to_string()),
                        ),
                    ]),
                )])
            }),
            strip: Some(options.strip.clone().unwrap_or_else(|| owned(&["script"]))),
            tag_names: Some(
                options
                    .tag_names
                    .clone()
                    .unwrap_or_else(|| owned(&DEFAULT_TAG_NAMES))
                    .into_iter()
                    .collect(),
            ),
        }
    }
}

fn read_definition(definition: &AttributeDefinition) -> Definition {
    match definition {
        AttributeDefinition::Name(name) => Definition {
            name: name.clone(),
            allowed: Vec::new(),
        },
        AttributeDefinition::Values(values) => {
            let name = match values.first() {
                Some(Primitive::String(name)) => name.clone(),
                _ => String::new(),
            };
            let allowed = values.iter().skip(1).cloned().map(Allowed::Value).collect();
            Definition { name, allowed }
        }
    }
}

struct Sanitizer<'s, 'a> {
    schema: &'s Schema,
    alloc: &'a Allocator,
    // Open element names, counted, for the `ancestors` rule
    ancestors: HashMap<&'a str, usize>,
}

struct Frame<'n, 'a> {
    node: &'n Node<'a>,
    next: usize,
    children: Vec<Node<'a>>,
}

impl<'n, 'a> Sanitizer<'_, 'a> {
    // Children are sanitized before their element is judged, as upstream does, so the
    // content of a dropped element can be kept; the walk keeps its own stack
    fn transform(&mut self, root: &'n Node<'a>) -> Output<'a> {
        let mut frames: Vec<Frame<'n, 'a>> = Vec::new();
        let mut result = self.enter(root, &mut frames);
        while let Some(frame) = frames.last_mut() {
            let children: &'n [Node<'a>] = match frame.node {
                Node::Root(root) => &root.children,
                Node::Element(element) => &element.children,
                _ => &[],
            };
            if let Some(child) = children.get(frame.next) {
                frame.next += 1;
                let output = self.enter(child, &mut frames);
                if let Some(frame) = frames.last_mut() {
                    push_output(&mut frame.children, output);
                }
                continue;
            }
            let Some(frame) = frames.pop() else {
                break;
            };
            let output = self.leave(frame.node, frame.children);
            match frames.last_mut() {
                Some(parent) => push_output(&mut parent.children, output),
                None => result = output,
            }
        }
        result
    }

    // A leaf's output, or `Output::None` with a frame pushed for a parent
    fn enter(
        &mut self,
        node: &'n Node<'a>,
        frames: &mut Vec<Frame<'n, 'a>>,
    ) -> Output<'a> {
        match node {
            Node::Element(element) => {
                *self.ancestors.entry(element.tag_name).or_default() += 1;
                frames.push(Frame {
                    node,
                    next: 0,
                    children: Vec::new(),
                });
                Output::None
            }
            Node::Root(_) => {
                frames.push(Frame {
                    node,
                    next: 0,
                    children: Vec::new(),
                });
                Output::None
            }
            Node::Text(text) => Output::One(Node::Text(Text {
                value: text.value,
                position: text.position,
                data: text.data,
            })),
            Node::Comment(comment) if self.schema.allow_comments => {
                let value = comment
                    .value
                    .find("-->")
                    .map_or(comment.value, |end| &comment.value[..end]);
                Output::One(Node::Comment(Comment {
                    value,
                    position: comment.position,
                    data: comment.data,
                }))
            }
            Node::Doctype(doctype) if self.schema.allow_doctypes => {
                Output::One(Node::Doctype(Doctype {
                    position: doctype.position,
                    data: doctype.data,
                }))
            }
            _ => Output::None,
        }
    }

    fn leave(&mut self, node: &'n Node<'a>, content: Vec<Node<'a>>) -> Output<'a> {
        let mut children = ArenaVec::with_capacity_in(content.len(), self.alloc);
        match node {
            Node::Root(root) => {
                children.extend(content);
                Output::One(Node::Root(Root {
                    children,
                    position: root.position,
                    data: root.data,
                    quirks_mode: root.quirks_mode,
                }))
            }
            Node::Element(element) => {
                let name = element.tag_name;
                if let Some(count) = self.ancestors.get_mut(name) {
                    *count -= 1;
                }
                let properties = self.sanitize_properties(element);
                if !self.is_safe_element(name) {
                    return match &self.schema.strip {
                        Some(strip) if !strip.iter().any(|each| each == name) => {
                            Output::Many(content)
                        }
                        _ => Output::None,
                    };
                }
                children.extend(content);
                Output::One(Node::Element(Element {
                    tag_name: name,
                    properties,
                    children,
                    position: element.position,
                    content: None,
                    data: element.data,
                }))
            }
            _ => Output::None,
        }
    }

    fn is_safe_element(&self, name: &str) -> bool {
        if name.is_empty()
            || name == "*"
            || self
                .schema
                .tag_names
                .as_ref()
                .is_some_and(|names| !names.contains(name))
        {
            return false;
        }
        match self.schema.ancestors.get(name) {
            Some(ancestors) => ancestors.iter().any(|ancestor| {
                self.ancestors
                    .get(ancestor.as_str())
                    .is_some_and(|count| *count > 0)
            }),
            None => true,
        }
    }

    fn sanitize_properties(&self, element: &Element<'a>) -> ArenaVec<'a, Property<'a>> {
        let tag = element.tag_name;
        let specific = self.schema.attributes.get(tag);
        let defaults = self.schema.attributes.get("*");
        let mut result = ArenaVec::new_in(self.alloc);
        for property in element.properties.iter() {
            let key = property.name;
            let value = self
                .sanitize_value(find_definition(specific, key), key, &property.value)
                .or_else(|| {
                    self.sanitize_value(
                        find_definition(defaults, key),
                        key,
                        &property.value,
                    )
                });
            if let Some(value) = value {
                result.push(Property { name: key, value });
            }
        }
        if let Some(required) = self.schema.required.get(tag) {
            for (key, value) in required {
                if !result.iter().any(|property| property.name == key) {
                    result.push(Property {
                        name: self.alloc.alloc_str(key),
                        value: self.create_value(value),
                    });
                }
            }
        }
        result
    }

    fn sanitize_value(
        &self,
        definition: Option<&Definition>,
        key: &str,
        value: &PropertyValue<'a>,
    ) -> Option<PropertyValue<'a>> {
        let definition = definition?;
        match value {
            PropertyValue::List(items) => {
                let mut list = ArenaVec::new_in(self.alloc);
                for item in items.iter() {
                    let primitive = match item {
                        PropertyItem::Number(number) => PropertyValue::Number(*number),
                        PropertyItem::String(text) => PropertyValue::String(text),
                    };
                    match self.sanitize_primitive(definition, key, &primitive) {
                        Some(PropertyValue::Number(number)) => {
                            list.push(PropertyItem::Number(number))
                        }
                        Some(PropertyValue::String(text)) => {
                            list.push(PropertyItem::String(text))
                        }
                        _ => {}
                    }
                }
                Some(PropertyValue::List(list))
            }
            _ => self.sanitize_primitive(definition, key, value),
        }
    }

    fn sanitize_primitive(
        &self,
        definition: &Definition,
        key: &str,
        value: &PropertyValue<'a>,
    ) -> Option<PropertyValue<'a>> {
        let text = match value {
            PropertyValue::Boolean(value) => value.to_string(),
            PropertyValue::Number(value) => {
                crate::actions::hast::stringify::write_js_number(*value)
            }
            PropertyValue::String(value) => value.to_string(),
            PropertyValue::Null | PropertyValue::List(_) => return None,
        };
        if !self.is_safe_protocol(key, &text) {
            return None;
        }
        if !definition.allowed.is_empty()
            && !definition.allowed.iter().any(|allowed| match allowed {
                Allowed::Language => text
                    .strip_prefix("language-")
                    .and_then(|rest| rest.chars().next())
                    .is_some_and(|next| {
                        !matches!(next, '\n' | '\r' | '\u{2028}' | '\u{2029}')
                    }),
                Allowed::Value(allowed) => is_same(allowed, value),
            })
        {
            return None;
        }
        if !self.schema.clobber_prefix.is_empty()
            && self.schema.clobber.iter().any(|name| name == key)
        {
            let prefixed = format!("{}{text}", self.schema.clobber_prefix);
            return Some(PropertyValue::String(self.alloc.alloc_str(&prefixed)));
        }
        Some(match value {
            PropertyValue::Boolean(value) => PropertyValue::Boolean(*value),
            PropertyValue::Number(value) => PropertyValue::Number(*value),
            PropertyValue::String(value) => PropertyValue::String(value),
            _ => PropertyValue::Null,
        })
    }

    // A URL with a scheme must use one the schema lists for the property
    fn is_safe_protocol(&self, key: &str, url: &str) -> bool {
        let Some(protocols) = self
            .schema
            .protocols
            .get(key)
            .filter(|list| !list.is_empty())
        else {
            return true;
        };
        let units: Vec<u16> = url.encode_utf16().collect();
        let find = |unit: u8| units.iter().position(|each| *each == u16::from(unit));
        let Some(colon) = find(b':') else {
            return true;
        };
        let before = |unit: u8| find(unit).is_some_and(|index| colon > index);
        if before(b'/') || before(b'?') || before(b'#') {
            return true;
        }
        protocols.iter().any(|protocol| {
            let protocol: Vec<u16> = protocol.encode_utf16().collect();
            colon == protocol.len() && units[..colon] == protocol[..]
        })
    }

    fn create_value(&self, value: &Primitive) -> PropertyValue<'a> {
        match value {
            Primitive::Boolean(value) => PropertyValue::Boolean(*value),
            Primitive::Number(value) => PropertyValue::Number(*value),
            Primitive::String(value) => {
                PropertyValue::String(self.alloc.alloc_str(value))
            }
        }
    }
}

fn push_output<'a>(children: &mut Vec<Node<'a>>, output: Output<'a>) {
    match output {
        Output::None => {}
        Output::One(node) => children.push(node),
        Output::Many(nodes) => children.extend(nodes),
    }
}

fn is_same(allowed: &Primitive, value: &PropertyValue) -> bool {
    match (allowed, value) {
        (Primitive::Boolean(allowed), PropertyValue::Boolean(value)) => allowed == value,
        (Primitive::Number(allowed), PropertyValue::Number(value)) => allowed == value,
        (Primitive::String(allowed), PropertyValue::String(value)) => allowed == value,
        _ => false,
    }
}

// The first definition named `key`, else, for a `data*` name, the `data*` one
fn find_definition<'d>(
    definitions: Option<&'d Vec<Definition>>,
    key: &str,
) -> Option<&'d Definition> {
    let definitions = definitions?;
    if let Some(found) = definitions.iter().find(|definition| definition.name == key) {
        return Some(found);
    }
    let is_data = key.len() > 4 && key.as_bytes()[..4].eq_ignore_ascii_case(b"data");
    if is_data {
        return definitions
            .iter()
            .find(|definition| definition.name == "data*");
    }
    None
}

const DEFAULT_TAG_NAMES: [&str; 53] = [
    "a",
    "b",
    "blockquote",
    "br",
    "code",
    "dd",
    "del",
    "details",
    "div",
    "dl",
    "dt",
    "em",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "hr",
    "i",
    "img",
    "input",
    "ins",
    "kbd",
    "li",
    "ol",
    "p",
    "picture",
    "pre",
    "q",
    "rp",
    "rt",
    "ruby",
    "s",
    "samp",
    "section",
    "source",
    "span",
    "strike",
    "strong",
    "sub",
    "summary",
    "sup",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "tt",
    "ul",
    "var",
];

fn create_default_attributes() -> HashMap<String, Vec<Definition>> {
    let name = |name: &str| Definition {
        name: name.to_string(),
        allowed: Vec::new(),
    };
    let class = |value: &str| Definition {
        name: "className".to_string(),
        allowed: vec![Allowed::Value(Primitive::String(value.to_string()))],
    };
    let aria = || {
        vec![
            name("ariaDescribedBy"),
            name("ariaLabel"),
            name("ariaLabelledBy"),
        ]
    };
    let with_aria = |mut more: Vec<Definition>| {
        let mut list = aria();
        list.append(&mut more);
        list
    };
    let everywhere = [
        "abbr",
        "accept",
        "acceptCharset",
        "accessKey",
        "action",
        "align",
        "alt",
        "axis",
        "border",
        "cellPadding",
        "cellSpacing",
        "char",
        "charOff",
        "charSet",
        "checked",
        "clear",
        "colSpan",
        "color",
        "cols",
        "compact",
        "coords",
        "dateTime",
        "dir",
        "encType",
        "frame",
        "hSpace",
        "headers",
        "height",
        "hrefLang",
        "htmlFor",
        "id",
        "isMap",
        "itemProp",
        "label",
        "lang",
        "maxLength",
        "media",
        "method",
        "multiple",
        "name",
        "noHref",
        "noShade",
        "noWrap",
        "open",
        "prompt",
        "readOnly",
        "rev",
        "rowSpan",
        "rows",
        "rules",
        "scope",
        "selected",
        "shape",
        "size",
        "span",
        "start",
        "summary",
        "tabIndex",
        "title",
        "useMap",
        "vAlign",
        "value",
        "width",
    ];
    HashMap::from([
        (
            "a".to_string(),
            with_aria(vec![
                name("dataFootnoteBackref"),
                name("dataFootnoteRef"),
                class("data-footnote-backref"),
                name("href"),
            ]),
        ),
        ("blockquote".to_string(), vec![name("cite")]),
        (
            "code".to_string(),
            vec![Definition {
                name: "className".to_string(),
                allowed: vec![Allowed::Language],
            }],
        ),
        ("del".to_string(), vec![name("cite")]),
        ("div".to_string(), vec![name("itemScope"), name("itemType")]),
        ("dl".to_string(), aria()),
        ("h2".to_string(), vec![class("sr-only")]),
        (
            "img".to_string(),
            with_aria(vec![name("longDesc"), name("src")]),
        ),
        (
            "input".to_string(),
            vec![
                Definition {
                    name: "disabled".to_string(),
                    allowed: vec![Allowed::Value(Primitive::Boolean(true))],
                },
                Definition {
                    name: "type".to_string(),
                    allowed: vec![Allowed::Value(Primitive::String(
                        "checkbox".to_string(),
                    ))],
                },
            ],
        ),
        ("ins".to_string(), vec![name("cite")]),
        ("li".to_string(), vec![class("task-list-item")]),
        (
            "ol".to_string(),
            with_aria(vec![class("contains-task-list")]),
        ),
        ("q".to_string(), vec![name("cite")]),
        (
            "section".to_string(),
            vec![name("dataFootnotes"), class("footnotes")],
        ),
        ("source".to_string(), vec![name("srcSet")]),
        ("summary".to_string(), aria()),
        ("table".to_string(), aria()),
        (
            "ul".to_string(),
            with_aria(vec![class("contains-task-list")]),
        ),
        (
            "*".to_string(),
            everywhere.iter().map(|each| name(each)).collect(),
        ),
    ])
}
