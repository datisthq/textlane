use std::collections::HashMap;

use crate::models::construct::{Resolver, State, StateFn};
use crate::models::event::{ContentType, Event, Kind, Link, Point};
use crate::models::token::Name;
use crate::utils::tokenizer::edit_map::EditMap;
use crate::utils::tokenizer::preprocess::ParseState;
use crate::utils::tokenizer::registry::call_resolver;
use crate::utils::tokenizer::subtokenize::Subresult;

const TAB_SIZE: usize = 4;

/// A container the document content type has open.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Container {
    BlockQuote,
    ListItem,
}

#[derive(Debug)]
pub(crate) struct ContainerState {
    pub(crate) kind: Container,
    /// Whether a list item started with a blank line, so a blank line next ends it.
    pub(crate) blank_initial: bool,
    /// How far a list item's content is indented.
    pub(crate) size: usize,
}

/// Whether a label start opens a link or an image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LabelKind {
    Image,
    Link,
}

/// A `[` or `![`: the enter and exit of its events.
#[derive(Clone, Debug)]
pub(crate) struct LabelStart {
    pub(crate) kind: LabelKind,
    pub(crate) start: (usize, usize),
    pub(crate) inactive: bool,
}

/// A label that closed: its start's events and its end's events.
#[derive(Clone, Debug)]
pub(crate) struct Label {
    pub(crate) kind: LabelKind,
    pub(crate) start: (usize, usize),
    pub(crate) end: (usize, usize),
}

/// What a byte at a point means: a byte, a virtual space of a tab, or the `\r` of a CRLF.
#[derive(Debug, PartialEq)]
enum ByteAction {
    Normal(u8),
    Ignore,
    Insert(u8),
}

#[derive(Debug, PartialEq)]
enum AttemptKind {
    Attempt,
    Check,
}

#[derive(Debug)]
struct Attempt {
    ok: State,
    nok: State,
    kind: AttemptKind,
    progress: Option<Progress>,
}

#[derive(Clone, Debug)]
struct Progress {
    events_len: usize,
    stack_len: usize,
    previous: Option<u8>,
    current: Option<u8>,
    point: Point,
}

/// State constructs share while tokenizing, so state functions need no closures.
#[derive(Debug, Default)]
pub(crate) struct TokenizeState<'a> {
    pub(crate) document_child: Option<Box<Tokenizer<'a>>>,
    pub(crate) document_child_state: Option<State>,
    pub(crate) document_container_stack: Vec<ContainerState>,
    pub(crate) document_continued: usize,
    pub(crate) document_data_index: Option<usize>,
    pub(crate) document_exits: Vec<Option<Vec<Event>>>,
    pub(crate) document_lazy_accepting_before: bool,
    /// For each open container, the first at or after it that an empty line does not
    /// simply continue; cleared whenever the containers change.
    pub(crate) document_blank_ends: Option<Vec<usize>>,
    pub(crate) space_or_tab_connect: bool,
    pub(crate) space_or_tab_content: Option<ContentType>,
    pub(crate) space_or_tab_min: usize,
    pub(crate) space_or_tab_max: usize,
    pub(crate) space_or_tab_size: usize,
    pub(crate) space_or_tab_token: Option<Name>,
    pub(crate) space_or_tab_eol_connect: bool,
    pub(crate) space_or_tab_eol_content: Option<ContentType>,
    pub(crate) space_or_tab_eol_ok: bool,
    pub(crate) definitions: Vec<String>,
    /// Link and image label starts (`[`, `![`) not yet closed.
    pub(crate) label_starts: Vec<LabelStart>,
    /// Label starts a label end could not close, turned into data later.
    pub(crate) label_starts_loose: Vec<LabelStart>,
    /// Link label starts before this index are inactive: links may not contain links.
    pub(crate) label_starts_inactive_before: usize,
    /// Labels that closed into links or images.
    pub(crate) labels: Vec<Label>,
    /// Token names partials (label, destination, title) emit, set by their caller.
    pub(crate) token_1: Name,
    pub(crate) token_2: Name,
    pub(crate) token_3: Name,
    pub(crate) token_4: Name,
    pub(crate) token_5: Name,
    /// Inline HTML comments, CDATA, declarations and instructions that scanned to the end
    /// without closing: none of that kind can close after them either.
    pub(crate) html_text_unclosed: [bool; 4],
    pub(crate) connect: bool,
    pub(crate) seen: bool,
    pub(crate) start: usize,
    pub(crate) marker_b: u8,
    pub(crate) end: usize,
    pub(crate) marker: u8,
    pub(crate) markers: &'static [u8],
    pub(crate) size: usize,
    pub(crate) size_b: usize,
    pub(crate) size_c: usize,
    /// The last start of a backtick run of each length, as code spans scan for closers.
    pub(crate) code_text_runs: HashMap<usize, usize>,
    /// Whether a code span scanned to the end of the content without closing.
    pub(crate) code_text_scanned: bool,
    /// Where a thematic break of a marker started and where it then failed.
    pub(crate) thematic_break_failed: Option<(u8, usize, usize)>,
}

/// Feeds bytes to state functions and records the events they emit.
#[derive(Debug)]
pub(crate) struct Tokenizer<'a> {
    column_start: Vec<(usize, usize)>,
    first_line: usize,
    line_start: Point,
    attempts: Vec<Attempt>,
    pub(crate) current: Option<u8>,
    pub(crate) previous: Option<u8>,
    pub(crate) point: Point,
    pub(crate) events: Vec<Event>,
    pub(crate) stack: Vec<Name>,
    pub(crate) map: EditMap,
    pub(crate) resolvers: Vec<Resolver>,
    pub(crate) parse_state: &'a ParseState<'a>,
    pub(crate) tokenize_state: TokenizeState<'a>,
    pub(crate) interrupt: bool,
    pub(crate) concrete: bool,
    pub(crate) pierce: bool,
    pub(crate) lazy: bool,
}

impl<'a> Tokenizer<'a> {
    pub(crate) fn new(point: Point, parse_state: &'a ParseState<'a>) -> Tokenizer<'a> {
        Tokenizer {
            column_start: vec![],
            first_line: point.line,
            line_start: point.clone(),
            attempts: vec![],
            current: None,
            previous: None,
            point,
            events: vec![],
            stack: vec![],
            map: EditMap::new(),
            resolvers: vec![],
            parse_state,
            tokenize_state: TokenizeState::default(),
            interrupt: false,
            concrete: false,
            pierce: false,
            lazy: false,
        }
    }

    pub(crate) fn register_resolver(&mut self, resolver: Resolver) {
        if !self.resolvers.contains(&resolver) {
            self.resolvers.push(resolver);
        }
    }

    pub(crate) fn register_resolver_before(&mut self, resolver: Resolver) {
        if !self.resolvers.contains(&resolver) {
            self.resolvers.insert(0, resolver);
        }
    }

    /// Skip to `point` whenever a new line starts, past the prefixes of open containers.
    pub(crate) fn define_skip(&mut self, mut point: Point) {
        move_point_back(self, &mut point);
        let info = (point.index, point.vs);
        let at = point.line - self.first_line;
        if at >= self.column_start.len() {
            self.column_start.push(info);
        } else {
            self.column_start[at] = info;
        }
        self.account_for_potential_skip();
    }

    fn account_for_potential_skip(&mut self) {
        let at = self.point.line - self.first_line;
        if self.point.column == 1 && at != self.column_start.len() {
            self.move_to(self.column_start[at]);
        }
    }

    pub(crate) fn consume(&mut self) {
        self.move_one();
        self.previous = self.current;
        self.current = None;
    }

    fn move_one(&mut self) {
        match byte_action(self.parse_state.bytes, &self.point) {
            ByteAction::Ignore => {
                self.point.index += 1;
            }
            ByteAction::Insert(byte) => {
                self.previous = Some(byte);
                self.point.column += 1;
                self.point.vs += 1;
            }
            ByteAction::Normal(byte) => {
                self.previous = Some(byte);
                self.point.vs = 0;
                self.point.index += 1;
                if byte == b'\n' {
                    self.point.line += 1;
                    self.point.column = 1;
                    if self.point.line - self.first_line + 1 > self.column_start.len() {
                        self.column_start.push((self.point.index, self.point.vs));
                    }
                    self.line_start = self.point.clone();
                    self.account_for_potential_skip();
                } else {
                    self.point.column += 1;
                }
            }
        }
    }

    fn move_to(&mut self, to: (usize, usize)) {
        let (to_index, to_vs) = to;
        while self.point.index < to_index
            || self.point.index == to_index && self.point.vs < to_vs
        {
            self.move_one();
        }
    }

    pub(crate) fn enter(&mut self, name: Name) {
        self.enter_impl(name, None);
    }

    pub(crate) fn enter_link(&mut self, name: Name, link: Link) {
        self.enter_impl(name, Some(link));
    }

    fn enter_impl(&mut self, name: Name, link: Option<Link>) {
        let mut point = self.point.clone();
        move_point_back(self, &mut point);
        self.stack.push(name);
        self.events.push(Event {
            kind: Kind::Enter,
            name,
            point,
            link,
        });
    }

    pub(crate) fn exit(&mut self, name: Name) {
        self.stack.pop();
        let mut point = self.point.clone();
        // An exit right after a line ending sits at the start of the next line
        if matches!(self.previous, Some(b'\n')) {
            point = self.line_start.clone();
        } else {
            move_point_back(self, &mut point);
        }
        self.events.push(Event {
            kind: Kind::Exit,
            name,
            point,
            link: None,
        });
    }

    fn capture(&self) -> Progress {
        Progress {
            previous: self.previous,
            current: self.current,
            point: self.point.clone(),
            events_len: self.events.len(),
            stack_len: self.stack.len(),
        }
    }

    fn free(&mut self, progress: Progress) {
        self.previous = progress.previous;
        self.current = progress.current;
        self.point = progress.point;
        self.events.truncate(progress.events_len);
        self.stack.truncate(progress.stack_len);
    }

    /// Try what follows and always revert it, then go to `ok` or `nok`.
    pub(crate) fn check(&mut self, ok: State, nok: State) {
        let progress = Some(self.capture());
        self.attempts.push(Attempt {
            kind: AttemptKind::Check,
            progress,
            ok,
            nok,
        });
    }

    /// Try what follows, revert it when it fails, then go to `ok` or `nok`.
    pub(crate) fn attempt(&mut self, ok: State, nok: State) {
        let progress = if matches!(nok, State::Nok) {
            None
        } else {
            Some(self.capture())
        };
        self.attempts.push(Attempt {
            kind: AttemptKind::Attempt,
            progress,
            ok,
            nok,
        });
    }

    /// Feed the bytes from `from` to `to` through `state`.
    pub(crate) fn push(
        &mut self,
        from: (usize, usize),
        to: (usize, usize),
        state: State,
    ) -> State {
        self.push_impl(from, to, state, false)
    }

    /// Feed the end of input, then run the registered resolvers when `resolve` is set.
    pub(crate) fn flush(&mut self, state: State, resolve: bool) -> Subresult {
        let to = (self.point.index, self.point.vs);
        self.push_impl(to, to, state, true);
        let mut value = Subresult {
            done: false,
            definitions: self.tokenize_state.definitions.split_off(0),
        };
        if resolve {
            let resolvers = self.resolvers.split_off(0);
            for resolver in resolvers {
                if let Some(mut result) = call_resolver(self, resolver) {
                    value.definitions.append(&mut result.definitions);
                }
            }
            self.map.consume(&mut self.events);
        }
        value
    }

    fn push_impl(
        &mut self,
        from: (usize, usize),
        to: (usize, usize),
        mut state: State,
        flush: bool,
    ) -> State {
        self.move_to(from);
        loop {
            match state {
                State::Ok | State::Nok => {
                    let Some(attempt) = self.attempts.pop() else {
                        break;
                    };
                    if (attempt.kind == AttemptKind::Check || matches!(state, State::Nok))
                        && let Some(progress) = attempt.progress
                    {
                        self.free(progress);
                    }
                    state = if matches!(state, State::Ok) {
                        attempt.ok
                    } else {
                        attempt.nok
                    };
                }
                State::Next(function) => {
                    let action = if self.point.index < to.0
                        || (self.point.index == to.0 && self.point.vs < to.1)
                    {
                        Some(byte_action(self.parse_state.bytes, &self.point))
                    } else if flush {
                        None
                    } else {
                        break;
                    };
                    if let Some(ByteAction::Ignore) = action {
                        self.move_one();
                    } else {
                        self.current = match action {
                            Some(ByteAction::Insert(byte) | ByteAction::Normal(byte)) => {
                                Some(byte)
                            }
                            _ => None,
                        };
                        state = call_state(self, function);
                    }
                }
                State::Retry(function) => {
                    state = call_state(self, function);
                }
            }
        }
        state
    }
}

fn call_state(tokenizer: &mut Tokenizer, function: StateFn) -> State {
    function(tokenizer)
}

fn move_point_back(tokenizer: &Tokenizer, point: &mut Point) {
    while point.index > 0 {
        point.index -= 1;
        let action = byte_action(tokenizer.parse_state.bytes, point);
        if !matches!(action, ByteAction::Ignore) {
            point.index += 1;
            break;
        }
    }
}

fn byte_action(bytes: &[u8], point: &Point) -> ByteAction {
    let Some(&byte) = bytes.get(point.index) else {
        return ByteAction::Ignore;
    };
    if byte == b'\r' {
        if bytes.get(point.index + 1) == Some(&b'\n') {
            ByteAction::Ignore
        } else {
            ByteAction::Normal(b'\n')
        }
    } else if byte == b'\t' {
        let remainder = point.column % TAB_SIZE;
        let vs = if remainder == 0 {
            0
        } else {
            TAB_SIZE - remainder
        };
        if point.vs == 0 {
            if vs == 0 {
                ByteAction::Normal(byte)
            } else {
                ByteAction::Insert(byte)
            }
        } else if vs == 0 {
            ByteAction::Normal(b' ')
        } else {
            ByteAction::Insert(b' ')
        }
    } else {
        ByteAction::Normal(byte)
    }
}
