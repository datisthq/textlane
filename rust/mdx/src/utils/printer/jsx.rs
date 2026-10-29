use crate::helpers::json::format_number;
use crate::models::estree::{Id, Value};
use crate::utils::printer::state::Printer;

impl Printer<'_> {
    /// estree-util-to-js's JSX handlers; other kinds print nothing.
    pub(crate) fn jsx(&mut self, id: Id) {
        match self.kind(id) {
            "JSXAttribute" => {
                self.field(id, "name");
                if let Some(value) = self.store.child(id, "value") {
                    self.write("=");
                    if self.kind(value) == "Literal" {
                        let text = match self.store.get(value, "value") {
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
            "JSXClosingElement" => {
                self.write("</");
                self.field(id, "name");
                self.write(">");
            }
            "JSXClosingFragment" => self.write("</>"),
            "JSXElement" => {
                self.field(id, "openingElement");
                for child in self.store.children(id, "children") {
                    self.node(child);
                }
                self.field(id, "closingElement");
            }
            "JSXExpressionContainer" => {
                self.write("{");
                self.field(id, "expression");
                self.write("}");
            }
            "JSXFragment" => {
                self.field(id, "openingFragment");
                for child in self.store.children(id, "children") {
                    self.node(child);
                }
                self.field(id, "closingFragment");
            }
            "JSXIdentifier" => {
                let name = self
                    .store
                    .str(id, "name")
                    .unwrap_or("undefined")
                    .to_string();
                self.write(&name);
            }
            "JSXMemberExpression" => {
                self.field(id, "object");
                self.write(".");
                self.field(id, "property");
            }
            "JSXNamespacedName" => {
                self.field(id, "namespace");
                self.write(":");
                self.field(id, "name");
            }
            "JSXOpeningElement" => {
                self.write("<");
                self.field(id, "name");
                for attribute in self.store.children(id, "attributes") {
                    self.write(" ");
                    self.node(attribute);
                }
                self.write(if self.store.flag(id, "selfClosing") {
                    " />"
                } else {
                    ">"
                });
            }
            "JSXOpeningFragment" => self.write("<>"),
            "JSXSpreadAttribute" => {
                self.write("{...");
                self.field(id, "argument");
                self.write("}");
            }
            "JSXText" => {
                let value = self.store.str(id, "value").unwrap_or_default();
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
