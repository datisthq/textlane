use std::sync::Arc;

use textlane_core::{Error, ScriptParser};

use crate::helpers::character::find_character_after;
use crate::helpers::script::{
    format_character, is_empty_script, is_id_cont, is_id_start, is_js_whitespace,
};

const JSX: &str = "micromark-extension-mdx-jsx";
const EXPRESSION: &str = "micromark-extension-mdx-expression";
const ESM: &str = "micromark-extension-mdxjs-esm";

/// Text a scanner reads, which may grow by a line when it asks past its end.
pub(crate) trait MdxText {
    fn bytes(&self) -> &[u8];
    /// Add the next line; `false` at the end of the document, or of the paragraph.
    fn extend(&mut self) -> bool;
    /// Whether the line starting at `at` continued lazily.
    fn lazy(&self, at: usize) -> bool;
}

/// A whole text, as a paragraph's: it never grows, and its lines are never lazy errors.
pub(crate) struct Fixed<'t>(pub(crate) &'t [u8]);

impl MdxText for Fixed<'_> {
    fn bytes(&self) -> &[u8] {
        self.0
    }
    fn extend(&mut self) -> bool {
        false
    }
    fn lazy(&self, _: usize) -> bool {
        false
    }
}

fn byte(text: &mut dyn MdxText, at: usize) -> Option<u8> {
    while at >= text.bytes().len() {
        if !text.extend() {
            return None;
        }
    }
    Some(text.bytes()[at])
}

fn character(text: &mut dyn MdxText, at: usize) -> Option<char> {
    match byte(text, at)? {
        byte if byte.is_ascii() => Some(char::from(byte)),
        _ => {
            // A multibyte character may straddle what is read so far
            let _ = byte(text, at + 3);
            find_character_after(text.bytes(), at)
        }
    }
}

fn is_eol(byte: Option<u8>) -> bool {
    matches!(byte, Some(b'\n' | b'\r'))
}

/// A JSX attribute as scanned: its byte span and what it holds.
#[derive(Debug, Clone)]
pub(crate) enum ScannedAttribute {
    /// `name` or `name=value`: the name's span, its local part's, and the value.
    Plain {
        span: (usize, usize),
        name: (usize, usize),
        local: Option<(usize, usize)>,
        value: Option<ScannedValue>,
    },
    /// `{...spread}`: the span and the expression's value.
    Expression { span: (usize, usize), value: String },
}

#[derive(Debug, Clone)]
pub(crate) enum ScannedValue {
    Literal(String),
    Expression(String),
}

/// A JSX tag as scanned.
#[derive(Debug, Clone)]
pub(crate) struct ScannedTag {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) close: bool,
    pub(crate) self_closing: bool,
    /// The name, members and local part joined as mdast-util-mdx-jsx joins them.
    pub(crate) name: Option<String>,
    pub(crate) attributes: Vec<ScannedAttribute>,
    /// Where a closing slash, a self-closing slash in a closing tag, or an attribute in
    /// one sits: the compile errors upstream raises at those tokens, in order.
    pub(crate) closing_slash: Option<usize>,
    pub(crate) closing_self_slash: Option<usize>,
    pub(crate) closing_attribute: Option<usize>,
}

/// What a JSX or expression scan needs to know of its place.
pub(crate) struct MdxScan<'p> {
    pub(crate) parser: Option<&'p Arc<dyn ScriptParser>>,
    /// Whether lazy lines may continue the construct: in text, not in flow.
    pub(crate) allow_lazy: bool,
}

fn crash_at(text: &mut dyn MdxText, at: usize, place: &str, expect: &str) -> Error {
    let character = character(text, at);
    let rule = if character.is_none() {
        "unexpected-eof"
    } else {
        "unexpected-character"
    };
    Error::new(
        format!("{JSX}:{rule}"),
        format!(
            "Unexpected {} {place}, expected {expect}",
            format_character(character)
        ),
    )
}

impl MdxScan<'_> {
    // ECMAScript whitespace and line endings; a lazy line in flow is an error
    fn es_whitespace(
        &self,
        text: &mut dyn MdxText,
        mut at: usize,
    ) -> Result<usize, Error> {
        loop {
            let current = byte(text, at);
            if is_eol(current) {
                at += if current == Some(b'\r') && byte(text, at + 1) == Some(b'\n') {
                    2
                } else {
                    1
                };
                // Asking for the next line's first byte reads it in
                let _ = byte(text, at);
                if !self.allow_lazy && text.lazy(at) {
                    return Err(Error::new(
                        format!("{JSX}:unexpected-lazy"),
                        "Unexpected lazy line in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc",
                    ));
                }
                continue;
            }
            match character(text, at) {
                Some(found) if is_js_whitespace(found) => at += found.len_utf8(),
                _ => return Ok(at),
            }
        }
    }

    fn starts_id(text: &mut dyn MdxText, at: usize) -> bool {
        character(text, at).is_some_and(is_id_start)
    }

    fn continues_id(text: &mut dyn MdxText, at: usize) -> Option<usize> {
        character(text, at)
            .filter(|found| is_id_cont(*found))
            .map(char::len_utf8)
    }

    fn is_whitespace(text: &mut dyn MdxText, at: usize) -> bool {
        is_eol(byte(text, at)) || character(text, at).is_some_and(is_js_whitespace)
    }

    /// A JSX tag at `at`, `<`: `Ok(None)` when it is no tag, an error when it is a broken one.
    pub(crate) fn scan_tag(
        &self,
        text: &mut dyn MdxText,
        at: usize,
    ) -> Result<Option<ScannedTag>, Error> {
        let mut tag = ScannedTag {
            start: at,
            end: at,
            close: false,
            self_closing: false,
            name: None,
            attributes: vec![],
            closing_slash: None,
            closing_self_slash: None,
            closing_attribute: None,
        };
        let mut index = at + 1;
        let after = byte(text, index);
        if matches!(after, Some(b'\t' | b' ')) || is_eol(after) {
            return Ok(None);
        }
        index = self.es_whitespace(text, index)?;
        // Before the name
        match byte(text, index) {
            Some(b'/') => {
                tag.close = true;
                tag.closing_slash = Some(index);
                index = self.es_whitespace(text, index + 1)?;
                match byte(text, index) {
                    Some(b'>') => return Ok(Some(finish(tag, index))),
                    _ if Self::starts_id(text, index) => {}
                    current => {
                        let note = if matches!(current, Some(b'*' | b'/')) {
                            " (note: JS comments in JSX tags are not supported in MDX)"
                        } else {
                            ""
                        };
                        return Err(crash_at(
                            text,
                            index,
                            "before name",
                            &format!(
                                "a character that can start a name, such as a letter, `$`, or `_`{note}"
                            ),
                        ));
                    }
                }
            }
            Some(b'>') => return Ok(Some(finish(tag, index))),
            _ if Self::starts_id(text, index) => {}
            current => {
                let note = if current == Some(b'!') {
                    " (note: to create a comment in MDX, use `{/* text */}`)"
                } else {
                    ""
                };
                return Err(crash_at(
                    text,
                    index,
                    "before name",
                    &format!(
                        "a character that can start a name, such as a letter, `$`, or `_`{note}"
                    ),
                ));
            }
        }
        // The primary name
        let name_start = index;
        index += character(text, index).map_or(1, char::len_utf8);
        loop {
            match byte(text, index) {
                Some(b'.' | b'/' | b':' | b'>' | b'{') => break,
                _ if Self::is_whitespace(text, index) => break,
                _ => match Self::continues_id(text, index) {
                    Some(size) => index += size,
                    None => {
                        let note = if byte(text, index) == Some(b'@') {
                            " (note: to create a link in MDX, use `[text](url)`)"
                        } else {
                            ""
                        };
                        return Err(crash_at(
                            text,
                            index,
                            "in name",
                            &format!(
                                "a name character such as letters, digits, `$`, or `_`; whitespace before attributes; or the end of the tag{note}"
                            ),
                        ));
                    }
                },
            }
        }
        let mut name =
            String::from_utf8_lossy(&text.bytes()[name_start..index]).into_owned();
        index = self.es_whitespace(text, index)?;
        match byte(text, index) {
            Some(b'.') => {
                // Members
                loop {
                    index = self.es_whitespace(text, index + 1)?;
                    if !Self::starts_id(text, index) {
                        return Err(crash_at(
                            text,
                            index,
                            "before member name",
                            "a character that can start an attribute name, such as a letter, `$`, or `_`; whitespace before attributes; or the end of the tag",
                        ));
                    }
                    let member_start = index;
                    index += character(text, index).map_or(1, char::len_utf8);
                    loop {
                        match byte(text, index) {
                            Some(b'.' | b'/' | b'>' | b'{') => break,
                            _ if Self::is_whitespace(text, index) => break,
                            _ => match Self::continues_id(text, index) {
                                Some(size) => index += size,
                                None => {
                                    let note = if byte(text, index) == Some(b'@') {
                                        " (note: to create a link in MDX, use `[text](url)`)"
                                    } else {
                                        ""
                                    };
                                    return Err(crash_at(
                                        text,
                                        index,
                                        "in member name",
                                        &format!(
                                            "a name character such as letters, digits, `$`, or `_`; whitespace before attributes; or the end of the tag{note}"
                                        ),
                                    ));
                                }
                            },
                        }
                    }
                    name.push('.');
                    name.push_str(&String::from_utf8_lossy(
                        &text.bytes()[member_start..index],
                    ));
                    index = self.es_whitespace(text, index)?;
                    match byte(text, index) {
                        Some(b'.') => continue,
                        Some(b'/' | b'>' | b'{') => break,
                        _ if Self::starts_id(text, index) => break,
                        _ => {
                            return Err(crash_at(
                                text,
                                index,
                                "after member name",
                                "a character that can start an attribute name, such as a letter, `$`, or `_`; whitespace before attributes; or the end of the tag",
                            ));
                        }
                    }
                }
            }
            Some(b':') => {
                index = self.es_whitespace(text, index + 1)?;
                if !Self::starts_id(text, index) {
                    let note = if matches!(byte(text, index), Some(b'+' | b'/'..=b'9')) {
                        " (note: to create a link in MDX, use `[text](url)`)"
                    } else {
                        ""
                    };
                    return Err(crash_at(
                        text,
                        index,
                        "before local name",
                        &format!(
                            "a character that can start a name, such as a letter, `$`, or `_`{note}"
                        ),
                    ));
                }
                let local_start = index;
                index += character(text, index).map_or(1, char::len_utf8);
                loop {
                    match byte(text, index) {
                        Some(b'/' | b'>' | b'{') => break,
                        _ if Self::is_whitespace(text, index) => break,
                        _ => match Self::continues_id(text, index) {
                            Some(size) => index += size,
                            None => {
                                return Err(crash_at(
                                    text,
                                    index,
                                    "in local name",
                                    "a name character such as letters, digits, `$`, or `_`; whitespace before attributes; or the end of the tag",
                                ));
                            }
                        },
                    }
                }
                name.push(':');
                name.push_str(&String::from_utf8_lossy(
                    &text.bytes()[local_start..index],
                ));
                index = self.es_whitespace(text, index)?;
                if !(matches!(byte(text, index), Some(b'/' | b'>' | b'{'))
                    || Self::starts_id(text, index))
                {
                    return Err(crash_at(
                        text,
                        index,
                        "after local name",
                        "a character that can start an attribute name, such as a letter, `$`, or `_`; whitespace before attributes; or the end of the tag",
                    ));
                }
            }
            Some(b'/' | b'>' | b'{') => {}
            _ if Self::starts_id(text, index) => {}
            _ => {
                return Err(crash_at(
                    text,
                    index,
                    "after name",
                    "a character that can start an attribute name, such as a letter, `$`, or `_`; whitespace before attributes; or the end of the tag",
                ));
            }
        }
        tag.name = Some(name);
        // Attributes
        loop {
            match byte(text, index) {
                Some(b'/') => {
                    if tag.close && tag.closing_self_slash.is_none() {
                        tag.closing_self_slash = Some(index);
                    }
                    index = self.es_whitespace(text, index + 1)?;
                    if byte(text, index) != Some(b'>') {
                        let note = if matches!(byte(text, index), Some(b'*' | b'/')) {
                            " (note: JS comments in JSX tags are not supported in MDX)"
                        } else {
                            ""
                        };
                        return Err(crash_at(
                            text,
                            index,
                            "after self-closing slash",
                            &format!("`>` to end the tag{note}"),
                        ));
                    }
                    tag.self_closing = true;
                    return Ok(Some(finish(tag, index)));
                }
                Some(b'>') => return Ok(Some(finish(tag, index))),
                Some(b'{') => {
                    if tag.close && tag.closing_attribute.is_none() {
                        tag.closing_attribute = Some(index);
                    }
                    let (end, value) = self.scan_expression(text, index, true, false)?;
                    tag.attributes.push(ScannedAttribute::Expression {
                        span: (index, end),
                        value,
                    });
                    index = self.es_whitespace(text, end)?;
                }
                _ if Self::starts_id(text, index) => {
                    if tag.close && tag.closing_attribute.is_none() {
                        tag.closing_attribute = Some(index);
                    }
                    index = self.scan_attribute(text, index, &mut tag)?;
                }
                _ => {
                    return Err(crash_at(
                        text,
                        index,
                        "before attribute name",
                        "a character that can start an attribute name, such as a letter, `$`, or `_`; whitespace before attributes; or the end of the tag",
                    ));
                }
            }
        }
    }

    fn scan_attribute(
        &self,
        text: &mut dyn MdxText,
        at: usize,
        tag: &mut ScannedTag,
    ) -> Result<usize, Error> {
        let mut index = at + character(text, at).map_or(1, char::len_utf8);
        loop {
            match byte(text, index) {
                Some(b'/' | b':' | b'=' | b'>' | b'{') => break,
                _ if Self::is_whitespace(text, index) => break,
                _ => match Self::continues_id(text, index) {
                    Some(size) => index += size,
                    None => {
                        return Err(crash_at(
                            text,
                            index,
                            "in attribute name",
                            "an attribute name character such as letters, digits, `$`, or `_`; `=` to initialize a value; whitespace before attributes; or the end of the tag",
                        ));
                    }
                },
            }
        }
        let name = (at, index);
        let mut end = index;
        let mut local = None;
        index = self.es_whitespace(text, index)?;
        if byte(text, index) == Some(b':') {
            index = self.es_whitespace(text, index + 1)?;
            if !Self::starts_id(text, index) {
                return Err(crash_at(
                    text,
                    index,
                    "before local attribute name",
                    "a character that can start an attribute name, such as a letter, `$`, or `_`; `=` to initialize a value; or the end of the tag",
                ));
            }
            let local_start = index;
            index += character(text, index).map_or(1, char::len_utf8);
            loop {
                match byte(text, index) {
                    Some(b'/' | b'=' | b'>' | b'{') => break,
                    _ if Self::is_whitespace(text, index) => break,
                    _ => match Self::continues_id(text, index) {
                        Some(size) => index += size,
                        None => {
                            return Err(crash_at(
                                text,
                                index,
                                "in local attribute name",
                                "an attribute name character such as letters, digits, `$`, or `_`; `=` to initialize a value; whitespace before attributes; or the end of the tag",
                            ));
                        }
                    },
                }
            }
            local = Some((local_start, index));
            end = index;
            index = self.es_whitespace(text, index)?;
            match byte(text, index) {
                Some(b'=' | b'/' | b'>' | b'{') => {}
                _ if Self::starts_id(text, index) => {}
                _ => {
                    return Err(crash_at(
                        text,
                        index,
                        "after local attribute name",
                        "a character that can start an attribute name, such as a letter, `$`, or `_`; `=` to initialize a value; or the end of the tag",
                    ));
                }
            }
        } else {
            match byte(text, index) {
                Some(b'=' | b'/' | b'>' | b'{') => {}
                _ if Self::starts_id(text, index) => {}
                _ => {
                    return Err(crash_at(
                        text,
                        index,
                        "after attribute name",
                        "a character that can start an attribute name, such as a letter, `$`, or `_`; `=` to initialize a value; or the end of the tag",
                    ));
                }
            }
        }
        if byte(text, index) != Some(b'=') {
            tag.attributes.push(ScannedAttribute::Plain {
                span: (at, end),
                name,
                local,
                value: None,
            });
            return Ok(index);
        }
        index = self.es_whitespace(text, index + 1)?;
        match byte(text, index) {
            Some(quote @ (b'"' | b'\'')) => {
                let mut value = String::new();
                let mut cursor = index + 1;
                loop {
                    match byte(text, cursor) {
                        None => {
                            return Err(crash_at(
                                text,
                                cursor,
                                "in attribute value",
                                &format!(
                                    "a corresponding closing quote `{}`",
                                    char::from(quote)
                                ),
                            ));
                        }
                        Some(found) if found == quote => break,
                        found if is_eol(found) => {
                            // The line ending is the value's, the indent after it is not
                            let size = if found == Some(b'\r')
                                && byte(text, cursor + 1) == Some(b'\n')
                            {
                                2
                            } else {
                                1
                            };
                            value.push_str(&String::from_utf8_lossy(
                                &text.bytes()[cursor..cursor + size],
                            ));
                            cursor = self.es_whitespace(text, cursor)?;
                        }
                        Some(_) => {
                            let start = cursor;
                            while let Some(found) = byte(text, cursor) {
                                if found == quote || is_eol(Some(found)) {
                                    break;
                                }
                                cursor += 1;
                            }
                            value.push_str(&String::from_utf8_lossy(
                                &text.bytes()[start..cursor],
                            ));
                        }
                    }
                }
                let end = cursor + 1;
                tag.attributes.push(ScannedAttribute::Plain {
                    span: (at, end),
                    name,
                    local,
                    value: Some(ScannedValue::Literal(value)),
                });
                self.es_whitespace(text, end)
            }
            Some(b'{') => {
                let (end, value) = self.scan_expression(text, index, false, false)?;
                tag.attributes.push(ScannedAttribute::Plain {
                    span: (at, end),
                    name,
                    local,
                    value: Some(ScannedValue::Expression(value)),
                });
                self.es_whitespace(text, end)
            }
            current => {
                let note = if current == Some(b'<') {
                    " (note: to use an element or fragment as a prop value in MDX, use `{<element />}`)"
                } else {
                    ""
                };
                Err(crash_at(
                    text,
                    index,
                    "before attribute value",
                    &format!(
                        "a character that can start an attribute value, such as `\"`, `'`, or `{{`{note}"
                    ),
                ))
            }
        }
    }

    /// An expression at `at`, `{`: its end and value, the lines' container prefixes and up
    /// to two columns of indent left out; `spread` when it must be one object spread.
    pub(crate) fn scan_expression(
        &self,
        text: &mut dyn MdxText,
        at: usize,
        spread: bool,
        allow_empty: bool,
    ) -> Result<(usize, String), Error> {
        let start_line_end = at;
        let _ = start_line_end;
        let mut value = String::new();
        let mut index = at + 1;
        let mut depth = 0usize;
        let mut crash: Option<Error> = None;
        let mut hard_crash = false;
        let mut parsed = 0usize;
        let allow_empty = allow_empty && !spread;
        loop {
            let Some(current) = byte(text, index) else {
                return Err(crash.unwrap_or_else(|| Error::new(
                    format!("{EXPRESSION}:unexpected-eof"),
                    "Unexpected end of file in expression, expected a corresponding closing brace for `{`",
                )));
            };
            if is_eol(Some(current)) {
                let size = if current == b'\r' && byte(text, index + 1) == Some(b'\n') {
                    2
                } else {
                    1
                };
                value.push_str(if size == 2 {
                    "\r\n"
                } else if current == b'\r' {
                    "\r"
                } else {
                    "\n"
                });
                index += size;
                let _ = byte(text, index);
                if !self.allow_lazy && text.lazy(index) {
                    return Err(Error::new(
                        format!("{EXPRESSION}:unexpected-lazy"),
                        "Unexpected lazy line in expression in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc",
                    ));
                }
                let mut prefix = 0;
                while prefix < 2 && matches!(byte(text, index), Some(b'\t' | b' ')) {
                    index += 1;
                    prefix += 1;
                }
                continue;
            }
            if current == b'}' && depth == 0 {
                match self.try_end(
                    &value,
                    spread,
                    allow_empty,
                    &mut crash,
                    &mut hard_crash,
                    &mut parsed,
                ) {
                    Ok(true) => return Ok((index + 1, value)),
                    Ok(false) => {
                        value.push('}');
                        index += 1;
                        continue;
                    }
                    Err(error) => return Err(error),
                }
            }
            if self.parser.is_none() {
                if current == b'{' {
                    depth += 1;
                } else if current == b'}' {
                    depth -= 1;
                }
            }
            let start = index;
            index += 1;
            while let Some(next) = byte(text, index) {
                if is_eol(Some(next)) || next == b'}' || next == b'{' {
                    break;
                }
                index += 1;
            }
            value.push_str(
                &String::from_utf8_lossy(&text.bytes()[start..index])
                    .replace('\0', "\u{FFFD}"),
            );
        }
    }

    fn try_end(
        &self,
        source: &str,
        spread: bool,
        allow_empty: bool,
        crash: &mut Option<Error>,
        hard_crash: &mut bool,
        parsed: &mut usize,
    ) -> Result<bool, Error> {
        let Some(parser) = self.parser else {
            return Ok(true);
        };
        // A value that failed before its end has content no later text turns into a comment
        if *hard_crash {
            return Ok(false);
        }
        if is_empty_script(source) {
            if !allow_empty {
                return Err(Error::new(
                    format!("{EXPRESSION}:unexpected-empty-expression"),
                    "Unexpected empty expression",
                ));
            }
            return Ok(true);
        }
        let (prefix, suffix) = if spread { ("({", "})") } else { ("", "") };
        let value = format!("{prefix}{source}{suffix}");
        let source_end = prefix.len() + source.len();
        *parsed += value.len();
        if *parsed > 16 * value.len() + (1 << 20) {
            *hard_crash = true;
            crash.get_or_insert_with(acorn_error);
            return Ok(false);
        }
        match parser.parse_expression(&value) {
            Ok(expression)
                if is_empty_script(
                    &value[expression.end.min(source_end)..source_end],
                ) =>
            {
                if spread {
                    check_spread(expression.object)?;
                }
                Ok(true)
            }
            Ok(_) => {
                *crash = Some(acorn_error());
                Ok(false)
            }
            Err(error) => {
                *hard_crash = error.end < source_end;
                *crash = Some(acorn_error());
                Ok(false)
            }
        }
    }
}

fn finish(mut tag: ScannedTag, at: usize) -> ScannedTag {
    tag.end = at + 1;
    tag
}

fn acorn_error() -> Error {
    Error::new(
        format!("{EXPRESSION}:acorn"),
        "Could not parse expression with acorn",
    )
}

fn check_spread(object: Option<Vec<(String, usize)>>) -> Result<(), Error> {
    let Some(properties) = object else {
        return Err(Error::new(
            format!("{EXPRESSION}:non-spread"),
            "Unexpected `ExpressionStatement` in code: expected an object spread (`{...spread}`)",
        ));
    };
    if properties.len() > 1 {
        return Err(Error::new(
            format!("{EXPRESSION}:spread-extra"),
            "Unexpected extra content in spread: only a single spread is supported",
        ));
    }
    match properties.first() {
        Some((kind, _)) if kind != "SpreadElement" => Err(Error::new(
            format!("{EXPRESSION}:non-spread"),
            format!("Unexpected `{kind}` in code: only spread elements are supported"),
        )),
        _ => Ok(()),
    }
}

const ALLOWED: [&str; 4] = [
    "ExportAllDeclaration",
    "ExportDefaultDeclaration",
    "ExportNamedDeclaration",
    "ImportDeclaration",
];

/// Whether ESM ends at a blank line or the end with `source`: its JavaScript parses, or
/// fails at its very end where more lines may finish it; adds the imports it declares.
pub(crate) fn check_esm(
    parser: Option<&Arc<dyn ScriptParser>>,
    source: &str,
    at_eof: bool,
    imports: &mut Vec<String>,
) -> Result<bool, Error> {
    let Some(parser) = parser else {
        return Ok(true);
    };
    let prefix = if imports.is_empty() {
        String::new()
    } else {
        format!("var {}\n", imports.join(","))
    };
    let value = format!("{prefix}{source}");
    let statements = match parser.parse_module(&value) {
        Ok(statements) => statements,
        Err(error) => {
            if !at_eof && error.end >= value.len() {
                return Ok(false);
            }
            return Err(Error::new(
                format!("{ESM}:acorn"),
                "Could not parse import/exports with acorn",
            ));
        }
    };
    let skip = usize::from(!prefix.is_empty());
    let mut found = vec![];
    for statement in statements.iter().skip(skip) {
        if !ALLOWED.contains(&statement.kind.as_str()) {
            return Err(Error::new(
                format!("{ESM}:non-esm"),
                format!(
                    "Unexpected `{}` in code: only import/exports are supported",
                    statement.kind
                ),
            ));
        }
        found.extend(statement.imports.iter().cloned());
    }
    imports.extend(found);
    Ok(true)
}

/// What MDX's text and flow share while the tree is built: the parser, the source for
/// places in messages, and the first errors of each kind.
pub(crate) struct MdxBuild<'d> {
    pub(crate) parser: Option<&'d Arc<dyn ScriptParser>>,
    pub(crate) source: &'d str,
    pub(crate) errors: &'d std::cell::RefCell<MdxErrors>,
    /// The innermost flow element open around a text, with its tag's span, and how many are.
    pub(crate) flow_open: Option<(Option<String>, usize, usize)>,
    pub(crate) flow_count: usize,
}

/// A syntax error, which upstream throws while tokenizing, beats any found compiling.
#[derive(Default)]
pub(crate) struct MdxErrors {
    pub(crate) crash: Option<Error>,
    pub(crate) compile: Option<Error>,
}

impl MdxErrors {
    pub(crate) fn crash(&mut self, error: Error) {
        self.crash.get_or_insert(error);
    }
    pub(crate) fn compile(&mut self, error: Error) {
        self.compile.get_or_insert(error);
    }
    pub(crate) fn first(&self) -> Option<Error> {
        self.crash.clone().or_else(|| self.compile.clone())
    }
}

const MDAST_JSX: &str = "mdast-util-mdx-jsx";

/// `line:column` of a byte offset, as unist-util-stringify-position writes it.
pub(crate) fn stringify_point(source: &str, point: textlane_core::Point) -> String {
    let point = textlane_core::Lines::new(source).to_unist_point(point);
    format!("{}:{}", point.line, point.column)
}

/// A tag as mdast-util-mdx-jsx names it in errors: `<a>` or `</a>`.
pub(crate) fn abbreviate(name: Option<&str>, close: bool) -> String {
    format!("<{}{}>", if close { "/" } else { "" }, name.unwrap_or(""))
}

/// The compile errors upstream raises at a tag's own tokens: a closing slash with nothing
/// open, then a self-closing slash or an attribute in a closing tag.
pub(crate) fn check_tag(tag: &ScannedTag, any_open: bool) -> Option<Error> {
    if tag.closing_slash.is_some() && !any_open {
        return Some(Error::new(
            format!("{MDAST_JSX}:unexpected-closing-slash"),
            "Unexpected closing slash `/` in tag, expected an open tag first",
        ));
    }
    let attribute = tag.closing_attribute.map(|at| (at, false));
    let slash = tag.closing_self_slash.map(|at| (at, true));
    let first = match (attribute, slash) {
        (Some(a), Some(b)) => Some(if a.0 < b.0 { a } else { b }),
        (a, b) => a.or(b),
    };
    first.map(|(_, slash)| {
        if slash {
            Error::new(
                format!("{MDAST_JSX}:unexpected-self-closing-slash"),
                "Unexpected self-closing slash `/` in closing tag, expected the end of the tag",
            )
        } else {
            Error::new(
                format!("{MDAST_JSX}:unexpected-attribute"),
                "Unexpected attribute in closing tag, expected the end of the tag",
            )
        }
    })
}

/// A closing tag for another element than the innermost open one.
pub(crate) fn mismatch_error(
    tag: &ScannedTag,
    open_name: Option<&str>,
    open_place: &str,
) -> Error {
    Error::new(
        format!("{MDAST_JSX}:end-tag-mismatch"),
        format!(
            "Unexpected closing tag `{}`, expected corresponding closing tag for `{}` ({open_place})",
            abbreviate(tag.name.as_deref(), true),
            abbreviate(open_name, false),
        ),
    )
}

/// A closing tag while something else than its element is open.
pub(crate) fn tail_error(tag: &ScannedTag, kind: &str, exit: &str, enter: &str) -> Error {
    Error::new(
        format!("{MDAST_JSX}:end-tag-mismatch"),
        format!(
            "Expected the closing tag `{}` either after the end of `{kind}` ({exit}) or another opening tag after the start of `{kind}` ({enter})",
            abbreviate(tag.name.as_deref(), tag.close),
        ),
    )
}

/// An element still open where `kind` ends, or at the end of the document.
pub(crate) fn unclosed_error(
    name: Option<&str>,
    place: &str,
    kind: Option<&str>,
) -> Error {
    let before = kind
        .map(|kind| format!(" before the end of `{kind}`"))
        .unwrap_or_default();
    Error::new(
        format!("{MDAST_JSX}:end-tag-mismatch"),
        format!(
            "Expected a closing tag for `{}` ({place}){before}",
            abbreviate(name, false)
        ),
    )
}

/// An empty unist `data`, as the MDX nodes carry for their estree.
pub(crate) fn empty_data(alloc: &textlane_core::Allocator) -> &textlane_core::Data<'_> {
    alloc.alloc(textlane_core::Data::Object(
        textlane_core::ArenaVec::new_in(alloc),
    ))
}

/// A scanned tag's attributes as mdast-util-mdx-jsx builds them.
pub(crate) fn build_attributes<'a>(
    alloc: &'a textlane_core::Allocator,
    tag: &ScannedTag,
    text: &'a str,
    span: impl Fn(usize, usize) -> textlane_core::Position<'a>,
) -> textlane_core::ArenaVec<'a, textlane_core::mdast::MdxJsxAttribute<'a>> {
    use textlane_core::mdast::{MdxJsxAttribute, MdxJsxAttributeValue};
    let mut attributes =
        textlane_core::ArenaVec::with_capacity_in(tag.attributes.len(), alloc);
    for attribute in &tag.attributes {
        attributes.push(match attribute {
            ScannedAttribute::Plain {
                span: (start, end),
                name,
                local,
                value,
            } => {
                let mut full = text[name.0..name.1].to_string();
                if let Some((from, to)) = local {
                    full.push(':');
                    full.push_str(&text[*from..*to]);
                }
                let value = value.as_ref().map(|value| match value {
                    ScannedValue::Literal(literal) => {
                        MdxJsxAttributeValue::Literal(alloc.alloc_str(
                            &crate::helpers::attribute::decode_terminated(literal),
                        ))
                    }
                    ScannedValue::Expression(expression) => {
                        MdxJsxAttributeValue::Expression {
                            value: alloc.alloc_str(expression),
                            data: Some(empty_data(alloc)),
                        }
                    }
                });
                MdxJsxAttribute::Attribute {
                    name: alloc.alloc_str(&full),
                    value,
                    position: Some(span(*start, *end)),
                    data: None,
                }
            }
            ScannedAttribute::Expression {
                span: (start, end),
                value,
            } => MdxJsxAttribute::Expression {
                value: alloc.alloc_str(value),
                position: Some(span(*start, *end)),
                data: Some(empty_data(alloc)),
            },
        });
    }
    attributes
}
