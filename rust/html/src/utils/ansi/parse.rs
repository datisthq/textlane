// A port of ansi-sequence-parser 1.1.3, which Shiki bundles to read `ansi` code

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum AnsiColor {
    // An index into the 16 named colors, black to bright white
    Named(usize),
    Rgb([u8; 3]),
    // An index into the 256-color table, `None` where `Number()` gives no entry
    Table(Option<u8>),
}

pub(crate) const BOLD: u8 = 1;
pub(crate) const DIM: u8 = 1 << 1;
pub(crate) const ITALIC: u8 = 1 << 2;
pub(crate) const UNDERLINE: u8 = 1 << 3;
pub(crate) const REVERSE: u8 = 1 << 4;
const HIDDEN: u8 = 1 << 5;
pub(crate) const STRIKETHROUGH: u8 = 1 << 6;
const OVERLINE: u8 = 1 << 7;

#[derive(Debug, PartialEq)]
pub(crate) struct AnsiSegment {
    pub(crate) text: String,
    pub(crate) foreground: Option<AnsiColor>,
    pub(crate) background: Option<AnsiColor>,
    pub(crate) decorations: u8,
}

enum Command {
    ResetAll,
    ResetForeground,
    ResetBackground,
    ResetDecoration(u8),
    SetForeground(AnsiColor),
    SetBackground(AnsiColor),
    SetDecoration(u8),
}

/// The parser's state, which carries over from one line to the next as Shiki's does.
#[derive(Default)]
pub(crate) struct AnsiParser {
    foreground: Option<AnsiColor>,
    background: Option<AnsiColor>,
    decorations: u8,
}

impl AnsiParser {
    /// Split a line at its `ESC[…m` sequences into the text each style covers.
    pub(crate) fn parse(&mut self, value: &str) -> Vec<AnsiSegment> {
        let mut segments = vec![];
        let mut position = 0;
        loop {
            let found = find_sequence(value, position);
            let end = found.map_or(value.len(), |(start, _, _)| start);
            if end > position {
                segments.push(AnsiSegment {
                    text: value[position..end].to_owned(),
                    foreground: self.foreground,
                    background: self.background,
                    decorations: self.decorations,
                });
            }
            let Some((_, sequence, next)) = found else {
                break;
            };
            self.apply(&parse_sequence(sequence));
            position = next;
            if position >= value.len() {
                break;
            }
        }
        segments
    }

    // Every reset in a sequence comes before every set, whatever their order
    fn apply(&mut self, commands: &[Command]) {
        for command in commands {
            match command {
                Command::ResetAll => *self = Self::default(),
                Command::ResetForeground => self.foreground = None,
                Command::ResetBackground => self.background = None,
                Command::ResetDecoration(flag) => self.decorations &= !flag,
                _ => {}
            }
        }
        for command in commands {
            match command {
                Command::SetForeground(color) => self.foreground = Some(*color),
                Command::SetBackground(color) => self.background = Some(*color),
                Command::SetDecoration(flag) => self.decorations |= flag,
                _ => {}
            }
        }
    }
}

// The next `ESC[…m` from `position`: where it starts, its parameters and where it ends;
// an escape that starts no such sequence leaves the rest of the line as text
fn find_sequence(value: &str, position: usize) -> Option<(usize, &str, usize)> {
    let escape = position + value[position..].find('\x1b')?;
    if value.as_bytes().get(escape + 1) != Some(&b'[') {
        return None;
    }
    let close = escape + value[escape..].find('m')?;
    Some((escape, &value[escape + 2..close], close + 1))
}

fn parse_sequence(sequence: &str) -> Vec<Command> {
    let parts: Vec<&str> = sequence.split(';').collect();
    let mut commands = vec![];
    let mut index = 0;
    while index < parts.len() {
        let code = parts[index];
        index += 1;
        let Some(code) = parse_int(code) else {
            continue;
        };
        let named = |offset: i64| Some(AnsiColor::Named(usize::try_from(offset).ok()?));
        let command = match code {
            0 => Some(Command::ResetAll),
            ..=-1 | 1..=9 => decoration(code).map(Command::SetDecoration),
            10..=29 => match decoration(code - 20) {
                // 22 ends bold as well as dim
                Some(DIM) => {
                    commands.push(Command::ResetDecoration(DIM));
                    Some(Command::ResetDecoration(BOLD))
                }
                flag => flag.map(Command::ResetDecoration),
            },
            30..=37 => named(code - 30).map(Command::SetForeground),
            38 => parse_color(&parts, &mut index).map(Command::SetForeground),
            39 => Some(Command::ResetForeground),
            40..=47 => named(code - 40).map(Command::SetBackground),
            48 => parse_color(&parts, &mut index).map(Command::SetBackground),
            49 => Some(Command::ResetBackground),
            53 => Some(Command::SetDecoration(OVERLINE)),
            55 => Some(Command::ResetDecoration(OVERLINE)),
            90..=97 => named(code - 90 + 8).map(Command::SetForeground),
            100..=107 => named(code - 100 + 8).map(Command::SetBackground),
            _ => None,
        };
        commands.extend(command);
    }
    commands
}

fn decoration(code: i64) -> Option<u8> {
    match code {
        1 => Some(BOLD),
        2 => Some(DIM),
        3 => Some(ITALIC),
        4 => Some(UNDERLINE),
        7 => Some(REVERSE),
        8 => Some(HIDDEN),
        9 => Some(STRIKETHROUGH),
        _ => None,
    }
}

// `38;2;r;g;b` and `38;5;n`; the parameters are taken even when they make no color
fn parse_color(parts: &[&str], index: &mut usize) -> Option<AnsiColor> {
    let mode = parts.get(*index).copied();
    *index += 1;
    match mode {
        Some("2") => {
            let end = (*index + 3).min(parts.len());
            let values = &parts[*index..end];
            *index = end;
            let mut rgb = [0; 3];
            if values.len() != 3 {
                return None;
            }
            for (slot, value) in rgb.iter_mut().zip(values) {
                *slot = u8::try_from(parse_int(value)?.clamp(0, 255)).ok()?;
            }
            Some(AnsiColor::Rgb(rgb))
        }
        Some("5") => {
            let value = parts.get(*index).copied();
            *index += 1;
            value
                .filter(|value| !value.is_empty())
                .map(|value| AnsiColor::Table(parse_index(value)))
        }
        _ => None,
    }
}

// JavaScript's `Number.parseInt` without a radix, saturating where it would not fit
fn parse_int(value: &str) -> Option<i64> {
    let value = value.trim_start_matches(is_js_space);
    let (negative, value) = match value.as_bytes().first() {
        Some(b'-') => (true, &value[1..]),
        Some(b'+') => (false, &value[1..]),
        _ => (false, value),
    };
    let (radix, value) = match value.get(..2) {
        Some("0x" | "0X") => (16, &value[2..]),
        _ => (10, value),
    };
    let mut result: Option<i64> = None;
    for digit in value.chars().map_while(|c| c.to_digit(radix)) {
        let next = result.unwrap_or(0);
        result = Some(
            next.saturating_mul(i64::from(radix))
                .saturating_add(digit.into()),
        );
    }
    result.map(|result| if negative { -result } else { result })
}

// The 256-color table entry JavaScript's `Number(value)` names, if any
fn parse_index(value: &str) -> Option<u8> {
    let value = value.trim_matches(is_js_space);
    if value.is_empty() {
        return Some(0);
    }
    let radix = match value.get(..2) {
        Some("0x" | "0X") => Some(16),
        Some("0o" | "0O") => Some(8),
        Some("0b" | "0B") => Some(2),
        _ => None,
    };
    if let Some(radix) = radix {
        return u8::from_str_radix(&value[2..], radix)
            .ok()
            .filter(|_| value[2..].bytes().all(|b| b.is_ascii_alphanumeric()));
    }
    // Rust reads `inf` and `nan`, which JavaScript does not; neither names an entry
    if value
        .bytes()
        .any(|b| b.is_ascii_alphabetic() && b != b'e' && b != b'E')
    {
        return None;
    }
    let number = value.parse::<f64>().ok()?;
    (number.fract() == 0.0 && (0.0..256.0).contains(&number)).then_some(number as u8)
}

// JavaScript's white space and line terminators
fn is_js_space(c: char) -> bool {
    (c.is_whitespace() && c != '\u{85}') || c == '\u{FEFF}'
}

#[cfg(test)]
#[path = "parse_unit.rs"]
mod unit;
