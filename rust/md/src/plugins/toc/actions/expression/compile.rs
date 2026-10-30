use regex::Regex;
use textlane_core::Error;

const DIGIT: &str = "[0-9]";
const WORD: &str = "[0-9A-Za-z_]";
const SPACE: &str = r"[\t\n\x{B}\x{C}\r \x{A0}\x{1680}\x{2000}-\x{200A}\x{2028}\x{2029}\x{202F}\x{205F}\x{3000}\x{FEFF}]";
const NOTHING: &str = r"[^\x{0}-\x{10FFFF}]";

/// mdast-util-toc's `toExpression`, `new RegExp('^(' + value + ')$', 'i')`, translated from
/// JavaScript's syntax to the `regex` crate's.
pub(crate) fn compile_expression(value: &str) -> Result<Regex, Error> {
    let source = format!("^({value})$");
    let translated = translate_source(&source)?;
    Regex::new(&format!("(?i){translated}")).map_err(|error| {
        Error::new(
            "invalid-options",
            format!("Invalid regular expression: /{source}/i: {error}"),
        )
    })
}

fn translate_source(source: &str) -> Result<String, Error> {
    let units = source.encode_utf16().collect::<Vec<u16>>();
    let mut reader = Reader { units, index: 0 };
    let mut output = String::new();
    while let Some(unit) = reader.next() {
        match unit {
            0x5C => translate_escape(&mut reader, &mut output, source)?,
            0x2E => output.push_str(r"[^\n\r\x{2028}\x{2029}]"),
            0x5B => translate_class(&mut reader, &mut output, source)?,
            0x28 => {
                if reader.peek(0) == Some(0x3F) {
                    let lookaround = matches!(
                        (reader.peek(1), reader.peek(2)),
                        (Some(0x3D | 0x21), _) | (Some(0x3C), Some(0x3D | 0x21))
                    );
                    if lookaround {
                        return Err(unsupported(source, "lookaround"));
                    }
                }
                output.push('(');
                if reader.peek(0) == Some(0x3F) {
                    reader.index += 1;
                    output.push('?');
                }
            }
            0x7B => {
                if let Some(length) = read_quantifier(&reader) {
                    output.push('{');
                    for _ in 0..length {
                        if let Some(unit) = reader.next() {
                            output.push(char::from(unit as u8));
                        }
                    }
                } else {
                    output.push_str(r"\{");
                }
            }
            0x7D => output.push_str(r"\}"),
            0x5D => output.push_str(r"\]"),
            0x29 | 0x7C | 0x5E | 0x24 | 0x2A | 0x2B | 0x3F => {
                output.push(char::from(unit as u8));
            }
            _ => {
                let character = reader.finish_character(unit);
                push_literal(&mut output, character);
            }
        }
    }
    Ok(output)
}

struct Reader {
    units: Vec<u16>,
    index: usize,
}

impl Reader {
    fn next(&mut self) -> Option<u16> {
        let unit = self.units.get(self.index).copied();
        self.index += 1;
        unit
    }

    fn peek(&self, ahead: usize) -> Option<u16> {
        self.units.get(self.index + ahead).copied()
    }

    // A high surrogate pairs with the low one after it; a lone one is no character
    fn finish_character(&mut self, unit: u16) -> Option<char> {
        if (0xD800..0xDC00).contains(&unit)
            && let Some(low) = self.peek(0)
            && (0xDC00..0xE000).contains(&low)
        {
            self.index += 1;
            let code =
                0x10000 + ((u32::from(unit) - 0xD800) << 10) + (u32::from(low) - 0xDC00);
            return char::from_u32(code);
        }
        char::from_u32(u32::from(unit))
    }

    fn read_hex(&mut self, count: usize) -> Option<u16> {
        let mut value = 0u16;
        for ahead in 0..count {
            let digit = char::from_u32(u32::from(self.peek(ahead)?))?.to_digit(16)?;
            value = value * 16 + digit as u16;
        }
        self.index += count;
        Some(value)
    }

    // `\uD83D\uDE00` is one character, as JavaScript reads it
    fn read_unicode_escape(&mut self) -> Option<Option<char>> {
        let unit = self.read_hex(4)?;
        if (0xD800..0xDC00).contains(&unit)
            && self.peek(0) == Some(0x5C)
            && self.peek(1) == Some(0x75)
        {
            let start = self.index;
            self.index += 2;
            match self.read_hex(4) {
                Some(low) if (0xDC00..0xE000).contains(&low) => {
                    let code = 0x10000
                        + ((u32::from(unit) - 0xD800) << 10)
                        + (u32::from(low) - 0xDC00);
                    return Some(char::from_u32(code));
                }
                _ => self.index = start,
            }
        }
        Some(char::from_u32(u32::from(unit)))
    }
}

// One atom of an escape: a set, as source, or a character (`None` for a lone surrogate)
enum Atom {
    Set(&'static str),
    Character(Option<char>),
}

fn translate_escape(
    reader: &mut Reader,
    output: &mut String,
    source: &str,
) -> Result<(), Error> {
    let Some(unit) = reader.next() else {
        return Err(Error::new(
            "invalid-options",
            format!("Invalid regular expression: /{source}/i: \\ at end of pattern"),
        ));
    };
    match unit {
        0x62 => output.push_str(r"(?-u:\b)"),
        0x42 => output.push_str(r"(?-u:\B)"),
        0x31..=0x39 => return Err(unsupported(source, "a back reference")),
        0x6B if reader.peek(0) == Some(0x3C) => {
            return Err(unsupported(source, "a back reference"));
        }
        _ => match read_escape_atom(reader, unit) {
            Atom::Set(set) => {
                output.push_str("(?-i:");
                output.push_str(set);
                output.push(')');
            }
            Atom::Character(character) => push_literal(output, character),
        },
    }
    Ok(())
}

// What an escape after `\` stands for, `unit` being the character after the backslash
fn read_escape_atom(reader: &mut Reader, unit: u16) -> Atom {
    let set = match unit {
        0x64 => Some(DIGIT),
        0x44 => Some("[^0-9]"),
        0x77 => Some(WORD),
        0x57 => Some("[^0-9A-Za-z_]"),
        0x73 => Some(SPACE),
        0x53 => Some(
            r"[^\t\n\x{B}\x{C}\r \x{A0}\x{1680}\x{2000}-\x{200A}\x{2028}\x{2029}\x{202F}\x{205F}\x{3000}\x{FEFF}]",
        ),
        _ => None,
    };
    if let Some(set) = set {
        return Atom::Set(set);
    }
    let code = match unit {
        0x74 => Some(0x09),
        0x6E => Some(0x0A),
        0x76 => Some(0x0B),
        0x66 => Some(0x0C),
        0x72 => Some(0x0D),
        0x30 => Some(read_octal(reader)),
        0x63 => match reader.peek(0) {
            Some(letter @ (0x41..=0x5A | 0x61..=0x7A)) => {
                reader.index += 1;
                Some(u32::from(letter % 32))
            }
            // Annex B: `\c` before anything else is a backslash, then `c`
            _ => {
                reader.index -= 1;
                Some(0x5C)
            }
        },
        0x78 => reader.read_hex(2).map(u32::from),
        0x75 => match reader.read_unicode_escape() {
            Some(character) => return Atom::Character(character),
            None => None,
        },
        _ => None,
    };
    match code {
        Some(code) => Atom::Character(char::from_u32(code)),
        None => Atom::Character(reader.finish_character(unit)),
    }
}

// Annex B's legacy octal escapes: `\0`, then up to two more octal digits under 0o377
fn read_octal(reader: &mut Reader) -> u32 {
    let mut value = 0;
    for _ in 0..2 {
        match reader.peek(0) {
            Some(digit @ 0x30..=0x37) if value * 8 + u32::from(digit - 0x30) <= 0o377 => {
                value = value * 8 + u32::from(digit - 0x30);
                reader.index += 1;
            }
            _ => break,
        }
    }
    value
}

// `{n}`, `{n,}` or `{n,m}` after `{`, its length up to and with `}`; else a literal brace
fn read_quantifier(reader: &Reader) -> Option<usize> {
    let mut ahead = 0;
    let mut digits = 0;
    let mut comma = false;
    loop {
        match reader.peek(ahead)? {
            0x30..=0x39 => digits += 1,
            0x2C if !comma && digits > 0 => comma = true,
            0x7D if digits > 0 => return Some(ahead + 1),
            _ => return None,
        }
        ahead += 1;
    }
}

fn translate_class(
    reader: &mut Reader,
    output: &mut String,
    source: &str,
) -> Result<(), Error> {
    let negated = reader.peek(0) == Some(0x5E);
    if negated {
        reader.index += 1;
    }
    let mut items: Vec<String> = Vec::new();
    loop {
        let Some(unit) = reader.next() else {
            return Err(Error::new(
                "invalid-options",
                format!(
                    "Invalid regular expression: /{source}/i: Unterminated character class"
                ),
            ));
        };
        if unit == 0x5D {
            break;
        }
        let atom = read_class_atom(reader, unit);
        // `a-z` is a range only between two characters; else the dash is itself
        if let Atom::Character(start) = atom
            && reader.peek(0) == Some(0x2D)
            && reader.peek(1).is_some_and(|unit| unit != 0x5D)
        {
            let saved = reader.index;
            reader.index += 1;
            let next = reader.next().unwrap_or(0x5D);
            match read_class_atom(reader, next) {
                Atom::Character(end) => {
                    match (start, end) {
                        (Some(start), Some(end)) if start > end => {
                            return Err(Error::new(
                                "invalid-options",
                                format!(
                                    "Invalid regular expression: /{source}/i: Range out of order in character class"
                                ),
                            ));
                        }
                        (Some(start), Some(end)) => items.push(format!(
                            r"\x{{{:X}}}-\x{{{:X}}}",
                            u32::from(start),
                            u32::from(end)
                        )),
                        _ => {
                            return Err(unsupported(
                                source,
                                "a range to a lone surrogate",
                            ));
                        }
                    }
                    continue;
                }
                Atom::Set(_) => reader.index = saved,
            }
        }
        match atom {
            Atom::Set(set) => items.push(set.to_string()),
            Atom::Character(Some(character)) => {
                items.push(format!(r"\x{{{:X}}}", u32::from(character)));
            }
            Atom::Character(None) => {}
        }
    }
    // `[]` matches nothing and `[^]` anything, which the `regex` crate spells otherwise
    match (negated, items.is_empty()) {
        (false, true) => output.push_str(NOTHING),
        (true, true) => output.push_str(r"[\x{0}-\x{10FFFF}]"),
        (false, false) => {
            output.push('[');
            output.extend(items);
            output.push(']');
        }
        (true, false) => {
            output.push_str("[^");
            output.extend(items);
            output.push(']');
        }
    }
    Ok(())
}

fn read_class_atom(reader: &mut Reader, unit: u16) -> Atom {
    if unit != 0x5C {
        return Atom::Character(reader.finish_character(unit));
    }
    let Some(next) = reader.next() else {
        return Atom::Character(Some('\\'));
    };
    match next {
        // In a class, `\b` is a backspace and `\B` a `B`
        0x62 => Atom::Character(Some('\u{8}')),
        0x2D => Atom::Character(Some('-')),
        _ => read_escape_atom(reader, next),
    }
}

fn push_literal(output: &mut String, character: Option<char>) {
    match character {
        Some(character) => {
            output.push_str(&regex::escape(character.encode_utf8(&mut [0; 4])));
        }
        None => output.push_str(NOTHING),
    }
}

fn unsupported(source: &str, construct: &str) -> Error {
    Error::new(
        "invalid-options",
        format!(
            "Cannot compile /{source}/i: {construct} has no equivalent in Rust's `regex`"
        ),
    )
}

#[cfg(test)]
#[path = "compile_unit.rs"]
mod unit;
