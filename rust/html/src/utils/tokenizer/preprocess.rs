use crate::models::parse_error::{ErrorCode, ParseError};

/// parse5's input stream preprocessor: reads one character at a time, turning CR and CRLF
/// into LF and reporting control characters and noncharacters.
#[derive(Debug)]
pub(crate) struct Preprocessor<'s> {
    pub(crate) input: &'s str,
    /// Where the current character starts; past the end once EOF was read, one byte
    /// further for each read after it, as parse5's `pos` keeps counting.
    pub(crate) pos: usize,
    next: usize,
    skip_next_line_feed: bool,
    last_error: Option<usize>,
    /// Whether the input ends the document, not a chunk more may follow.
    pub(crate) last_chunk: bool,
    /// Whether a read ran past the end of a chunk that is not the last: parse5's sticky
    /// `endOfChunkHit`, after which the tokenizer waits for the next chunk.
    pub(crate) chunk_end: bool,
}

impl<'s> Preprocessor<'s> {
    pub(crate) fn new(input: &'s str) -> Self {
        Self {
            input,
            pos: 0,
            next: 0,
            skip_next_line_feed: false,
            last_error: None,
            last_chunk: true,
            chunk_end: false,
        }
    }

    /// Reads the next character, `None` at the end of the input.
    pub(crate) fn advance(&mut self, errors: &mut Vec<ParseError>) -> Option<char> {
        loop {
            self.pos = self.next;
            let Some(char) = self
                .input
                .get(self.pos..)
                .and_then(|rest| rest.chars().next())
            else {
                self.next = self.pos + 1;
                self.chunk_end = !self.last_chunk;
                return None;
            };
            self.next = self.pos + char.len_utf8();
            match char {
                '\r' => {
                    self.skip_next_line_feed = true;
                    return Some('\n');
                }
                '\n' if self.skip_next_line_feed => {
                    self.skip_next_line_feed = false;
                }
                _ => {
                    self.skip_next_line_feed = false;
                    self.check_character(char, errors);
                    return Some(char);
                }
            }
        }
    }

    /// Moves back or forward to the ASCII character at `pos`, as the tokenizer does around
    /// a character reference.
    pub(crate) fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
        self.next = pos + 1;
    }

    /// Where `pattern` ends if the input continues with it from the current character,
    /// comparing UTF-16 units as parse5 does: case-insensitively by setting bit `0x20`.
    pub(crate) fn match_sequence(
        &mut self,
        pattern: &str,
        case_sensitive: bool,
    ) -> Option<usize> {
        let mut expected = pattern.encode_utf16();
        let mut end = self.pos;
        let mut units = [0; 2];
        let input = self.input;
        let rest = input.get(self.pos..)?;
        for char in rest.chars() {
            for &unit in char.encode_utf16(&mut units).iter() {
                let Some(want) = expected.next() else {
                    return Some(end);
                };
                let unit = if case_sensitive { unit } else { unit | 0x20 };
                if unit != want {
                    return None;
                }
            }
            end += char.len_utf8();
        }
        if expected.next().is_some() {
            self.chunk_end = !self.last_chunk;
            return None;
        }
        Some(end)
    }

    /// The character at byte `pos`, with CR read as LF.
    pub(crate) fn peek_at(&mut self, pos: usize) -> Option<char> {
        let Some(char) = self.input.get(pos..).and_then(|rest| rest.chars().next())
        else {
            self.chunk_end = !self.last_chunk;
            return None;
        };
        Some(if char == '\r' { '\n' } else { char })
    }

    fn check_character(&mut self, char: char, errors: &mut Vec<ParseError>) {
        let code = u32::from(char);
        let common = (0x20..0x7f).contains(&code)
            || char == '\n'
            || (0xa0..0xfdd0).contains(&code);
        if common {
            return;
        }
        let error = if is_control(code) {
            ErrorCode::ControlCharacterInInputStream
        } else if is_noncharacter(code) {
            ErrorCode::NoncharacterInInputStream
        } else {
            return;
        };
        // A character read again after a reference was rewound is reported once
        if self.last_error != Some(self.pos) {
            self.last_error = Some(self.pos);
            errors.push(ParseError {
                code: error,
                start: self.pos,
                end: self.pos,
            });
        }
    }
}

/// A control character other than NUL and ASCII whitespace, as parse5's
/// `isControlCodePoint`.
pub(crate) fn is_control(code: u32) -> bool {
    ((0x01..=0x1f).contains(&code) && !matches!(code, 0x09 | 0x0a | 0x0c | 0x0d))
        || (0x7f..=0x9f).contains(&code)
}

pub(crate) fn is_noncharacter(code: u32) -> bool {
    (0xfdd0..=0xfdef).contains(&code) || (code & 0xfffe == 0xfffe && code <= 0x10ffff)
}
