use textlane_core::Error;
use textlane_core::hast::{Element, Node, PropertyItem, PropertyValue};

use crate::helpers::omission::{Siblings, is_closing_omitted, is_opening_omitted};
use crate::helpers::property::find_property;
use crate::helpers::schema::{
    BOOLEAN, COMMA_SEPARATED, HTML, Info, OVERLOADED_BOOLEAN, SVG,
};
use crate::models::options::StringifyOptions;

// html-void-elements
const VOID_ELEMENTS: [&str; 20] = [
    "area", "base", "basefont", "bgsound", "br", "col", "command", "embed", "frame",
    "hr", "image", "img", "input", "keygen", "link", "meta", "param", "source", "track",
    "wbr",
];

// What to escape in attribute names, unquoted, single- and double-quoted values: indexed
// by whether parse errors are not allowed, then whether dangerous characters are not
const NAME: [[&str; 2]; 2] = [
    ["\t\n\x0C\r &/=>", "\t\n\x0C\r \"&'/=>`"],
    ["\0\t\n\x0C\r \"&'/<=>", "\0\t\n\x0C\r \"&'/<=>`"],
];
const UNQUOTED: [[&str; 2]; 2] = [
    ["\t\n\x0C\r &>", "\0\t\n\x0C\r \"&'<=>`"],
    ["\0\t\n\x0C\r \"&'<=>`", "\0\t\n\x0C\r \"&'<=>`"],
];
const SINGLE: [[&str; 2]; 2] = [["&'", "\"&'`"], ["\0&'", "\0\"&'`"]];
const DOUBLE: [[&str; 2]; 2] = [["\"&", "\"&'`"], ["\0\"&", "\0\"&'`"]];

/// Serialize hast as HTML, as hast-util-to-html does.
///
/// Walks with an explicit stack instead of recursing, so a deeply nested tree cannot
/// overflow the call stack.
pub fn stringify_hast(
    node: &Node<'_>,
    options: &StringifyOptions,
) -> Result<String, Error> {
    let quote = match options.quote.as_deref() {
        None | Some("" | "\"") => '"',
        Some("'") => '\'',
        Some(quote) => {
            return Err(Error::new(
                "invalid-options",
                format!("Invalid quote `{quote}`, expected `'` or `\"`"),
            ));
        }
    };
    let mut state = State {
        options,
        output: String::new(),
        schema: if options.space.as_deref() == Some("svg") {
            &SVG
        } else {
            &HTML
        },
        quote,
        alternative: if quote == '"' { '\'' } else { '"' },
    };
    let mut stack = vec![Step::Open {
        node,
        index: 0,
        parent: None,
    }];
    while let Some(step) = stack.pop() {
        match step {
            Step::Open {
                node,
                index,
                parent,
            } => state.open(node, index, parent, &mut stack),
            Step::Close(close) => state.close(close),
        }
    }
    Ok(state.output)
}

struct State<'o> {
    options: &'o StringifyOptions,
    output: String,
    schema: &'static [Info],
    quote: char,
    alternative: char,
}

enum Step<'n, 'a> {
    Open {
        node: &'n Node<'a>,
        index: usize,
        parent: Option<Siblings<'n, 'a>>,
    },
    Close(Close<'n, 'a>),
}

struct Close<'n, 'a> {
    element: &'n Element<'a>,
    index: usize,
    parent: Option<Siblings<'n, 'a>>,
    schema: &'static [Info],
    omit: bool,
    self_closing: bool,
    // Where the content starts, to tell an empty element, which closes itself
    content_start: usize,
    wrote_start_tag: bool,
    tight_slash: bool,
}

fn is_svg(schema: &'static [Info]) -> bool {
    std::ptr::eq(schema, &SVG[..])
}

impl<'o> State<'o> {
    fn open<'n, 'a>(
        &mut self,
        node: &'n Node<'a>,
        index: usize,
        parent: Option<Siblings<'n, 'a>>,
        stack: &mut Vec<Step<'n, 'a>>,
    ) {
        match node {
            Node::Root(root) => {
                let siblings = Siblings {
                    tag_name: None,
                    children: &root.children,
                };
                push_children(stack, siblings);
            }
            Node::Element(element) => self.open_element(element, index, parent, stack),
            Node::Text(text) => self.write_text(text.value, parent),
            Node::Raw(raw) => {
                if self.options.allow_dangerous_html {
                    self.output.push_str(raw.value);
                } else {
                    self.write_text(raw.value, parent);
                }
            }
            Node::Comment(comment) => self.write_comment(comment.value),
            Node::Doctype(_) => {
                self.output.push_str("<!");
                self.output.push_str(if self.options.upper_doctype {
                    "DOCTYPE"
                } else {
                    "doctype"
                });
                if !self.options.tight_doctype {
                    self.output.push(' ');
                }
                self.output.push_str("html>");
            }
        }
    }

    fn open_element<'n, 'a>(
        &mut self,
        element: &'n Element<'a>,
        index: usize,
        parent: Option<Siblings<'n, 'a>>,
        stack: &mut Vec<Step<'n, 'a>>,
    ) {
        let schema = self.schema;
        let svg = is_svg(schema);
        let omit = !svg && self.options.omit_optional_tags;
        let self_closing = if svg {
            self.options.close_empty_elements
        } else {
            let name = element.tag_name.to_lowercase();
            match &self.options.voids {
                Some(voids) => voids.contains(&name),
                None => VOID_ELEMENTS.contains(&name.as_str()),
            }
        };
        if !svg && element.tag_name == "svg" {
            self.schema = &SVG;
        }
        let attributes = self.serialize_attributes(element);
        let mut wrote_start_tag = false;
        let mut tight_slash = false;
        if !attributes.is_empty() || !omit || !is_opening_omitted(element, index, parent)
        {
            self.output.push('<');
            self.output.push_str(element.tag_name);
            if !attributes.is_empty() {
                self.output.push(' ');
                self.output.push_str(&attributes);
            }
            let last = attributes.chars().next_back();
            tight_slash = self.options.tight_self_closing
                && last != Some('/')
                && last.is_none_or(|last| last == '"' || last == '\'');
            self.output.push('>');
            wrote_start_tag = true;
        }
        let children = match element.content {
            Some(content) if !svg && element.tag_name == "template" => &content.children,
            _ => &element.children,
        };
        stack.push(Step::Close(Close {
            element,
            index,
            parent,
            schema,
            omit,
            self_closing,
            content_start: self.output.len(),
            wrote_start_tag,
            tight_slash,
        }));
        push_children(
            stack,
            Siblings {
                tag_name: Some(element.tag_name),
                children,
            },
        );
    }

    fn close(&mut self, close: Close) {
        let self_closing = close.self_closing && self.output.len() == close.content_start;
        if self_closing
            && close.wrote_start_tag
            && (is_svg(close.schema) || self.options.close_self_closing)
        {
            self.output.pop();
            if !close.tight_slash {
                self.output.push(' ');
            }
            self.output.push_str("/>");
        }
        self.schema = close.schema;
        if !self_closing
            && (!close.omit
                || !is_closing_omitted(close.element, close.index, close.parent))
        {
            self.output.push_str("</");
            self.output.push_str(close.element.tag_name);
            self.output.push('>');
        }
    }

    fn write_text(&mut self, value: &str, parent: Option<Siblings>) {
        if parent
            .and_then(|parent| parent.tag_name)
            .is_some_and(|name| name == "script" || name == "style")
        {
            self.output.push_str(value);
        } else {
            push_escaped(&mut self.output, value, "<&");
        }
    }

    fn write_comment(&mut self, value: &str) {
        if self.options.bogus_comments {
            self.output.push_str("<?");
            push_escaped(&mut self.output, value, ">");
            self.output.push('>');
            return;
        }
        self.output.push_str("<!--");
        // `/^>|^->|<!--|-->|--!>|<!-$/g`
        let mut rest = value;
        let mut at_start = true;
        while !rest.is_empty() {
            let found = if at_start && rest.starts_with('>') {
                Some(1)
            } else if at_start && rest.starts_with("->") {
                Some(2)
            } else if rest.starts_with("<!--") || rest.starts_with("--!>") {
                Some(4)
            } else if rest.starts_with("-->") || rest == "<!-" {
                Some(3)
            } else {
                None
            };
            match found {
                Some(size) => {
                    push_escaped(&mut self.output, &rest[..size], "<>");
                    rest = &rest[size..];
                }
                None => {
                    let character = rest.chars().next().unwrap_or_default();
                    self.output.push(character);
                    rest = &rest[character.len_utf8()..];
                }
            }
            at_start = false;
        }
        self.output.push_str("-->");
    }

    fn serialize_attributes(&self, element: &Element) -> String {
        let values: Vec<String> = element
            .properties
            .iter()
            .filter(|property| !matches!(property.value, PropertyValue::Null))
            .map(|property| self.serialize_attribute(property.name, &property.value))
            .filter(|value| !value.is_empty())
            .collect();
        let mut result = String::new();
        for (index, value) in values.iter().enumerate() {
            result.push_str(value);
            let last = value.chars().next_back();
            let quoted =
                self.options.tight_attributes && matches!(last, Some('"' | '\''));
            if index + 1 != values.len() && !quoted {
                result.push(' ');
            }
        }
        result
    }

    fn serialize_attribute(&self, key: &str, value: &PropertyValue) -> String {
        let found = find_property(self.schema, key);
        let x = usize::from(!(self.options.allow_parse_errors && !is_svg(self.schema)));
        let y = usize::from(!self.options.allow_dangerous_characters);
        let attribute = found.attribute.as_ref();
        let is_bool = found.kinds & (BOOLEAN | OVERLOADED_BOOLEAN) != 0;
        let value: Value = match value {
            PropertyValue::String(text)
                if found.kinds & OVERLOADED_BOOLEAN != 0
                    && (*text == attribute || text.is_empty()) =>
            {
                Value::Boolean(true)
            }
            PropertyValue::String(text)
                if is_bool && (*text == attribute || text.is_empty()) =>
            {
                Value::Boolean(!text.is_empty())
            }
            PropertyValue::Number(number) if is_bool => {
                Value::Boolean(*number != 0.0 && !number.is_nan())
            }
            PropertyValue::List(_) if is_bool => Value::Boolean(true),
            PropertyValue::Null => return String::new(),
            PropertyValue::Boolean(boolean) => Value::Boolean(*boolean),
            PropertyValue::Number(number) if number.is_nan() => return String::new(),
            PropertyValue::Number(number) => Value::Text(write_js_number(*number)),
            PropertyValue::String(text) => Value::Text((*text).to_string()),
            PropertyValue::List(items) => {
                let items: Vec<String> = items
                    .iter()
                    .map(|item| match item {
                        PropertyItem::Number(number) => write_js_number(*number),
                        PropertyItem::String(text) => (*text).to_string(),
                    })
                    .collect();
                Value::Text(if found.kinds & COMMA_SEPARATED != 0 {
                    join_commas(&items, !self.options.tight_comma_separated_lists)
                } else {
                    items.join(" ").trim_matches(is_js_space).to_string()
                })
            }
        };
        let value = match value {
            Value::Boolean(false) => return String::new(),
            Value::Boolean(true) => return escape(attribute, NAME[x][y]),
            Value::Text(value) => value,
        };
        let name = escape(attribute, NAME[x][y]);
        if self.options.collapse_empty_attributes && value.is_empty() {
            return name;
        }
        let unquoted = self
            .options
            .prefer_unquoted
            .then(|| escape(&value, UNQUOTED[x][y]));
        let result = match unquoted {
            Some(unquoted) if unquoted == value => unquoted,
            _ => {
                let mut quote = self.quote;
                if self.options.quote_smart
                    && value.matches(quote).count()
                        > value.matches(self.alternative).count()
                {
                    quote = self.alternative;
                }
                let subset = if quote == '\'' {
                    SINGLE[x][y]
                } else {
                    DOUBLE[x][y]
                };
                format!("{quote}{}{quote}", escape(&value, subset))
            }
        };
        if result.is_empty() {
            name
        } else {
            format!("{name}={result}")
        }
    }
}

enum Value {
    Boolean(bool),
    Text(String),
}

fn push_children<'n, 'a>(stack: &mut Vec<Step<'n, 'a>>, parent: Siblings<'n, 'a>) {
    for (index, child) in parent.children.iter().enumerate().rev() {
        stack.push(Step::Open {
            node: child,
            index,
            parent: Some(parent),
        });
    }
}

// comma-separated-tokens' `stringify`: a trailing empty item keeps its comma
fn join_commas(items: &[String], pad_left: bool) -> String {
    let separator = if pad_left { ", " } else { "," };
    let mut joined = items.join(separator);
    if items.last().is_some_and(String::is_empty) {
        joined.push_str(separator);
    }
    joined.trim_matches(is_js_space).to_string()
}

fn is_js_space(character: char) -> bool {
    character.is_whitespace() || character == '\u{FEFF}'
}

// stringify-entities with a subset: each listed character as a hexadecimal reference
fn escape(value: &str, subset: &str) -> String {
    let mut output = String::with_capacity(value.len());
    push_escaped(&mut output, value, subset);
    output
}

fn push_escaped(output: &mut String, value: &str, subset: &str) {
    for character in value.chars() {
        if subset.contains(character) {
            output.push_str(&format!("&#x{:X};", u32::from(character)));
        } else {
            output.push(character);
        }
    }
}

/// A number as JavaScript's `String(number)` writes it.
pub(crate) fn write_js_number(number: f64) -> String {
    if number.is_infinite() {
        return if number > 0.0 {
            "Infinity"
        } else {
            "-Infinity"
        }
        .to_string();
    }
    if number == 0.0 {
        return "0".to_string();
    }
    let magnitude = number.abs();
    if (1e-6..1e21).contains(&magnitude) {
        return format!("{number}");
    }
    // JavaScript writes the shortest digits with an exponent: `1e+21`, `1.5e-7`
    let formatted = format!("{number:e}");
    match formatted.split_once('e') {
        Some((mantissa, exponent)) if !exponent.starts_with('-') => {
            format!("{mantissa}e+{exponent}")
        }
        _ => formatted,
    }
}
