use std::cell::{OnceCell, RefCell};
use std::collections::HashSet;

use crate::helpers::identifier::normalize_identifier;
use crate::models::construct::State;
use crate::models::event::{Event, Point};
use crate::utils::tokenizer::document::tokenize_document;
use crate::utils::tokenizer::driver::Tokenizer;
use crate::utils::tokenizer::subtokenize::subtokenize;

/// What every tokenizer of one document shares.
#[derive(Debug)]
pub(crate) struct ParseState<'a> {
    pub(crate) bytes: &'a [u8],
    /// The normalized identifiers of the definitions found so far.
    pub(crate) definitions: HashSet<String>,
    /// The most characters a definition's identifier has.
    pub(crate) definition_chars_max: usize,
    /// For each byte, the first byte at or after it that is not whitespace, built when
    /// first needed: labels skip whitespace runs through it.
    pub(crate) next_non_whitespace: OnceCell<Vec<u32>>,
    /// Container prefixes, as where each prefixed line starts and where its content does:
    /// slices of tokens that span lines leave them out, as micromark's chunks do.
    pub(crate) prefixes: RefCell<Vec<(usize, usize)>>,
}

impl ParseState<'_> {
    fn add_definitions(&mut self, definitions: Vec<String>) {
        for definition in definitions {
            self.definition_chars_max =
                self.definition_chars_max.max(definition.chars().count());
            self.definitions.insert(definition);
        }
    }

    /// Whether the label text from byte `start` to `end` matches a definition; it costs
    /// the length of the longest identifier, not of the label.
    pub(crate) fn is_defined(&self, start: usize, end: usize) -> bool {
        if self.definitions.is_empty() {
            return false;
        }
        let next = self.next_non_whitespace.get_or_init(|| {
            let mut next = vec![0u32; self.bytes.len() + 1];
            let mut at = self.bytes.len() as u32;
            next[self.bytes.len()] = at;
            for index in (0..self.bytes.len()).rev() {
                if !matches!(self.bytes[index], b'\t' | b'\n' | b'\r' | b' ') {
                    at = index as u32;
                }
                next[index] = at;
            }
            next
        });
        let is_whitespace = |byte: u8| matches!(byte, b'\t' | b'\n' | b'\r' | b' ');
        // A character takes at most four bytes, so a longer run is too long
        let bytes_max = self
            .definition_chars_max
            .saturating_mul(4)
            .saturating_add(4);
        let mut collapsed = String::new();
        let mut chars = 0;
        let mut index = next[start] as usize;
        while index < end {
            let mut run_end = index;
            while run_end < end && !is_whitespace(self.bytes[run_end]) {
                run_end += 1;
                if run_end - index > bytes_max {
                    return false;
                }
            }
            let Ok(run) = std::str::from_utf8(&self.bytes[index..run_end]) else {
                return false;
            };
            if !collapsed.is_empty() {
                collapsed.push(' ');
            }
            collapsed.push_str(run);
            chars += run.chars().count();
            if chars > self.definition_chars_max {
                return false;
            }
            index = (next[run_end] as usize).min(end);
        }
        self.definitions.contains(&normalize_identifier(&collapsed))
    }
}

/// Tokenize a document into its events, subtokenizing until no linked content is left.
pub(crate) fn tokenize_input(value: &str) -> (Vec<Event>, Vec<(usize, usize)>) {
    let mut parse_state = ParseState {
        bytes: value.as_bytes(),
        definitions: HashSet::new(),
        definition_chars_max: 0,
        next_non_whitespace: OnceCell::new(),
        prefixes: RefCell::new(vec![]),
    };
    let start = Point {
        line: 1,
        column: 1,
        index: 0,
        vs: 0,
    };
    let (mut events, mut result) = {
        let mut tokenizer = Tokenizer::new(start, &parse_state);
        let state = tokenizer.push(
            (0, 0),
            (parse_state.bytes.len(), 0),
            State::Next(tokenize_document),
        );
        let result = tokenizer.flush(state, true);
        (tokenizer.events, result)
    };
    loop {
        parse_state.add_definitions(std::mem::take(&mut result.definitions));
        if result.done {
            return (events, parse_state.prefixes.take());
        }
        result = subtokenize(&mut events, &parse_state, None);
    }
}
