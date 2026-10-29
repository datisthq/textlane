use crate::helpers::json::format_number;
use crate::models::estree::{Id, Value};
use crate::models::name::{Key, Kind};
use crate::utils::printer::state::Printer;

impl Printer<'_> {
    /// estree-util-to-js's JSX handlers; other kinds print nothing.
    pub(crate) fn jsx(&mut self, id: Id) {
        match self.kind(id) {
            Kind::JSX_ATTRIBUTE => {
                self.field(id, Key::NAME);
                if let Some(value) = self.store.child(id, Key::VALUE) {
                    self.write("=");
                    if self.kind(value) == Kind::LITERAL {
                        let text = match self.store.get(value, Key::VALUE) {
                            Some(Value::Str(text)) => text.to_string(),
                            Some(Value::Number(number)) => format_number(*number),
                            Some(Value::Bool(flag)) => flag.to_string(),
                            _ => "null".to_string(),
                        };
                        let quoted =
                            format!("\"{}\"", encode_jsx(&text).replace('"', "&quot;"));
                        self.write(&quoted);
                    } else {
                        self.node(value);
                    }
                }
            }
            Kind::JSX_CLOSING_ELEMENT => {
                self.write("</");
                self.field(id, Key::NAME);
                self.write(">");
            }
            Kind::JSX_CLOSING_FRAGMENT => self.write("</>"),
            Kind::JSX_ELEMENT => {
                self.field(id, Key::OPENING_ELEMENT);
                for child in self.store.children(id, Key::CHILDREN) {
                    self.node(child);
                }
                self.field(id, Key::CLOSING_ELEMENT);
            }
            Kind::JSX_EXPRESSION_CONTAINER => {
                self.write("{");
                self.field(id, Key::EXPRESSION);
                self.write("}");
            }
            Kind::JSX_FRAGMENT => {
                self.field(id, Key::OPENING_FRAGMENT);
                for child in self.store.children(id, Key::CHILDREN) {
                    self.node(child);
                }
                self.field(id, Key::CLOSING_FRAGMENT);
            }
            Kind::JSX_IDENTIFIER => {
                let name = self
                    .store
                    .str(id, Key::NAME)
                    .unwrap_or("undefined")
                    .to_string();
                self.write(&name);
            }
            Kind::JSX_MEMBER_EXPRESSION => {
                self.field(id, Key::OBJECT);
                self.write(".");
                self.field(id, Key::PROPERTY);
            }
            Kind::JSX_NAMESPACED_NAME => {
                self.field(id, Key::NAMESPACE);
                self.write(":");
                self.field(id, Key::NAME);
            }
            Kind::JSX_OPENING_ELEMENT => {
                self.write("<");
                self.field(id, Key::NAME);
                for attribute in self.store.children(id, Key::ATTRIBUTES) {
                    self.write(" ");
                    self.node(attribute);
                }
                self.write(if self.store.flag(id, Key::SELF_CLOSING) {
                    " />"
                } else {
                    ">"
                });
            }
            Kind::JSX_OPENING_FRAGMENT => self.write("<>"),
            Kind::JSX_SPREAD_ATTRIBUTE => {
                self.write("{...");
                self.field(id, Key::ARGUMENT);
                self.write("}");
            }
            Kind::JSX_TEXT => {
                let value = self.store.str(id, Key::VALUE).unwrap_or_default();
                let mut text = String::with_capacity(value.len());
                for character in encode_jsx(value).chars() {
                    match character {
                        '<' => text.push_str("&lt;"),
                        '>' => text.push_str("&gt;"),
                        '{' => text.push_str("&#123;"),
                        '}' => text.push_str("&#125;"),
                        _ => text.push(character),
                    }
                }
                self.write(&text);
            }
            _ => {}
        }
    }
}

// `&` before `#` or a letter, which would read as a character reference
fn encode_jsx(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        result.push(character);
        if character == '&'
            && characters
                .peek()
                .is_some_and(|next| *next == '#' || next.is_ascii_alphabetic())
        {
            result.push_str("amp;");
        }
    }
    result
}
