use std::borrow::Cow;

use oxc_ast::ast::{CommentKind, Program};
use textlane_core::Error;

use crate::helpers::entity::decode_entities;
use crate::models::estree::{Id, Node, Store, Value};

/// Read a program oxc parsed into the store as acorn's ESTree, through oxc's own ESTree
/// serializer; JSX strings decoded as acorn-jsx does and every `start`/`end` moved by `base`.
/// Gives the program and its comments, in source order.
pub(crate) fn read_script(
    store: &mut Store,
    program: &Program<'_>,
    source: &str,
    base: usize,
) -> Result<(Id, Vec<Id>), Error> {
    let json = program.to_estree_json(false, false);
    let program_id = Reader {
        bytes: json.as_bytes(),
        text: &json,
        index: 0,
        store,
        base,
    }
    .read()?
    .as_node()
    .ok_or_else(|| invalid("not an object"))?;
    let mut comments = Vec::with_capacity(program.comments.len());
    for comment in &program.comments {
        let (start, end) = (comment.span.start as usize, comment.span.end as usize);
        let (kind, value) = match comment.kind {
            CommentKind::Line => ("Line", source.get(start + 2..end)),
            _ => ("Block", source.get(start + 2..end.saturating_sub(2))),
        };
        comments.push(store.add(
            kind,
            vec![
                ("value", Value::str(value.unwrap_or_default())),
                ("start", Value::Number((base + start) as f64)),
                ("end", Value::Number((base + end) as f64)),
            ],
        ));
    }
    Ok((program_id, comments))
}

enum Frame {
    Object(Node, Option<Cow<'static, str>>),
    Array(Vec<Value>),
}

struct Reader<'j, 's> {
    bytes: &'j [u8],
    text: &'j str,
    index: usize,
    store: &'s mut Store,
    base: usize,
}

fn invalid(reason: &str) -> Error {
    Error::new(
        "mdx-failed",
        format!("Cannot read the JavaScript tree: {reason}"),
    )
}

impl Reader<'_, '_> {
    // JSON nests as deep as the script does, so values are built on an explicit stack
    fn read(&mut self) -> Result<Value, Error> {
        let mut stack: Vec<Frame> = Vec::new();
        loop {
            self.skip_whitespace();
            let mut value = match self.peek()? {
                b'{' => {
                    self.index += 1;
                    self.skip_whitespace();
                    if self.peek()? == b'}' {
                        self.index += 1;
                        self.finish_object(Node {
                            kind: Cow::Borrowed(""),
                            fields: Vec::new(),
                        })
                    } else {
                        let key = self.read_key()?;
                        stack.push(Frame::Object(
                            Node {
                                kind: Cow::Borrowed(""),
                                fields: Vec::new(),
                            },
                            Some(key),
                        ));
                        continue;
                    }
                }
                b'[' => {
                    self.index += 1;
                    self.skip_whitespace();
                    if self.peek()? == b']' {
                        self.index += 1;
                        Value::List(Vec::new())
                    } else {
                        stack.push(Frame::Array(Vec::new()));
                        continue;
                    }
                }
                b'"' => Value::Str(self.read_string()?.into()),
                b't' => self.read_word("true", Value::Bool(true))?,
                b'f' => self.read_word("false", Value::Bool(false))?,
                b'n' => self.read_word("null", Value::Null)?,
                _ => self.read_number()?,
            };
            loop {
                let Some(frame) = stack.last_mut() else {
                    return Ok(value);
                };
                self.skip_whitespace();
                match frame {
                    Frame::Object(node, key) => {
                        let key =
                            key.take().ok_or_else(|| invalid("a value without a key"))?;
                        node.fields.push((key, value));
                        if self.peek()? == b',' {
                            self.index += 1;
                            self.skip_whitespace();
                            *key_slot(&mut stack) = Some(self.read_key()?);
                            break;
                        }
                        self.expect(b'}')?;
                        let Some(Frame::Object(node, _)) = stack.pop() else {
                            return Err(invalid("unbalanced object"));
                        };
                        value = self.finish_object(node);
                    }
                    Frame::Array(items) => {
                        items.push(value);
                        if self.peek()? == b',' {
                            self.index += 1;
                            break;
                        }
                        self.expect(b']')?;
                        let Some(Frame::Array(items)) = stack.pop() else {
                            return Err(invalid("unbalanced array"));
                        };
                        value = Value::List(items);
                    }
                }
            }
        }
    }

    fn finish_object(&mut self, mut node: Node) -> Value {
        if let Some(index) = node.fields.iter().position(|(key, _)| &**key == "type")
            && let Value::Str(kind) = &node.fields[index].1
        {
            node.kind = Cow::Owned(kind.to_string());
            node.fields.remove(index);
        }
        for (key, value) in &mut node.fields {
            if (&**key == "start" || &**key == "end")
                && let Value::Number(offset) = value
            {
                *offset += self.base as f64;
            }
        }
        match &*node.kind {
            "JSXText" => decode_field(&mut node, true),
            "JSXAttribute" => {
                let literal = node
                    .fields
                    .iter()
                    .find(|(key, _)| &**key == "value")
                    .and_then(|(_, value)| value.as_node())
                    .filter(|value| self.store.is(*value, "Literal"));
                if let Some(literal) = literal
                    && let Some(Value::Str(text)) = self.store.get(literal, "value")
                {
                    let decoded = decode_entities(text, false);
                    self.store.set(literal, "value", Value::Str(decoded.into()));
                }
            }
            _ => {}
        }
        Value::Node(self.store.add_node(node))
    }

    fn skip_whitespace(&mut self) {
        while self
            .bytes
            .get(self.index)
            .is_some_and(|byte| matches!(byte, b' ' | b'\n' | b'\r' | b'\t'))
        {
            self.index += 1;
        }
    }

    fn peek(&self) -> Result<u8, Error> {
        self.bytes
            .get(self.index)
            .copied()
            .ok_or_else(|| invalid("unexpected end"))
    }

    fn expect(&mut self, byte: u8) -> Result<(), Error> {
        if self.peek()? != byte {
            return Err(invalid("unexpected character"));
        }
        self.index += 1;
        Ok(())
    }

    fn read_key(&mut self) -> Result<Cow<'static, str>, Error> {
        let key = self.read_string()?;
        self.skip_whitespace();
        self.expect(b':')?;
        Ok(Cow::Owned(key))
    }

    fn read_word(&mut self, word: &str, value: Value) -> Result<Value, Error> {
        if !self.text[self.index..].starts_with(word) {
            return Err(invalid("unexpected word"));
        }
        self.index += word.len();
        Ok(value)
    }

    fn read_number(&mut self) -> Result<Value, Error> {
        let start = self.index;
        while self.bytes.get(self.index).is_some_and(|byte| {
            matches!(byte, b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
        }) {
            self.index += 1;
        }
        self.text[start..self.index]
            .parse()
            .map(Value::Number)
            .map_err(|_| invalid("not a number"))
    }

    fn read_string(&mut self) -> Result<String, Error> {
        self.expect(b'"')?;
        let mut result = String::new();
        let mut start = self.index;
        loop {
            match self.peek()? {
                b'"' => {
                    result.push_str(&self.text[start..self.index]);
                    self.index += 1;
                    return Ok(result);
                }
                b'\\' => {
                    result.push_str(&self.text[start..self.index]);
                    self.index += 1;
                    let escape = self.peek()?;
                    self.index += 1;
                    match escape {
                        b'n' => result.push('\n'),
                        b'r' => result.push('\r'),
                        b't' => result.push('\t'),
                        b'b' => result.push('\u{8}'),
                        b'f' => result.push('\u{c}'),
                        b'u' => {
                            let unit = self.read_unit()?;
                            let character = if (0xd800..0xdc00).contains(&unit)
                                && self.text[self.index..].starts_with("\\u")
                            {
                                self.index += 2;
                                let low = self.read_unit()?;
                                char::decode_utf16([unit, low])
                                    .next()
                                    .and_then(Result::ok)
                            } else {
                                char::from_u32(u32::from(unit))
                            };
                            result.push(character.unwrap_or('\u{fffd}'));
                        }
                        other => result.push(char::from(other)),
                    }
                    start = self.index;
                }
                _ => self.index += 1,
            }
        }
    }

    fn read_unit(&mut self) -> Result<u16, Error> {
        let digits = self
            .text
            .get(self.index..self.index + 4)
            .ok_or_else(|| invalid("short escape"))?;
        self.index += 4;
        u16::from_str_radix(digits, 16).map_err(|_| invalid("bad escape"))
    }
}

fn key_slot(stack: &mut [Frame]) -> &mut Option<Cow<'static, str>> {
    match stack.last_mut() {
        Some(Frame::Object(_, key)) => key,
        _ => unreachable!("only called while an object is open"),
    }
}

fn decode_field(node: &mut Node, normalize: bool) {
    for (key, value) in &mut node.fields {
        if &**key == "value"
            && let Value::Str(text) = value
        {
            *text = decode_entities(text, normalize).into();
        }
    }
}
