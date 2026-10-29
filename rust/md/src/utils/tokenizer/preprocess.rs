use crate::models::options::{GfmOptions, MathOptions, Matter};
use std::cell::{Cell, OnceCell, RefCell};

use std::collections::HashSet;
use textlane_core::Error;

use crate::helpers::identifier::normalize_identifier;
use crate::models::construct::State;
use crate::models::event::{Event, Point};
use crate::models::mdx::MdxScript;
use crate::utils::tokenizer::document::tokenize_document;
use crate::utils::tokenizer::driver::Tokenizer;
use crate::utils::tokenizer::recycle::take_events;
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
    /// GFM's options, when its constructs are on.
    pub(crate) gfm: Option<GfmOptions>,
    /// Whether definition lists are on.
    pub(crate) definition_list: bool,
    /// The normalized labels of GFM footnote definitions, all found before any text.
    pub(crate) gfm_footnote_definitions: HashSet<String>,
    /// The most characters a footnote definition's label has.
    pub(crate) gfm_footnote_chars_max: usize,
    /// Math's options, when its constructs are on.
    pub(crate) math: Option<MathOptions>,
    /// Whether directives are on.
    pub(crate) directive: bool,
    /// Prefixes of lines inside container directives, found after the document's own:
    /// merged into `prefixes` at the end.
    pub(crate) prefixes_nested: RefCell<Vec<(usize, usize)>>,
    /// Where content chunks start that must not join the content before them.
    pub(crate) content_breaks: RefCell<HashSet<usize>>,
    /// The kinds of frontmatter `md.frontmatter` adds, in order.
    pub(crate) frontmatter: Vec<Matter>,
    /// MDX's parser, when its constructs are on.
    pub(crate) mdx: Option<MdxScript>,
    /// Whether a syntax error stopped tokenizing; `error` holds it.
    pub(crate) failed: Cell<bool>,
    pub(crate) error: RefCell<Option<Error>>,
}

impl ParseState<'_> {
    /// Whether indented code is on: MDX turns it off, so any indent opens flow constructs.
    pub(crate) fn code_indented(&self) -> bool {
        self.mdx.is_none()
    }

    /// The most indent a flow construct may have before it.
    pub(crate) fn indent_max(&self) -> usize {
        if self.code_indented() { 3 } else { usize::MAX }
    }
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
        self.is_in(&self.definitions, self.definition_chars_max, start, end)
    }

    /// Whether the label text from byte `start` to `end` matches a GFM footnote
    /// definition, at the same cost as `is_defined`.
    pub(crate) fn is_gfm_footnote_defined(&self, start: usize, end: usize) -> bool {
        let set = &self.gfm_footnote_definitions;
        self.is_in(set, self.gfm_footnote_chars_max, start, end)
    }

    fn read_next_non_whitespace(&self) -> &[u32] {
        self.next_non_whitespace.get_or_init(|| {
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
        })
    }

    fn is_in(
        &self,
        set: &HashSet<String>,
        chars_max: usize,
        start: usize,
        end: usize,
    ) -> bool {
        if set.is_empty() {
            return false;
        }
        let next = self.read_next_non_whitespace();
        let is_whitespace = |byte: u8| matches!(byte, b'\t' | b'\n' | b'\r' | b' ');
        // A character takes at most four bytes, so a longer run is too long
        let bytes_max = chars_max.saturating_mul(4).saturating_add(4);
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
            if chars > chars_max {
                return false;
            }
            index = (next[run_end] as usize).min(end);
        }
        set.contains(&normalize_identifier(&collapsed))
    }
}

/// A document's events and its container prefixes, as where each prefixed line starts and
/// where its content does.
pub(crate) type Tokenized = (Vec<Event>, Vec<(usize, usize)>);

/// Tokenize a document into its events, subtokenizing until no linked content is left.
pub(crate) fn tokenize_input(
    value: &str,
    gfm: Option<&GfmOptions>,
    math: Option<&MathOptions>,
    directive: bool,
    frontmatter: &[Matter],
    definition_list: bool,
    mdx: Option<&MdxScript>,
) -> Result<Tokenized, Error> {
    check_length(value.len())?;
    let mut parse_state = ParseState {
        bytes: value.as_bytes(),
        definitions: HashSet::new(),
        definition_chars_max: 0,
        next_non_whitespace: OnceCell::new(),
        prefixes: RefCell::new(vec![]),
        gfm: gfm.cloned(),
        definition_list,
        gfm_footnote_definitions: HashSet::new(),
        gfm_footnote_chars_max: 0,
        math: math.cloned(),
        directive,
        prefixes_nested: RefCell::new(vec![]),
        content_breaks: RefCell::new(HashSet::new()),
        frontmatter: frontmatter.to_vec(),
        mdx: mdx.cloned(),
        failed: Cell::new(false),
        error: RefCell::new(None),
    };
    let start = Point {
        line: 1,
        column: 1,
        index: 0,
        vs: 0,
    };
    let (mut events, mut result, footnotes) = {
        let mut tokenizer = Tokenizer::new(start, &parse_state);
        tokenizer.events = take_events();
        let state = tokenizer.push(
            (0, 0),
            (parse_state.bytes.len(), 0),
            State::Next(tokenize_document),
        );
        let result = tokenizer.flush(state, true);
        let footnotes =
            std::mem::take(&mut tokenizer.tokenize_state.gfm_footnote_definitions);
        (tokenizer.events, result, footnotes)
    };
    if let Some(error) = parse_state.error.take() {
        return Err(error);
    }
    for label in footnotes {
        parse_state.gfm_footnote_chars_max = parse_state
            .gfm_footnote_chars_max
            .max(label.chars().count());
        parse_state.gfm_footnote_definitions.insert(label);
    }
    loop {
        parse_state.add_definitions(std::mem::take(&mut result.definitions));
        if result.done {
            return Ok((events, merge_prefixes(&parse_state)));
        }
        result = subtokenize(&mut events, &parse_state, None);
        if let Some(error) = parse_state.error.take() {
            return Err(error);
        }
    }
}

// A line inside a container directive has the document's prefix and its content's: the
// longer one ends where the line's content starts
fn merge_prefixes(parse_state: &ParseState) -> Vec<(usize, usize)> {
    let mut prefixes = parse_state.prefixes.take();
    let nested = parse_state.prefixes_nested.take();
    if nested.is_empty() {
        return prefixes;
    }
    prefixes.extend(nested);
    prefixes.sort_unstable();
    let mut merged: Vec<(usize, usize)> = Vec::with_capacity(prefixes.len());
    for (line_start, content_start) in prefixes {
        match merged.last_mut() {
            Some(last) if last.0 == line_start => last.1 = last.1.max(content_start),
            _ => merged.push((line_start, content_start)),
        }
    }
    merged
}

// Points hold lines, columns and offsets as `u32`, and a tab adds up to four columns per
// byte: past 1 GiB a column could wrap, which core's `Point::MAX_OFFSET` caps at anyway
pub(crate) fn check_length(len: usize) -> Result<(), Error> {
    if len > MAX_INPUT {
        return Err(Error::new(
            "input-too-long",
            "Cannot parse Markdown longer than 1 GiB",
        ));
    }
    Ok(())
}

const MAX_INPUT: usize = (1 << 30) - 1;

#[cfg(test)]
#[path = "preprocess_unit.rs"]
mod unit;
