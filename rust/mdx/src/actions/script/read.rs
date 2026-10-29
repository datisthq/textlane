use std::borrow::Cow;

use oxc_ast::ast::{CommentKind, Program};
use textlane_core::Error;

use crate::helpers::entity::decode_entities;
use crate::models::estree::{Id, Store, Value};
use crate::models::name::{Key, Kind};

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
        pending: Vec::new(),
    }
    .read()?
    .as_node()
    .ok_or_else(|| invalid("not an object"))?;
    let mut comments = Vec::with_capacity(program.comments.len());
    for comment in &program.comments {
        let (start, end) = (comment.span.start as usize, comment.span.end as usize);
        let (kind, value) = match comment.kind {
            CommentKind::Line => (Kind::LINE, source.get(start + 2..end)),
            _ => (Kind::BLOCK, source.get(start + 2..end.saturating_sub(2))),
        };
        comments.push(store.add(
            kind,
            [
                (Key::VALUE, Value::str(value.unwrap_or_default())),
                (Key::START, Value::Number((base + start) as f64)),
                (Key::END, Value::Number((base + end) as f64)),
            ],
        ));
    }
    Ok((program_id, comments))
}

enum Frame {
    // An object's kind, where its fields start in `pending`, and the key read last
    Object(Kind, usize, Option<Key>),
    Array(Vec<Value>),
}

struct Reader<'j, 's> {
    bytes: &'j [u8],
    text: &'j str,
    index: usize,
    store: &'s mut Store,
    base: usize,
    pending: Vec<(Key, Value)>,
}

fn invalid(reason: &str) -> Error {
    Error::new(
        "mdx-failed",
        format!("Cannot read the JavaScript tree: {reason}"),
    )
}

impl<'j> Reader<'j, '_> {
    // JSON nests as deep as the script does, so values are built on an explicit stack
    fn read(&mut self) -> Result<Value, Error> {
        let mut stack: Vec<Frame> = Vec::new();
        loop {
            self.skip_whitespace();
            let mut typed = false;
            let mut value = match self.peek()? {
                b'{' => {
                    self.index += 1;
                    self.skip_whitespace();
                    if self.peek()? == b'}' {
                        self.index += 1;
                        self.finish_object(Kind::NONE, self.pending.len())
                    } else {
                        let key = self.read_key()?;
                        stack.push(Frame::Object(
                            Kind::NONE,
                            self.pending.len(),
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
                b'"' => {
                    let text = self.read_string()?;
                    match stack.last_mut() {
                        Some(Frame::Object(kind, _, Some(Key::TYPE)))
                            if *kind == Kind::NONE =>
                        {
                            *kind = self.store.intern_kind(&text);
                            typed = true;
                            Value::Null
                        }
                        _ => Value::Str(text.into()),
                    }
                }
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
                    Frame::Object(_, _, key) => {
                        let key =
                            key.take().ok_or_else(|| invalid("a value without a key"))?;
                        if !std::mem::take(&mut typed) {
                            self.pending.push((key, value));
                        }
                        if self.peek()? == b',' {
                            self.index += 1;
                            self.skip_whitespace();
                            *key_slot(&mut stack) = Some(self.read_key()?);
                            break;
                        }
                        self.expect(b'}')?;
                        let Some(Frame::Object(kind, start, _)) = stack.pop() else {
                            return Err(invalid("unbalanced object"));
                        };
                        value = self.finish_object(kind, start);
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

    fn finish_object(&mut self, kind: Kind, start: usize) -> Value {
        let fields = &mut self.pending[start..];
        for (key, value) in fields.iter_mut() {
            if (*key == Key::START || *key == Key::END)
                && let Value::Number(offset) = value
            {
                *offset += self.base as f64;
            }
        }
        match kind {
            Kind::JSX_TEXT => decode_field(fields, true),
            Kind::JSX_ATTRIBUTE => {
                let literal = fields
                    .iter()
                    .find(|(key, _)| *key == Key::VALUE)
                    .and_then(|(_, value)| value.as_node())
                    .filter(|value| self.store.is(*value, Kind::LITERAL));
                if let Some(literal) = literal
                    && let Some(Value::Str(text)) = self.store.get(literal, Key::VALUE)
                {
                    let decoded = decode_entities(text, false);
                    self.store
                        .set(literal, Key::VALUE, Value::Str(decoded.into()));
                }
            }
            _ => {}
        }
        Value::Node(self.store.add(kind, self.pending.drain(start..)))
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

    fn read_key(&mut self) -> Result<Key, Error> {
        let key = self.read_string()?;
        self.skip_whitespace();
        self.expect(b':')?;
        Ok(self.store.intern_key(&key))
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

    // Borrowed unless it has escapes: keys and types are only looked up, never kept
    fn read_string(&mut self) -> Result<Cow<'j, str>, Error> {
        self.expect(b'"')?;
        let text = self.text;
        let mut result = String::new();
        let mut start = self.index;
        loop {
            match self.peek()? {
                b'"' if result.is_empty() => {
                    self.index += 1;
                    return Ok(Cow::Borrowed(&text[start..self.index - 1]));
                }
                b'"' => {
                    result.push_str(&text[start..self.index]);
                    self.index += 1;
                    return Ok(Cow::Owned(result));
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

fn key_slot(stack: &mut [Frame]) -> &mut Option<Key> {
    match stack.last_mut() {
        Some(Frame::Object(_, _, key)) => key,
        _ => unreachable!("only called while an object is open"),
    }
}

fn decode_field(fields: &mut [(Key, Value)], normalize: bool) {
    for (key, value) in fields {
        if *key == Key::VALUE
            && let Value::Str(text) = value
        {
            *text = decode_entities(text, normalize).into();
        }
    }
}
