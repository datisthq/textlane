use std::collections::HashSet;

use textlane_core::Point;

use crate::actions::markdown::parse::ParseSettings;
use crate::helpers::decode::decode_string;
use crate::helpers::identifier::normalize_identifier;
use crate::utils::onepass::directive::{DirectiveHead, scan_head};
use crate::utils::onepass::mdx::{
    MdxScan, MdxText, ScannedAttribute, ScannedTag, check_esm,
};
use crate::utils::onepass::scan::{
    is_blank_from, is_space_or_tab, scan_atx, scan_colon_fence, scan_delimiter_row,
    scan_destination, scan_fence_close, scan_fence_open, scan_footnote_label,
    scan_head_row, scan_html_end, scan_html_start, scan_label, scan_list_marker,
    scan_setext, scan_thematic_break, scan_title, skip_space_eol, trim_atx_content,
};

pub(crate) const NONE: u32 = u32::MAX;

/// A line of a leaf: its content from `start` (`pad` spaces first when it starts inside a
/// tab, `virt` columns into it) to `end`, its line ending to `next`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Line {
    pub(crate) begin: usize,
    pub(crate) start: usize,
    pub(crate) virt: u8,
    pub(crate) pad: u8,
    pub(crate) end: usize,
    pub(crate) next: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Kind {
    Document,
    BlockQuote,
    List {
        ordered: bool,
        marker: u8,
        start: u32,
    },
    Item {
        width: usize,
    },
    Paragraph,
    Heading {
        depth: u8,
        setext: bool,
    },
    ThematicBreak,
    Fenced {
        marker: u8,
        size: usize,
        indent: usize,
        info: (usize, usize),
    },
    Indented,
    Html {
        kind: u8,
    },
    Definition {
        index: u32,
    },
    /// A GFM footnote definition; its label's raw bytes.
    Footnote {
        label: (usize, usize),
    },
    /// A GFM table: its head row, delimiter row and body rows are its lines.
    Table,
    /// A leaf directive, `::name`, by its head in `Blocks::directives`.
    LeafDirective {
        index: u32,
    },
    /// A container directive: its head, its fence's size and indent.
    ContainerDirective {
        index: u32,
        size: usize,
        prefix: usize,
    },
    /// A definition term: one line of the paragraph above a description.
    DefTerm,
    /// A definition description: its content's indent, whether its first line was blank,
    /// and its spread.
    DefDescription {
        width: usize,
        blank_initial: bool,
    },
    /// ESM: its raw bytes.
    Esm {
        value: (usize, usize),
    },
    /// MDX tags and expressions alone on their lines: `mdx_items[first..last]`.
    MdxFlow {
        first: u32,
        last: u32,
    },
    /// Frontmatter of the `matter`th kind; its value's raw bytes.
    Frontmatter {
        matter: u32,
        value: (usize, usize),
    },
}

#[derive(Clone, Debug)]
pub(crate) struct Block {
    pub(crate) kind: Kind,
    pub(crate) parent: u32,
    pub(crate) first: u32,
    pub(crate) last: u32,
    pub(crate) next: u32,
    prev: u32,
    pub(crate) start: Point,
    pub(crate) end: Point,
    pub(crate) lines: (u32, u32),
    pub(crate) spread: bool,
    /// An unclosed fence or HTML that took its last line ending.
    pub(crate) swallowed: bool,
    pub(crate) ended: bool,
    eol_end: usize,
    open: bool,
    blank: bool,
    // Blank lines in a row a container directive holds
    blank_run: u32,
    trailing_blank: u32,
    // Where a paragraph's first line starts after its container prefixes, indent included
    prefix_end: usize,
    opened_line: usize,
}

/// A definition, decoded.
#[derive(Debug)]
pub(crate) struct Def {
    pub(crate) label: String,
    pub(crate) identifier: String,
    pub(crate) url: String,
    pub(crate) title: Option<String>,
}

/// The block structure of a document, parsed line by line as cmark does, with the leaves'
/// lines kept for the inline pass.
pub(crate) struct Blocks<'s> {
    pub(crate) bytes: &'s [u8],
    mdx: bool,
    /// A syntax error that stopped parsing.
    pub(crate) error: Option<textlane_core::Error>,
    /// The MDX tags and expressions of flow lines, by block.
    pub(crate) mdx_items: Vec<FlowItem>,
    imports: Vec<String>,
    source: &'s str,
    pub(crate) settings: &'s ParseSettings<'s>,
    pub(crate) blocks: Vec<Block>,
    pub(crate) lines: Vec<Line>,
    pub(crate) defs: Vec<Def>,
    pub(crate) identifiers: HashSet<String>,
    pub(crate) longest_identifier: usize,
    /// GFM footnote labels, normalized.
    pub(crate) footnotes: HashSet<String>,
    pub(crate) directives: Vec<DirectiveHead>,
    tip: usize,
    // Bumped by every change to the open blocks, so a blank line can reuse the last match
    version: u64,
    blank_match: Option<(u64, usize, bool)>,
    line_start: usize,
    line_end: usize,
    line_next: usize,
    offset: usize,
    column: usize,
    partial: bool,
    tab_column: usize,
    first_nonspace: usize,
    first_nonspace_column: usize,
    indent: usize,
    blank: bool,
    code_full: bool,
    // Whether a leaf closing now takes its last line ending: at the end, or when a new
    // container starts the line (micromark's non-lazy continuation)
    swallow: bool,
    // Where a container that a swallowing leaf ended in ends instead
    swallow_end: Option<Point>,
    // The swallowing leaf's end, and where items around it end
    cascade_end: Option<(Point, Point)>,
    eof: bool,
    // Where a list item's prefix ends when nothing follows its marker
    prefix_end: Option<Point>,
    skip_to: usize,
    // Matters that found no closing fence: none opening later finds one either
    matter_failed: Vec<bool>,
    // An unclosed matter scanned to the end with its lines concrete: as upstream, no
    // container opens after it
    no_containers: bool,
    // Lines that continued a paragraph lazily
    lazy_lines: Vec<u32>,
}

pub(crate) fn parse_blocks<'s>(
    source: &'s str,
    settings: &'s ParseSettings<'s>,
) -> Blocks<'s> {
    let bytes = source.as_bytes();
    let mut parser = Blocks {
        bytes,
        mdx: settings.mdx.is_some(),
        error: None,
        mdx_items: vec![],
        imports: vec![],
        source,
        settings,
        blocks: Vec::with_capacity(bytes.len() / 64 + 4),
        lines: Vec::with_capacity(bytes.len() / 32 + 4),
        defs: vec![],
        identifiers: HashSet::new(),
        longest_identifier: 0,
        footnotes: HashSet::new(),
        directives: Vec::new(),
        tip: 0,
        version: 0,
        blank_match: None,
        line_start: 0,
        line_end: 0,
        line_next: 0,
        offset: 0,
        column: 0,
        partial: false,
        tab_column: 0,
        first_nonspace: 0,
        first_nonspace_column: 0,
        indent: 0,
        blank: false,
        code_full: false,
        swallow: false,
        swallow_end: None,
        cascade_end: None,
        eof: false,
        prefix_end: None,
        skip_to: 0,
        matter_failed: vec![],
        no_containers: false,
        lazy_lines: vec![],
    };
    parser
        .blocks
        .push(Block::new(Kind::Document, NONE, Point::new(0, 0), 0));
    let mut start = 0;
    while start < bytes.len() {
        let mut end = start;
        while end < bytes.len() && bytes[end] != b'\n' && bytes[end] != b'\r' {
            end += 1;
        }
        let next = match bytes.get(end) {
            Some(b'\r') if bytes.get(end + 1) == Some(&b'\n') => end + 2,
            Some(_) => end + 1,
            None => end,
        };
        parser.process_line(start, end, next);
        if parser.error.is_some() {
            return parser;
        }
        // Frontmatter takes the lines through its closing fence at once
        start = next.max(parser.skip_to);
    }
    parser.swallow = true;
    parser.eof = true;
    while parser.tip != 0 {
        let parent = parser.blocks[parser.tip].parent as usize;
        parser.finalize(parser.tip);
        parser.tip = parent;
    }
    parser.blocks[0].end = Point::new(bytes.len(), 0);
    parser
}

impl Block {
    fn new(kind: Kind, parent: u32, start: Point, line: usize) -> Self {
        Block {
            kind,
            parent,
            first: NONE,
            last: NONE,
            next: NONE,
            prev: NONE,
            start,
            end: start,
            lines: (0, 0),
            spread: false,
            swallowed: false,
            ended: false,
            eol_end: 0,
            open: true,
            blank: false,
            blank_run: 0,
            prefix_end: 0,
            trailing_blank: 0,
            opened_line: line,
        }
    }

    fn accepts_lines(&self) -> bool {
        matches!(
            self.kind,
            Kind::Fenced { .. } | Kind::Indented | Kind::Html { .. }
        )
    }
}

impl Blocks<'_> {
    fn point(&self) -> Point {
        let virt = if self.partial {
            self.column - self.tab_column
        } else {
            0
        };
        Point::new(self.offset, virt)
    }

    fn peek(&self, at: usize) -> u8 {
        if at < self.line_end {
            self.bytes[at]
        } else {
            b'\n'
        }
    }

    fn find_first_nonspace(&mut self) {
        let mut index = self.offset;
        let mut column = self.column;
        let mut to_tab = 4 - (column % 4);
        while index < self.line_end {
            match self.bytes[index] {
                b' ' => {
                    index += 1;
                    column += 1;
                    to_tab -= 1;
                    if to_tab == 0 {
                        to_tab = 4;
                    }
                }
                b'\t' => {
                    index += 1;
                    column += to_tab;
                    to_tab = 4;
                }
                _ => break,
            }
        }
        self.first_nonspace = index;
        self.first_nonspace_column = column;
        self.indent = column - self.column;
        self.blank = index >= self.line_end;
    }

    fn advance_offset(&mut self, mut count: usize, columns: bool) {
        while count > 0 && self.offset < self.line_end {
            if self.bytes[self.offset] == b'\t' {
                let to_tab = 4 - (self.column % 4);
                if columns {
                    if !self.partial {
                        self.tab_column = self.column;
                    }
                    self.partial = to_tab > count;
                    let advance = to_tab.min(count);
                    self.column += advance;
                    if !self.partial {
                        self.offset += 1;
                    }
                    count -= advance;
                } else {
                    self.partial = false;
                    self.column += to_tab;
                    self.offset += 1;
                    count -= 1;
                }
            } else {
                self.partial = false;
                self.offset += 1;
                self.column += 1;
                count -= 1;
            }
        }
    }

    fn advance_to_nonspace(&mut self) {
        self.offset = self.first_nonspace;
        self.column = self.first_nonspace_column;
        self.partial = false;
    }

    fn process_line(&mut self, start: usize, end: usize, next: usize) {
        self.line_start = start;
        self.line_end = end;
        self.line_next = next;
        self.offset = start;
        self.column = 0;
        self.partial = false;
        self.code_full = false;

        let mut container = 0;
        let mut all_matched = true;
        let mut failed = NONE as usize;
        let mut quote: Option<(usize, usize)> = None;
        let blank_line = is_blank_from(self.bytes, start, end);
        // Blank lines through deep items match the same way until a block opens or closes
        let reuse = match self.blank_match {
            Some((version, block, all)) if blank_line && version == self.version => {
                Some((block, all))
            }
            _ => None,
        };
        let mut cacheable = blank_line;
        if let Some((block, all)) = reuse {
            container = block;
            all_matched = all;
            self.offset = end;
            self.column = 0;
        }
        while reuse.is_none() {
            let last = self.blocks[container].last;
            if last == NONE || !self.blocks[last as usize].open {
                break;
            }
            let child = last as usize;
            self.find_first_nonspace();
            if matches!(
                self.blocks[child].kind,
                Kind::Fenced { .. }
                    | Kind::Indented
                    | Kind::Html { kind: 1..=5 }
                    | Kind::Footnote { .. }
                    | Kind::ContainerDirective { .. }
                    | Kind::DefDescription { .. }
            ) || (matches!(self.blocks[child].kind, Kind::Item { .. })
                && self.blocks[child].first == NONE)
            {
                cacheable = false;
            }
            let matched = match self.blocks[child].kind {
                Kind::BlockQuote => {
                    if (self.indent <= 3 || self.mdx)
                        && self.peek(self.first_nonspace) == b'>'
                    {
                        self.advance_offset(self.indent + 1, true);
                        if is_space_or_tab(self.peek(self.offset)) {
                            self.advance_offset(1, true);
                        }
                        quote = Some((child, self.offset));
                        true
                    } else {
                        false
                    }
                }
                Kind::Item { width } => {
                    if self.indent >= width {
                        self.advance_offset(width, true);
                        true
                    } else if self.blank && self.blocks[child].first != NONE {
                        self.advance_to_nonspace();
                        true
                    } else {
                        false
                    }
                }
                Kind::List { .. } => true,
                Kind::Fenced {
                    marker,
                    size,
                    indent,
                    ..
                } => {
                    if (self.indent <= 3 || self.mdx)
                        && scan_fence_close(
                            self.bytes,
                            self.first_nonspace,
                            self.line_end,
                            marker,
                            size,
                        )
                    {
                        self.close_fence(child);
                        return;
                    }
                    let mut remaining = indent;
                    while remaining > 0 && is_space_or_tab(self.peek(self.offset)) {
                        self.advance_offset(1, true);
                        remaining -= 1;
                    }
                    true
                }
                Kind::Indented => {
                    if self.indent >= 4 {
                        self.advance_offset(4, true);
                        self.code_full = true;
                        true
                    } else if self.blank {
                        self.advance_to_nonspace();
                        true
                    } else {
                        false
                    }
                }
                Kind::Html { kind } => !(self.blank && kind >= 6),
                Kind::Paragraph | Kind::Table => !self.blank,
                Kind::DefDescription {
                    width,
                    blank_initial,
                } => {
                    if self.blank {
                        // Blank lines after a blank first line end it at the next filled one
                        if blank_initial {
                            self.blocks[child].blank_run = u32::MAX;
                        }
                        let columns = self.indent.min(width);
                        self.advance_offset(columns, true);
                        self.blocks[child].lines.0 = 1;
                        true
                    } else {
                        let further = self.blocks[child].blank_run == u32::MAX;
                        if let Kind::DefDescription { blank_initial, .. } =
                            &mut self.blocks[child].kind
                        {
                            *blank_initial = false;
                        }
                        if further || self.indent < width.max(1) {
                            false
                        } else {
                            self.blocks[child].lines.0 = 0;
                            self.advance_offset(width, true);
                            true
                        }
                    }
                }
                Kind::ContainerDirective { size, prefix, .. } => {
                    if (self.indent <= 3 || self.mdx)
                        && scan_colon_fence(
                            self.bytes,
                            self.first_nonspace,
                            self.line_end,
                        ) >= size
                        && scan_colon_fence(
                            self.bytes,
                            self.first_nonspace,
                            self.line_end,
                        ) > 0
                    {
                        self.blocks[child].ended = true;
                        self.blocks[child].spread |= self.blocks[child].blank;
                        // What is open inside ends with the directive's content, as at
                        // the end of a document
                        self.swallow = true;
                        self.swallow_end = None;
                        self.close_unmatched(child);
                        self.swallow = false;
                        self.touch(child, Point::new(self.line_end, 0));
                        let parent = self.blocks[child].parent as usize;
                        self.finalize(child);
                        self.tip = parent;
                        return;
                    }
                    let mut remaining = prefix;
                    while remaining > 0 && is_space_or_tab(self.peek(self.offset)) {
                        self.advance_offset(1, true);
                        remaining -= 1;
                    }
                    let block = &mut self.blocks[child];
                    block.eol_end = self.line_next;
                    block.trailing_blank = self.offset as u32;
                    block.lines.0 = u32::from(self.blank);
                    true
                }
                Kind::Footnote { .. } => {
                    if self.blank {
                        self.blocks[child].eol_end = self.offset;
                        let columns = self.indent.min(4);
                        self.advance_offset(columns, true);
                        true
                    } else if self.indent >= 4 {
                        self.advance_offset(4, true);
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            };
            if !matched {
                all_matched = false;
                failed = child;
                break;
            }
            container = child;
        }
        let mut last_matched = container;
        if cacheable && reuse.is_none() {
            self.blank_match = Some((self.version, container, all_matched));
        }

        let mut maybe_lazy = self.blocks[self.tip].kind == Kind::Paragraph;
        let mut first_iteration = true;
        // As micromark, whether new blocks interrupt is decided once a line, before any opens
        // micromark's indented code is still the flow's construct on the line after it
        let interrupt = self.blocks[last_matched].kind == Kind::Paragraph
            || (self.blocks[self.tip].kind == Kind::Indented
                && self.blocks[self.tip].parent as usize == last_matched);
        let mut done = false;
        while !self.blocks[container].accepts_lines() {
            self.find_first_nonspace();
            let indented = self.indent >= 4 && !self.mdx;
            let at = self.first_nonspace;
            let mut paragraph = self.blocks[container].kind == Kind::Paragraph;
            let setext_start = self.blocks[container].start;
            // A definition description, before any container opened on the line
            if first_iteration
                && self.settings.definition_list
                && !self.no_containers
                && self.indent <= 3
                && self.peek(at) == b':'
                && (at + 1 >= self.line_end || is_space_or_tab(self.peek(at + 1)))
            {
                match self.start_description(container, failed) {
                    Ok((parent, spread)) => {
                        container = self.open_description(parent, at, spread);
                        first_iteration = false;
                        maybe_lazy = false;
                        if self.blocks[container].accepts_lines() {
                            break;
                        }
                        continue;
                    }
                    // What was open ended: the line starts anew in its parent
                    Err(true) => {
                        container = self.tip;
                        last_matched = container;
                        paragraph = false;
                        maybe_lazy = false;
                    }
                    Err(false) => {}
                }
            }
            first_iteration = false;
            if self.mdx
                && let Some(done_here) = self.try_mdx_flow(&mut container, at, paragraph)
            {
                if done_here {
                    done = true;
                    break;
                }
                if self.error.is_some() {
                    return;
                }
            }
            // Frontmatter goes first, as micromark puts extensions first, at a line's start
            if (container == 0 || (paragraph && self.blocks[container].parent == 0))
                && self.offset == self.line_start
                && !self.settings.frontmatter.is_empty()
                && let Some((matter, value, end, next)) = self.scan_frontmatter()
            {
                let kind = Kind::Frontmatter { matter, value };
                container =
                    self.add_child(container, kind, Point::new(self.line_start, 0));
                self.touch(container, Point::new(end, 0));
                self.close_now(container);
                self.skip_to = next;
                done = true;
                break;
            }
            // Only definitions above an underline: they go, and the line starts anew
            if paragraph
                && !indented
                && scan_setext(self.bytes, at, self.line_end).is_some()
                && !self.extract_definitions(container)
            {
                container = self.blocks[container].parent as usize;
                last_matched = container;
                paragraph = false;
            }
            let gfm = self.settings.gfm.is_some();
            if paragraph
                && gfm
                && !indented
                && let Some(cells) = scan_delimiter_row(self.bytes, at, self.line_end)
                && !self.starts_container(at)
                && self.head_row_cells(container) == Some(cells)
            {
                container = self.start_table(container);
                let line = Line {
                    begin: self.line_start,
                    start: at,
                    virt: 0,
                    pad: 0,
                    end: self.line_end,
                    next: self.line_next,
                };
                self.push_line(container, line);
                self.touch(container, Point::new(self.line_end, 0));
                done = true;
                break;
            }
            if !indented
                && gfm
                && !self.no_containers
                && self.peek(at) == b'['
                && self.peek(at + 1) == b'^'
                && let Some(label_end) =
                    scan_footnote_label(self.bytes, at + 2, self.line_end)
                && self.peek(label_end + 1) == b':'
            {
                let start = Point::new(at, 0);
                self.advance_offset(label_end + 2 - self.offset, false);
                while is_space_or_tab(self.peek(self.offset)) {
                    self.advance_offset(1, true);
                }
                let raw = String::from_utf8_lossy(&self.bytes[at + 2..label_end]);
                self.footnotes.insert(normalize_identifier(&raw));
                container = self.add_child(
                    container,
                    Kind::Footnote {
                        label: (at + 2, label_end),
                    },
                    start,
                );
                self.touch(container, Point::new(self.line_end, 0));
            } else if !indented
                && self.settings.directive
                && self.peek(at) == b':'
                && self.peek(at + 1) == b':'
                && let Some((kind, head_end)) = self.scan_flow_directive(at)
            {
                let start = Point::new(at, 0);
                let is_leaf = matches!(kind, Kind::LeafDirective { .. });
                container = self.add_child(container, kind, start);
                self.touch(container, Point::new(self.line_end, 0));
                let _ = head_end;
                if is_leaf {
                    self.close_now(container);
                }
                done = true;
                break;
            } else if !indented && !self.no_containers && self.peek(at) == b'>' {
                let start = Point::new(at, 0);
                self.advance_offset(at + 1 - self.offset, false);
                if is_space_or_tab(self.peek(self.offset)) {
                    self.advance_offset(1, true);
                }
                container = self.add_child(container, Kind::BlockQuote, start);
                quote = Some((container, self.offset));
            } else if let Some((depth, after)) = (!indented)
                .then(|| scan_atx(self.bytes, at, self.line_end))
                .flatten()
            {
                container = self.add_child(
                    container,
                    Kind::Heading {
                        depth,
                        setext: false,
                    },
                    Point::new(at, 0),
                );
                let (content_start, content_end) =
                    trim_atx_content(self.bytes, after, self.line_end);
                self.push_line(
                    container,
                    Line {
                        begin: self.line_start,
                        start: content_start,
                        virt: 0,
                        pad: 0,
                        end: content_end,
                        next: content_end,
                    },
                );
                self.touch(container, Point::new(self.line_end, 0));
                self.close_now(container);
                done = true;
                break;
            } else if let Some((marker, size)) = (!indented)
                .then(|| {
                    let math = self.settings.math.is_some();
                    scan_fence_open(self.bytes, at, self.line_end, math)
                })
                .flatten()
            {
                let mut info_start = at + size;
                while info_start < self.line_end
                    && is_space_or_tab(self.bytes[info_start])
                {
                    info_start += 1;
                }
                let kind = Kind::Fenced {
                    marker,
                    size,
                    indent: self.indent,
                    info: (info_start, self.line_end),
                };
                container = self.add_child(container, kind, Point::new(at, 0));
                self.touch(container, Point::new(self.line_end, 0));
                done = true;
                break;
            } else if let Some(kind) = (!indented && !self.mdx)
                .then(|| scan_html_start(self.bytes, at, self.line_end, paragraph))
                .flatten()
            {
                let start = self.point();
                container = self.add_child(container, Kind::Html { kind }, start);
                break;
            } else if let Some(depth) = (!indented && paragraph)
                .then(|| scan_setext(self.bytes, at, self.line_end))
                .flatten()
                && self.extract_definitions(container)
            {
                self.blocks[container].kind = Kind::Heading {
                    depth,
                    setext: true,
                };
                self.blocks[container].start = setext_start;
                self.touch(container, Point::new(self.line_end, 0));
                self.close_now(container);
                done = true;
                break;
            } else if !indented
                && !(paragraph && !all_matched)
                && scan_thematic_break(self.bytes, at, self.line_end)
            {
                container =
                    self.add_child(container, Kind::ThematicBreak, Point::new(at, 0));
                self.touch(container, Point::new(self.line_end, 0));
                self.close_now(container);
                done = true;
                break;
            } else if let Some((ordered, marker, start, size)) = (!self.no_containers
                && (!indented
                    || matches!(self.blocks[container].kind, Kind::List { .. })))
            .then(|| scan_list_marker(self.bytes, at, self.line_end))
            .flatten()
            .filter(|_| self.indent < 4 || self.mdx)
            .filter(|(ordered, _, start, size)| {
                !interrupt
                    || (!is_blank_from(self.bytes, at + size, self.line_end)
                        && (!ordered || *start == 1))
            }) {
                let marker_point = Point::new(at, 0);
                let marker_offset = self.indent;
                self.advance_offset(at + size - self.offset, false);
                let (saved_offset, saved_column, saved_partial, saved_tab) =
                    (self.offset, self.column, self.partial, self.tab_column);
                while self.column - saved_column <= 5
                    && is_space_or_tab(self.peek(self.offset))
                {
                    self.advance_offset(1, true);
                }
                let spaces = self.column - saved_column;
                let padding = if !(1..5).contains(&spaces) || self.offset >= self.line_end
                {
                    self.offset = saved_offset;
                    self.column = saved_column;
                    self.partial = saved_partial;
                    self.tab_column = saved_tab;
                    if spaces > 0 {
                        self.advance_offset(1, true);
                    }
                    size + 1
                } else {
                    size + spaces
                };
                if is_blank_from(self.bytes, at + size, self.line_end) {
                    self.prefix_end = Some(Point::new(at + size, 0));
                }
                let same_list = match self.blocks[container].kind {
                    Kind::List {
                        ordered: list_ordered,
                        marker: list_marker,
                        ..
                    } => list_ordered == ordered && list_marker == marker,
                    _ => false,
                };
                if !same_list {
                    container = self.add_child(
                        container,
                        Kind::List {
                            ordered,
                            marker,
                            start,
                        },
                        marker_point,
                    );
                }
                if !is_blank_from(self.bytes, at + size, self.line_end) {
                    self.prefix_end = None;
                } else {
                    self.prefix_end = Some(Point::new(at + size, 0));
                }
                container = self.add_child(
                    container,
                    Kind::Item {
                        width: marker_offset + padding,
                    },
                    marker_point,
                );
                self.prefix_end = None;
                self.touch(container, Point::new(self.line_end, 0));
            } else if indented && !maybe_lazy && !self.blank {
                let start = self.point();
                self.advance_offset(4, true);
                container = self.add_child(container, Kind::Indented, start);
                break;
            } else {
                break;
            }
            if self.blocks[container].accepts_lines() {
                break;
            }
            maybe_lazy = false;
        }
        if done {
            return;
        }

        self.find_first_nonspace();
        let tip_is_paragraph = self.blocks[self.tip].kind == Kind::Paragraph;
        if self.tip != last_matched
            && container == last_matched
            && !self.blank
            && tip_is_paragraph
            && !self.in_unmatched_directive(last_matched)
        {
            let line = self.continuation_line();
            let tip = self.tip;
            self.lazy_lines.push(self.lines.len() as u32);
            self.push_line(tip, line);
            self.touch(tip, Point::new(self.line_end, 0));
            return;
        }
        // A blank last line leaves the quotes it does not continue open to the end
        if self.blank && quote.is_some() && self.line_next == self.bytes.len() {
            let mut block = self.tip;
            while block != container && block != 0 {
                if self.blocks[block].kind == Kind::BlockQuote {
                    self.blocks[block].end = Point::new(self.line_end, 0);
                }
                block = self.blocks[block].parent as usize;
            }
        }
        // New blocks closed what was left open; otherwise the unmatched ones close here
        self.close_unmatched(container);
        if self.blank
            && let Some((quote, prefix_end)) = quote
        {
            self.touch(quote, Point::new(self.line_end, 0));
            // A list in the quote ends after the quote's prefix, not before it as items do
            let mut block = container;
            while block != quote && block != 0 {
                if matches!(self.blocks[block].kind, Kind::List { .. })
                    && self.blocks[block].opened_line != self.line_start
                {
                    self.blocks[block].end = Point::new(prefix_end, 0);
                }
                block = self.blocks[block].parent as usize;
            }
        }
        let kind = self.blocks[container].kind;
        match kind {
            Kind::Indented => {
                if self.blank && !self.code_full {
                    self.blocks[container].trailing_blank += 1;
                    let line = self.current_line();
                    self.push_line(container, line);
                } else {
                    self.blocks[container].trailing_blank = 0;
                    let line = self.current_line();
                    self.push_line(container, line);
                    self.touch(container, Point::new(self.line_end, 0));
                }
            }
            Kind::Fenced { .. } => {
                let line = self.current_line();
                self.push_line(container, line);
                self.touch(container, Point::new(self.line_end, 0));
            }
            Kind::Html { kind } => {
                let line = self.current_line();
                self.push_line(container, line);
                self.touch(container, Point::new(self.line_end, 0));
                if kind <= 5
                    && scan_html_end(&self.bytes[self.offset..self.line_end], kind)
                {
                    self.blocks[container].ended = true;
                    self.close_leaf(container);
                }
            }
            _ if self.blank => {
                let block = &mut self.blocks[container];
                let fresh_item =
                    matches!(block.kind, Kind::Item { .. } | Kind::Footnote { .. })
                        && block.opened_line == self.line_start;
                if !fresh_item {
                    block.blank = true;
                    block.blank_run = block.blank_run.saturating_add(1);
                }
            }
            Kind::Paragraph => {
                let line = self.continuation_line();
                self.push_line(container, line);
                self.touch(container, Point::new(self.line_end, 0));
            }
            Kind::Table => {
                let line = self.paragraph_line();
                self.push_line(container, line);
                self.touch(container, Point::new(self.line_end, 0));
            }
            _ => {
                let start = Point::new(self.first_nonspace, 0);
                let prefix_end = self.offset;
                let paragraph = self.add_child(container, Kind::Paragraph, start);
                self.blocks[paragraph].prefix_end = prefix_end;
                let line = self.paragraph_line();
                self.push_line(paragraph, line);
                self.touch(paragraph, Point::new(self.line_end, 0));
            }
        }
    }

    // Where a description at this line's `:` opens, and its spread: after the description
    // that did not continue, or after a paragraph whose lines become its terms
    fn start_description(
        &mut self,
        container: usize,
        failed: usize,
    ) -> Result<(usize, bool), bool> {
        let plain = self.indent == 0 && self.offset == self.line_start;
        if failed != NONE as usize
            && matches!(self.blocks[failed].kind, Kind::DefDescription { .. })
        {
            let blank_last = self.blocks[failed].lines.0 == 1;
            let parent = self.blocks[failed].parent as usize;
            return Ok((parent, blank_last));
        }
        if self.blocks[container].kind == Kind::Paragraph {
            let parent = self.blocks[container].parent as usize;
            if !self.make_terms(container) {
                return Err(true);
            }
            return Ok((parent, false));
        }
        // A table row before it ends the table instead
        if self.blocks[container].kind == Kind::Table {
            let parent = self.blocks[container].parent as usize;
            self.finalize(container);
            self.tip = parent;
            return Err(true);
        }
        // The lazy lines of a paragraph in a block quote that did not continue
        if failed != NONE as usize
            && self.blocks[failed].kind == Kind::BlockQuote
            && self.indent == 0
            && self.blocks[self.tip].kind == Kind::Paragraph
            && self.blocks[self.tip].parent as usize == failed
        {
            let paragraph = self.tip;
            let (first, last) = self.blocks[paragraph].lines;
            let mut lazy = last;
            while lazy > first && self.lazy_lines.binary_search(&(lazy - 1)).is_ok() {
                lazy -= 1;
            }
            if lazy == last || lazy == first {
                self.close_unmatched(container);
                return Err(true);
            }
            let begin = self.lines[lazy as usize].begin;
            let block = &mut self.blocks[paragraph];
            block.lines.1 = lazy;
            block.end = Point::new(begin, 0);
            block.swallowed = true;
            self.blocks[failed].end = Point::new(begin, 0);
            self.close_unmatched(container);
            self.push_terms(container, lazy, last, Point::new(begin, 0));
            return Ok((container, false));
        }
        // A paragraph left open in containers that did not continue: it may not take the
        // line lazily, so they close and the line starts anew
        if failed != NONE as usize && self.blocks[self.tip].kind == Kind::Paragraph {
            self.close_unmatched(container);
            return Err(true);
        }
        // A paragraph and one blank line above
        let last = self.blocks[container].last;
        if last != NONE
            && self.blocks[last as usize].kind == Kind::Paragraph
            && self.blocks[container].blank_run == 1
            && self.tip == container
        {
            if !self.make_terms(last as usize) {
                return Err(false);
            }
            return Ok((container, plain));
        }
        Err(false)
    }

    // Turn a paragraph's lines into terms, one each, once its definitions are off
    fn make_terms(&mut self, paragraph: usize) -> bool {
        if self.blocks[paragraph].open {
            if self.tip == paragraph {
                let parent = self.blocks[paragraph].parent as usize;
                self.finalize(paragraph);
                self.tip = parent;
            } else {
                self.finalize(paragraph);
            }
        }
        if !matches!(self.blocks[paragraph].kind, Kind::Paragraph)
            || self.blocks[paragraph].lines.0 >= self.blocks[paragraph].lines.1
        {
            return false;
        }
        // Removed for definitions: the paragraph is no longer its parent's
        let parent = self.blocks[paragraph].parent as usize;
        if self.blocks[parent].last != paragraph as u32 {
            return false;
        }
        let (first, last) = self.blocks[paragraph].lines;
        let block = &mut self.blocks[paragraph];
        block.kind = Kind::DefTerm;
        block.lines = (first, first + 1);
        // More of the paragraph follows: trailing spaces make a hard break
        block.swallowed = last > first + 1;
        block.start = Point::new(block.prefix_end, 0);
        block.end = Point::new(self.lines[first as usize].next, 0);
        self.push_terms(parent, first + 1, last, Point::new(0, 0));
        self.version += 1;
        true
    }

    // A term per line from `first` to `last`, into `parent`; the first starts at `start`
    // when that is set, else each where its line's content does
    fn push_terms(&mut self, parent: usize, first: u32, last: u32, start: Point) {
        for line in first..last {
            let begin = if line == first && start.offset() > 0 {
                start
            } else {
                Point::new(self.lines[line as usize].start, 0)
            };
            let mut term =
                Block::new(Kind::DefTerm, parent as u32, begin, self.line_start);
            term.lines = (line, line + 1);
            term.swallowed = line + 1 < last;
            term.end = Point::new(self.lines[line as usize].next, 0);
            term.open = false;
            let index = self.blocks.len();
            self.blocks.push(term);
            self.link_child(parent, index);
        }
    }

    // Open a description at `at` in `parent`: its prefix is the marker and up to four
    // columns of whitespace; with more, one is the prefix's and the rest indented code
    fn open_description(&mut self, parent: usize, at: usize, spread: bool) -> usize {
        let marker_offset = self.indent;
        self.advance_offset(at + 1 - self.offset, false);
        let blank = is_blank_from(self.bytes, self.offset, self.line_end);
        let (saved_offset, saved_column, saved_partial, saved_tab) =
            (self.offset, self.column, self.partial, self.tab_column);
        while self.column - saved_column < 5 && is_space_or_tab(self.peek(self.offset)) {
            self.advance_offset(1, true);
        }
        let spaces = self.column - saved_column;
        let width = if blank {
            marker_offset + 2
        } else if spaces > 4 {
            self.offset = saved_offset;
            self.column = saved_column;
            self.partial = saved_partial;
            self.tab_column = saved_tab;
            self.advance_offset(1, true);
            marker_offset + 2
        } else {
            marker_offset + 1 + spaces
        };
        let start = if blank {
            Point::new(at + 1, 0)
        } else {
            self.point()
        };
        // A sibling description ends where this one's prefix does
        if let Some(previous) = self.open_description_of(parent) {
            self.blocks[previous].end = if blank {
                Point::new(at + 1, 0)
            } else {
                self.point()
            };
            self.blocks[previous].ended = true;
        }
        let kind = Kind::DefDescription {
            width,
            blank_initial: blank,
        };
        let description = self.add_child(parent, kind, start);
        self.blocks[description].spread = spread;
        // Whether its last line was blank, for a sibling's spread
        self.blocks[description].lines.0 = u32::from(blank);
        self.touch(description, Point::new(self.line_end, 0));
        description
    }

    fn open_description_of(&self, parent: usize) -> Option<usize> {
        let last = self.blocks[parent].last;
        (last != NONE
            && self.blocks[last as usize].open
            && matches!(self.blocks[last as usize].kind, Kind::DefDescription { .. }))
        .then_some(last as usize)
    }

    // ESM at a line's start, or tags and expressions alone on lines: `Some(true)` once one
    // took the line, `Some(false)` when a syntax error stopped parsing
    fn try_mdx_flow(
        &mut self,
        container: &mut usize,
        at: usize,
        paragraph: bool,
    ) -> Option<bool> {
        let settings = self.settings;
        let parser = settings.mdx.and_then(|mdx| mdx.0.as_ref());
        let source_text = self.source;
        let byte = self.peek(at);
        // ESM: `import` or `export` and a space, at the first column, not interrupting
        if (byte == b'i' || byte == b'e')
            && at == self.line_start
            && self.offset == self.line_start
            && self.blocks[self.tip].kind != Kind::Paragraph
            && (self.bytes[at..].starts_with(b"import ")
                || self.bytes[at..].starts_with(b"export "))
        {
            let end;
            let mut line_end = self.line_end;
            let mut parsed = 0usize;
            loop {
                let next = next_line(self.bytes, line_end);
                let at_eof = line_end >= self.bytes.len();
                // Is the next line blank, or the end?
                let mut probe = next;
                while probe < self.bytes.len() && is_space_or_tab(self.bytes[probe]) {
                    probe += 1;
                }
                let blank_next = at_eof
                    || probe >= self.bytes.len()
                    || matches!(self.bytes[probe], b'\n' | b'\r');
                if at_eof || blank_next {
                    let source = &source_text[at..line_end];
                    // Many tries at one block are given up on: each parses all of it
                    parsed += source.len();
                    if parsed > 16 * source.len() + (1 << 20) {
                        self.error = Some(textlane_core::Error::new(
                            "micromark-extension-mdxjs-esm:acorn",
                            "Could not parse import/exports with acorn",
                        ));
                        return Some(false);
                    }
                    match check_esm(parser, source, at_eof, &mut self.imports) {
                        Ok(true) => {
                            end = line_end;
                            break;
                        }
                        Ok(false) => {}
                        Err(error) => {
                            self.error = Some(error);
                            return Some(false);
                        }
                    }
                }
                if at_eof {
                    end = line_end;
                    break;
                }
                // The next line, blank or not, is the ESM's
                let mut stop = next;
                while stop < self.bytes.len()
                    && !matches!(self.bytes[stop], b'\n' | b'\r')
                {
                    stop += 1;
                }
                line_end = stop;
            }
            let block = self.add_child(
                *container,
                Kind::Esm { value: (at, end) },
                Point::new(at, 0),
            );
            self.touch(block, Point::new(end, 0));
            self.close_now(block);
            self.skip_to = next_line(self.bytes, end);
            *container = self.tip;
            return Some(true);
        }
        if byte != b'<' && byte != b'{' {
            return None;
        }
        let chain = self.chain(*container);
        let mut text = FlowText::new(self, at, chain);
        let scan = MdxScan {
            parser,
            allow_lazy: false,
        };
        let mut items = vec![];
        let mut index = 0;
        loop {
            match byte_of(&mut text, index) {
                Some(b'<') => match scan.scan_tag(&mut text, index) {
                    Ok(Some(tag)) => {
                        index = tag.end;
                        items.push(FlowItem::Tag(tag));
                    }
                    Ok(None) => return None,
                    Err(error) => {
                        self.error = Some(error);
                        return Some(false);
                    }
                },
                Some(b'{') => match scan.scan_expression(&mut text, index, false, true) {
                    Ok((end, value)) => {
                        items.push(FlowItem::Expression {
                            span: (index, end),
                            value,
                        });
                        index = end;
                    }
                    Err(error) => {
                        self.error = Some(error);
                        return Some(false);
                    }
                },
                _ => return None,
            }
            while matches!(byte_of(&mut text, index), Some(b' ' | b'\t')) {
                index += 1;
            }
            // After an expression only a tag may follow on the line
            let after_expression =
                matches!(items.last(), Some(FlowItem::Expression { .. }));
            match byte_of(&mut text, index) {
                None | Some(b'\n' | b'\r') => break,
                Some(b'<') => continue,
                Some(b'{') if !after_expression => continue,
                _ => return None,
            }
        }
        let _ = paragraph;
        // Spans of the text become source points
        let map = |at: usize| text.map(at);
        for item in &mut items {
            item.map(&map);
        }
        let (end, consumed_next) = text.end(index);
        let first = self.mdx_items.len() as u32;
        self.mdx_items.extend(items);
        let last = self.mdx_items.len() as u32;
        let block =
            self.add_child(*container, Kind::MdxFlow { first, last }, Point::new(at, 0));
        self.touch(block, Point::new(end, 0));
        self.close_now(block);
        self.skip_to = consumed_next;
        *container = self.tip;
        Some(true)
    }

    // The open blocks from the document down to `container`
    fn chain(&self, container: usize) -> Vec<Kind> {
        let mut chain = vec![];
        let mut block = container;
        while block != 0 {
            let kind = self.blocks[block].kind;
            let has_children = self.blocks[block].first != NONE;
            chain.push((kind, has_children));
            block = self.blocks[block].parent as usize;
        }
        chain.reverse();
        chain.into_iter().map(|(kind, _)| kind).collect()
    }

    // Whether a container directive lies between the tip and `last_matched`: its lines are
    // concrete, so a lazy line ends it
    fn in_unmatched_directive(&self, last_matched: usize) -> bool {
        let mut block = self.tip;
        while block != last_matched && block != 0 {
            if matches!(self.blocks[block].kind, Kind::ContainerDirective { .. }) {
                return true;
            }
            block = self.blocks[block].parent as usize;
        }
        false
    }

    // `::name` alone on its line, or `:::name` opening a container
    fn scan_flow_directive(&mut self, at: usize) -> Option<(Kind, usize)> {
        let bytes = self.bytes;
        let colons = scan_colon_fence(bytes, at, self.line_end).max({
            let mut index = at;
            while index < self.line_end && bytes[index] == b':' {
                index += 1;
            }
            index - at
        });
        let text = self.source;
        let head_at = at + colons;
        if colons == 2 || colons >= 3 {
            let head = scan_head(bytes, text, head_at, self.line_end, false, false)?;
            let mut end = head.end;
            while end < self.line_end && is_space_or_tab(bytes[end]) {
                end += 1;
            }
            if end != self.line_end {
                return None;
            }
            let index = self.directives.len() as u32;
            self.directives.push(head);
            let kind = if colons == 2 {
                Kind::LeafDirective { index }
            } else {
                Kind::ContainerDirective {
                    index,
                    size: colons,
                    prefix: self.indent,
                }
            };
            return Some((kind, end));
        }
        None
    }

    // A matter whose fences hold this line and later ones: its index, its value's bytes,
    // the end of its closing fence and the start of the line after it
    fn scan_frontmatter(&mut self) -> Option<(u32, (usize, usize), usize, usize)> {
        let bytes = self.bytes;
        let start = self.line_start;
        for (index, matter) in self.settings.frontmatter.iter().enumerate() {
            if (start != 0 && !matter.anywhere)
                || self.matter_failed.get(index).copied().unwrap_or(false)
                || !bytes[start..].starts_with(matter.open.as_bytes())
                || matter.open.is_empty()
            {
                continue;
            }
            let after = start + matter.open.len();
            let Some(open_end) = fence_line_end(bytes, after) else {
                continue;
            };
            if open_end >= bytes.len() {
                continue;
            }
            let mut line = next_line(bytes, open_end);
            let close = matter.close.as_bytes();
            let found = loop {
                if line >= bytes.len() {
                    break None;
                }
                if !close.is_empty()
                    && bytes[line..].starts_with(close)
                    && let Some(end) = fence_line_end(bytes, line + close.len())
                {
                    break Some((line, end));
                }
                let mut end = line;
                while end < bytes.len() && !matches!(bytes[end], b'\n' | b'\r') {
                    end += 1;
                }
                if end >= bytes.len() {
                    break None;
                }
                line = next_line(bytes, end);
            };
            match found {
                Some((close_start, close_end)) => {
                    let value = trim_matter(bytes, open_end, close_start);
                    return Some((
                        index as u32,
                        value,
                        close_end,
                        next_line(bytes, close_end),
                    ));
                }
                None => {
                    if self.matter_failed.len() <= index {
                        self.matter_failed.resize(index + 1, false);
                    }
                    self.matter_failed[index] = true;
                    self.no_containers = true;
                }
            }
        }
        None
    }

    // Whether a block quote or list item interrupting a paragraph starts at `at`: the
    // document takes containers before the flow sees the line
    fn starts_container(&self, at: usize) -> bool {
        if self.peek(at) == b'>' {
            return true;
        }
        scan_list_marker(self.bytes, at, self.line_end).is_some_and(
            |(ordered, _, start, size)| {
                !is_blank_from(self.bytes, at + size, self.line_end)
                    && (!ordered || start == 1)
            },
        )
    }

    // The cells of a paragraph's last line read as a table head row
    fn head_row_cells(&self, paragraph: usize) -> Option<usize> {
        let (_, last) = self.blocks[paragraph].lines;
        let line = self.lines[last as usize - 1];
        let mut start = line.start;
        while start < line.end && is_space_or_tab(self.bytes[start]) {
            start += 1;
        }
        if start - line.start > 3 && last - 1 > self.blocks[paragraph].lines.0 {
            return None;
        }
        scan_head_row(self.bytes, start, line.end)
    }

    // Split a paragraph's last line off as a table's head; the rest stays a paragraph
    fn start_table(&mut self, paragraph: usize) -> usize {
        let (first, last) = self.blocks[paragraph].lines;
        let line = self.lines[last as usize - 1];
        let mut start = line.start;
        while start < line.end && is_space_or_tab(self.bytes[start]) {
            start += 1;
        }
        if last - first == 1 {
            let block = &mut self.blocks[paragraph];
            block.kind = Kind::Table;
            block.start = Point::new(start, 0);
            self.version += 1;
            return paragraph;
        }
        let previous = self.lines[last as usize - 2];
        let block = &mut self.blocks[paragraph];
        block.lines.1 = last - 1;
        block.end = Point::new(previous.end, 0);
        let parent = block.parent as usize;
        let table = self.add_child(parent, Kind::Table, Point::new(start, 0));
        self.blocks[table].lines = (last - 1, last);
        table
    }

    fn current_line(&self) -> Line {
        let (start, virt, pad) = if self.partial {
            let width = 4 - (self.tab_column % 4);
            let virt = self.column - self.tab_column;
            (self.offset, virt as u8, (width - virt) as u8)
        } else {
            (self.offset, 0, 0)
        };
        Line {
            begin: self.line_start,
            start,
            virt,
            pad,
            end: self.line_end,
            next: self.line_next,
        }
    }

    // A later line of a paragraph keeps its indent: labels and code spans read it
    fn continuation_line(&self) -> Line {
        Line {
            begin: self.line_start,
            start: if self.partial {
                self.first_nonspace
            } else {
                self.offset
            },
            virt: 0,
            pad: 0,
            end: self.line_end,
            next: self.line_next,
        }
    }

    fn paragraph_line(&self) -> Line {
        Line {
            begin: self.line_start,
            start: self.first_nonspace,
            virt: 0,
            pad: 0,
            end: self.line_end,
            next: self.line_next,
        }
    }

    fn push_line(&mut self, block: usize, line: Line) {
        self.blocks[block].eol_end = line.next;
        let index = self.lines.len() as u32;
        self.lines.push(line);
        let range = &mut self.blocks[block].lines;
        if range.0 == range.1 {
            *range = (index, index + 1);
        } else {
            range.1 = index + 1;
        }
    }

    // Set the end of a block and every block around it
    // Ends only grow while blocks are open, so an ancestor already this far ends the walk
    fn touch(&mut self, mut block: usize, end: Point) {
        while block != 0 {
            let current = self.blocks[block].end;
            if current.offset() >= end.offset() && current != self.blocks[block].start {
                break;
            }
            self.blocks[block].end = end;
            block = self.blocks[block].parent as usize;
        }
    }

    fn add_child(&mut self, mut parent: usize, kind: Kind, start: Point) -> usize {
        if matches!(
            kind,
            Kind::BlockQuote
                | Kind::List { .. }
                | Kind::Item { .. }
                | Kind::Footnote { .. }
                | Kind::DefDescription { .. }
        ) {
            self.swallow = true;
            // Items before a sibling item end after its prefix; anything else the leaf ends
            let sibling = matches!(kind, Kind::Item { .. })
                && matches!(self.blocks[parent].kind, Kind::List { .. });
            let point = self.prefix_end.take().unwrap_or_else(|| self.point());
            self.swallow_end = sibling.then_some(point);
        }
        self.close_unmatched(parent);
        self.swallow = false;
        self.swallow_end = None;
        while !self.can_contain(parent, kind) {
            let up = self.blocks[parent].parent as usize;
            self.finalize(parent);
            parent = up;
            self.tip = up;
        }
        let parent_block = &mut self.blocks[parent];
        parent_block.blank_run = 0;
        if parent_block.blank {
            parent_block.blank = false;
            // Blank lines inside a container directive are the item around it's own
            if matches!(
                parent_block.kind,
                Kind::Item { .. } | Kind::List { .. } | Kind::ContainerDirective { .. }
            ) {
                parent_block.spread = true;
            }
        }
        let index = self.blocks.len();
        let mut block = Block::new(kind, parent as u32, start, self.line_start);
        block.eol_end = self.line_next;
        self.blocks.push(block);
        self.link_child(parent, index);
        self.tip = index;
        index
    }

    // A leaf complete on its line: closed at once, its parent the tip again
    fn close_now(&mut self, block: usize) {
        self.version += 1;
        self.blocks[block].open = false;
        self.tip = self.blocks[block].parent as usize;
    }

    fn link_child(&mut self, parent: usize, index: usize) {
        let last = self.blocks[parent].last;
        if last == NONE {
            self.blocks[parent].first = index as u32;
        } else {
            self.blocks[last as usize].next = index as u32;
        }
        self.blocks[index].prev = last;
        self.blocks[parent].last = index as u32;
        self.version += 1;
    }

    fn can_contain(&self, parent: usize, kind: Kind) -> bool {
        match self.blocks[parent].kind {
            Kind::Document
            | Kind::BlockQuote
            | Kind::Item { .. }
            | Kind::Footnote { .. }
            | Kind::ContainerDirective { .. }
            | Kind::DefDescription { .. } => !matches!(kind, Kind::Item { .. }),
            Kind::List { .. } => matches!(kind, Kind::Item { .. }),
            _ => false,
        }
    }

    fn close_unmatched(&mut self, last_matched: usize) {
        while self.tip != last_matched {
            let parent = self.blocks[self.tip].parent as usize;
            self.finalize(self.tip);
            self.tip = parent;
        }
        self.cascade_end = None;
    }

    fn close_fence(&mut self, block: usize) {
        self.blocks[block].ended = true;
        self.touch(block, Point::new(self.line_end, 0));
        self.close_leaf(block);
    }

    // Close the open leaf and every block inside it; the leaf is the tip
    fn close_leaf(&mut self, block: usize) {
        self.close_unmatched(block);
        let parent = self.blocks[block].parent as usize;
        self.finalize(block);
        self.tip = parent;
    }

    fn finalize(&mut self, index: usize) {
        if !self.blocks[index].open {
            return;
        }
        self.version += 1;
        self.blocks[index].open = false;
        let parent = self.blocks[index].parent as usize;
        match self.blocks[index].kind {
            Kind::Paragraph => {
                self.extract_definitions(index);
            }
            Kind::Indented => {
                let block = &mut self.blocks[index];
                let trailing = block.trailing_blank;
                block.lines.1 -= trailing;
                if trailing > 0 {
                    self.blocks[parent].blank = true;
                }
            }
            Kind::Item { .. } | Kind::List { .. } | Kind::BlockQuote => {
                // A footnote definition that took blank lines ends past the content around it
                let last = self.blocks[index].last;
                if last != NONE
                    && self.blocks[last as usize].end.offset()
                        > self.blocks[index].end.offset()
                {
                    self.blocks[index].end = self.blocks[last as usize].end;
                }
                if let Some((leaf_end, item_end)) = self.cascade_end {
                    let item = matches!(self.blocks[index].kind, Kind::Item { .. });
                    self.blocks[index].end = if item { item_end } else { leaf_end };
                }
                if self.blocks[index].blank
                    && !matches!(self.blocks[index].kind, Kind::BlockQuote)
                {
                    self.blocks[parent].blank = true;
                }
            }
            // A footnote definition closed after blank lines ends where the last of them
            // starts, after its container prefixes
            Kind::Footnote { .. } if self.cascade_end.is_some() => {
                if let Some((leaf_end, _)) = self.cascade_end {
                    self.blocks[index].end = leaf_end;
                }
            }
            Kind::Footnote { .. } if self.blocks[index].blank => {
                let end = self.blocks[index].eol_end;
                self.blocks[index].end = Point::new(end, 0);
                self.blocks[parent].blank = true;
            }
            // A description takes the blank lines after it: it ends where the line that
            // stops it starts
            Kind::DefDescription { .. } if !self.blocks[index].ended => {
                let end = if self.eof {
                    self.bytes.len()
                } else {
                    self.line_start
                };
                self.blocks[index].end = Point::new(end, 0);
            }
            // An unclosed container directive takes its last line ending as a fence does;
            // a lazy line after an empty last one ends it where that one's content starts
            Kind::ContainerDirective { .. } if !self.blocks[index].ended => {
                let swallow = if self.eof {
                    !self.in_quote(index)
                } else {
                    self.swallow
                };
                let block = &mut self.blocks[index];
                if swallow {
                    if block.eol_end > block.end.offset() {
                        block.end = Point::new(block.eol_end, 0);
                    }
                    block.spread |= block.blank;
                } else if block.lines.0 == 1 {
                    block.end = Point::new(block.trailing_blank as usize, 0);
                    // Only the last blank line trails
                    block.spread |= block.blank_run >= 2;
                }
                if block.spread && matches!(self.blocks[parent].kind, Kind::Item { .. }) {
                    self.blocks[parent].spread = true;
                }
            }
            Kind::ContainerDirective { .. } => {
                if self.blocks[index].spread
                    && matches!(self.blocks[parent].kind, Kind::Item { .. })
                {
                    self.blocks[parent].spread = true;
                }
            }
            Kind::Fenced { .. } | Kind::Html { kind: 1..=5 }
                if !self.blocks[index].ended =>
            {
                self.end_unclosed(index);
            }
            _ => {}
        }
    }

    fn in_quote(&self, mut block: usize) -> bool {
        while block != 0 {
            block = self.blocks[block].parent as usize;
            if self.blocks[block].kind == Kind::BlockQuote {
                return true;
            }
        }
        false
    }

    // An unclosed fence or HTML takes its last line ending unless the next line is lazy;
    // a lazy one leaves out its last line when that is empty
    fn end_unclosed(&mut self, index: usize) {
        let (first, last) = self.blocks[index].lines;
        // At the end, an empty line continues items with content but no block quote
        let swallow = if self.eof {
            !self.in_quote(index)
        } else {
            self.swallow
        };
        // A blank last line with no line ending, at the end, is no line of it
        // In a container, whose prefix takes its whitespace
        if self.eof && swallow && last > first && self.blocks[index].parent != 0 {
            let line = self.lines[last as usize - 1];
            if line.start >= line.end && line.next == line.end && line.begin < line.end {
                let block = &mut self.blocks[index];
                block.lines.1 -= 1;
                block.end = Point::new(line.begin, 0);
                block.swallowed = block.lines.1 > block.lines.0;
                return;
            }
        }
        let block = &mut self.blocks[index];
        if swallow {
            if block.eol_end > block.end.offset() {
                let end = Point::new(block.eol_end, 0);
                block.end = end;
                block.swallowed = true;
                self.cascade_end = Some((end, self.swallow_end.unwrap_or(end)));
            }
        } else if last > first {
            let line = self.lines[last as usize - 1];
            if line.start >= line.end {
                block.lines.1 -= 1;
                block.end = Point::new(line.begin, 0);
                block.swallowed = block.lines.1 > block.lines.0;
            }
        }
    }

    /// Take the definitions off the start of a paragraph, as blocks before it; whether
    /// anything is left of it.
    fn extract_definitions(&mut self, paragraph: usize) -> bool {
        let (first, last) = self.blocks[paragraph].lines;
        if self.bytes.get(self.lines[first as usize].start) != Some(&b'[') {
            return true;
        }
        let (text, segments) =
            join_lines(self.bytes, &self.lines[first as usize..last as usize]);
        let bytes = text.as_bytes();
        let mut at = 0;
        let mut consumed = first;
        let mut found = vec![];
        while let Some((def, end)) = parse_definition(bytes, at) {
            // A definition ends a line, so the rest starts at the next one
            let line = segments.partition_point(|segment| segment.0 <= end) - 1;
            let line_index = first as usize + line;
            let start = map_segment(&segments, at);
            found.push((def, start, Point::new(self.lines[line_index].end, 0)));
            consumed = line_index as u32 + 1;
            at = segments
                .get(line + 1)
                .map_or(bytes.len(), |segment| segment.0);
            while at < bytes.len() && is_space_or_tab(bytes[at]) {
                at += 1;
            }
            if consumed == last {
                break;
            }
        }
        if found.is_empty() {
            return true;
        }
        let parent = self.blocks[paragraph].parent as usize;
        // The paragraph is its parent's last child; relink the definitions before it
        let previous = self.blocks[paragraph].prev;
        self.version += 1;
        let mut inserted = vec![];
        for (def, start, end) in found {
            self.longest_identifier = self.longest_identifier.max(def.identifier.len());
            self.identifiers.insert(def.identifier.clone());
            let index = self.defs.len() as u32;
            self.defs.push(def);
            let mut block = Block::new(
                Kind::Definition { index },
                parent as u32,
                start,
                self.line_start,
            );
            block.end = end;
            block.open = false;
            inserted.push(self.blocks.len() as u32);
            self.blocks.push(block);
        }
        for pair in inserted.windows(2) {
            self.blocks[pair[0] as usize].next = pair[1];
            self.blocks[pair[1] as usize].prev = pair[0];
        }
        self.blocks[inserted[0] as usize].prev = previous;
        let (head, tail) = (inserted[0], inserted[inserted.len() - 1]);
        if previous == NONE {
            self.blocks[parent].first = head;
        } else {
            self.blocks[previous as usize].next = head;
        }
        if consumed == last {
            // Nothing is left: the last definition takes the paragraph's place
            self.blocks[tail as usize].next = NONE;
            self.blocks[parent].last = tail;
            self.blocks[paragraph].open = false;
            if self.tip == paragraph {
                self.tip = parent;
            }
            false
        } else {
            self.blocks[tail as usize].next = paragraph as u32;
            self.blocks[paragraph].prev = tail;
            let line = &mut self.lines[consumed as usize];
            let prefix_end = line.start;
            while line.start < line.end && is_space_or_tab(self.bytes[line.start]) {
                line.start += 1;
            }
            let start = line.start;
            let block = &mut self.blocks[paragraph];
            block.lines.0 = consumed;
            block.start = Point::new(start, 0);
            block.prefix_end = prefix_end;
            true
        }
    }
}

/// The lines of a leaf as one string with their line endings, borrowing a single line, and
/// where each line starts in it and in the source.
pub(crate) fn join_lines<'s>(
    bytes: &'s [u8],
    lines: &[Line],
) -> (std::borrow::Cow<'s, str>, Vec<(usize, usize)>) {
    let text =
        |from: usize, to: usize| std::str::from_utf8(&bytes[from..to]).unwrap_or("");
    if let [line] = lines {
        return (
            std::borrow::Cow::Borrowed(text(line.start, line.end)),
            vec![(0, line.start)],
        );
    }
    let mut value =
        String::with_capacity(lines.iter().map(|line| line.next - line.start).sum());
    let mut segments = Vec::with_capacity(lines.len());
    for (index, line) in lines.iter().enumerate() {
        segments.push((value.len(), line.start));
        let end = if index + 1 == lines.len() {
            line.end
        } else {
            line.next
        };
        value.push_str(text(line.start, end));
    }
    (std::borrow::Cow::Owned(value), segments)
}

fn map_segment(segments: &[(usize, usize)], at: usize) -> Point {
    let index = segments.partition_point(|segment| segment.0 <= at) - 1;
    let (virtual_start, source_start) = segments[index];
    Point::new(source_start + at - virtual_start, 0)
}

// A definition at `at`, through the end of its line: the definition and its end
fn parse_definition(bytes: &[u8], at: usize) -> Option<(Def, usize)> {
    let label_end = scan_label(bytes, at)?;
    if bytes.get(label_end) != Some(&b':') {
        return None;
    }
    let (destination_at, _) = skip_space_eol(bytes, label_end + 1);
    let (url_start, url_end, after_url) = scan_destination(bytes, destination_at)?;
    if after_url == destination_at {
        return None;
    }
    let raw_label = std::str::from_utf8(&bytes[at + 1..label_end - 1]).ok()?;
    let text =
        |from: usize, to: usize| std::str::from_utf8(&bytes[from..to]).unwrap_or("");
    let make = |title: Option<String>| Def {
        label: decode_string(raw_label),
        identifier: normalize_identifier(raw_label).to_lowercase(),
        url: decode_string(text(url_start, url_end)),
        title,
    };
    let (title_at, eol) = skip_space_eol(bytes, after_url);
    if title_at > after_url
        && let Some((title_start, title_end, after_title)) = scan_title(bytes, title_at)
    {
        let line_end = skip_line_space(bytes, after_title);
        if line_end.is_some() {
            let title = (title_end > title_start).then(|| {
                decode_string(&strip_line_prefixes(text(title_start, title_end)))
            });
            return Some((make(title), after_title));
        }
    }
    let _ = eol;
    skip_line_space(bytes, after_url)?;
    Some((make(None), after_url))
}

// Only spaces and tabs to the end of the line: where the line ends
fn skip_line_space(bytes: &[u8], at: usize) -> Option<usize> {
    let mut index = at;
    while index < bytes.len() && is_space_or_tab(bytes[index]) {
        index += 1;
    }
    match bytes.get(index) {
        None | Some(b'\n' | b'\r') => Some(index),
        _ => None,
    }
}

/// A value whose later lines lose their indent, as micromark's line prefixes in titles and
/// inline HTML are no part of them.
pub(crate) fn strip_line_prefixes(value: &str) -> std::borrow::Cow<'_, str> {
    if !value.contains(['\n', '\r']) {
        return std::borrow::Cow::Borrowed(value);
    }
    let mut result = String::with_capacity(value.len());
    let mut at_line_start = false;
    for character in value.chars() {
        match character {
            '\n' | '\r' => {
                at_line_start = true;
                result.push(character);
            }
            ' ' | '\t' if at_line_start => {}
            _ => {
                at_line_start = false;
                result.push(character);
            }
        }
    }
    std::borrow::Cow::Owned(result)
}

// Spaces and tabs to the end of a fence's line: where the line ends
fn fence_line_end(bytes: &[u8], at: usize) -> Option<usize> {
    let mut index = at;
    while index < bytes.len() && is_space_or_tab(bytes[index]) {
        index += 1;
    }
    match bytes.get(index) {
        None | Some(b'\n' | b'\r') => Some(index),
        _ => None,
    }
}

fn next_line(bytes: &[u8], end: usize) -> usize {
    match bytes.get(end) {
        Some(b'\r') if bytes.get(end + 1) == Some(&b'\n') => end + 2,
        Some(_) => end + 1,
        None => end,
    }
}

// mdast-util-frontmatter drops the line ending after the opening fence and the one before
// the closing fence
fn trim_matter(bytes: &[u8], start: usize, end: usize) -> (usize, usize) {
    let start = next_line(bytes, start).min(end);
    let mut stop = end;
    if stop > start && bytes[stop - 1] == b'\n' {
        stop -= 1;
        if stop > start && bytes[stop - 1] == b'\r' {
            stop -= 1;
        }
    } else if stop > start && bytes[stop - 1] == b'\r' {
        stop -= 1;
    }
    (start, stop)
}

/// A JSX tag or expression of a flow line, its spans already source offsets.
#[derive(Debug, Clone)]
pub(crate) enum FlowItem {
    Tag(ScannedTag),
    Expression { span: (usize, usize), value: String },
}

impl FlowItem {
    fn map(&mut self, map: &impl Fn(usize) -> usize) {
        match self {
            FlowItem::Tag(tag) => {
                tag.start = map(tag.start);
                tag.end = map(tag.end);
                for attribute in &mut tag.attributes {
                    match attribute {
                        ScannedAttribute::Plain {
                            span, name, local, ..
                        } => {
                            *span = (map(span.0), map(span.1));
                            *name = (map(name.0), map(name.1));
                            if let Some(local) = local {
                                *local = (map(local.0), map(local.1));
                            }
                        }
                        ScannedAttribute::Expression { span, .. } => {
                            *span = (map(span.0), map(span.1))
                        }
                    }
                }
            }
            FlowItem::Expression { span, .. } => *span = (map(span.0), map(span.1)),
        }
    }
}

fn byte_of(text: &mut FlowText, at: usize) -> Option<u8> {
    while at >= text.text.len() {
        if !text.extend() {
            return None;
        }
    }
    Some(text.text[at])
}

/// The lines of a flow construct from its start on, each after the prefixes of the blocks
/// it is in; a line those do not continue is lazy.
pub(crate) struct FlowText<'s> {
    bytes: &'s [u8],
    chain: Vec<Kind>,
    text: Vec<u8>,
    // Where each line starts in the text and in the source
    segments: Vec<(usize, usize)>,
    lazy: Vec<usize>,
    // Where the next line starts in the source
    next: usize,
}

impl<'s> FlowText<'s> {
    fn new(blocks: &Blocks<'s>, at: usize, chain: Vec<Kind>) -> Self {
        let bytes = blocks.bytes;
        let end = blocks.line_end;
        FlowText {
            bytes,
            chain,
            text: bytes[at..end].to_vec(),
            segments: vec![(0, at)],
            lazy: vec![],
            next: end,
        }
    }

    // A text offset as a source offset; an end at a line's start is the line before's end
    fn map(&self, at: usize) -> usize {
        let index = self
            .segments
            .partition_point(|segment| segment.0 <= at)
            .max(1)
            - 1;
        let (virtual_start, source_start) = self.segments[index];
        source_start + at - virtual_start
    }

    // Where the construct ends in the source, and where the line after its last starts
    fn end(&self, at: usize) -> (usize, usize) {
        let mut stop = self.map(at);
        while stop < self.bytes.len() && !matches!(self.bytes[stop], b'\n' | b'\r') {
            stop += 1;
        }
        let _ = stop;
        let end = self.map(at);
        let mut line_end = end;
        while line_end < self.bytes.len()
            && !matches!(self.bytes[line_end], b'\n' | b'\r')
        {
            line_end += 1;
        }
        (line_end, next_line(self.bytes, line_end))
    }

    // Each prefix of the chain on the line at `start`: where the content starts, or nothing
    fn match_prefixes(&self, start: usize, end: usize) -> Option<usize> {
        let bytes = self.bytes;
        let mut index = start;
        let mut column = 0;
        let blank = is_blank_from(bytes, start, end);
        let skip_columns = |index: &mut usize, column: &mut usize, mut count: usize| {
            while count > 0 && *index < end && is_space_or_tab(bytes[*index]) {
                let width = if bytes[*index] == b'\t' {
                    4 - *column % 4
                } else {
                    1
                };
                *column += width;
                *index += 1;
                count = count.saturating_sub(width);
            }
        };
        let indent = |index: usize, column: usize| {
            let mut at = index;
            let mut columns = column;
            while at < end && is_space_or_tab(bytes[at]) {
                columns += if bytes[at] == b'\t' {
                    4 - columns % 4
                } else {
                    1
                };
                at += 1;
            }
            (at, columns - column)
        };
        for kind in &self.chain {
            match *kind {
                Kind::BlockQuote => {
                    let (at, _) = indent(index, column);
                    if bytes.get(at) != Some(&b'>') || at >= end {
                        return None;
                    }
                    column += at - index + 1;
                    index = at + 1;
                    if index < end && is_space_or_tab(bytes[index]) {
                        skip_columns(&mut index, &mut column, 1);
                    }
                }
                Kind::Item { width } | Kind::DefDescription { width, .. } => {
                    if blank {
                        return Some(end);
                    }
                    let (_, size) = indent(index, column);
                    if size < width {
                        return None;
                    }
                    skip_columns(&mut index, &mut column, width);
                }
                Kind::Footnote { .. } => {
                    if blank {
                        return Some(end);
                    }
                    let (_, size) = indent(index, column);
                    if size < 4 {
                        return None;
                    }
                    skip_columns(&mut index, &mut column, 4);
                }
                Kind::ContainerDirective { prefix, .. } => {
                    skip_columns(&mut index, &mut column, prefix)
                }
                _ => {}
            }
        }
        Some(index)
    }
}

impl MdxText for FlowText<'_> {
    fn bytes(&self) -> &[u8] {
        &self.text
    }

    fn extend(&mut self) -> bool {
        let eol = self.next;
        if eol >= self.bytes.len() {
            return false;
        }
        let start = next_line(self.bytes, eol);
        self.text.extend_from_slice(&self.bytes[eol..start]);
        let mut end = start;
        while end < self.bytes.len() && !matches!(self.bytes[end], b'\n' | b'\r') {
            end += 1;
        }
        let content = match self.match_prefixes(start, end) {
            Some(content) => content,
            None => {
                // A line that opens a container ends the flow, as the end of the input
                let mut first = start;
                while first < end && is_space_or_tab(self.bytes[first]) {
                    first += 1;
                }
                if first < end
                    && (self.bytes[first] == b'>'
                        || scan_list_marker(self.bytes, first, end).is_some())
                {
                    self.text.truncate(self.text.len() - (start - eol));
                    return false;
                }
                self.lazy.push(self.text.len());
                start
            }
        };
        self.segments.push((self.text.len(), content));
        self.text.extend_from_slice(&self.bytes[content..end]);
        self.next = end;
        true
    }

    fn lazy(&self, at: usize) -> bool {
        self.lazy.binary_search(&at).is_ok()
    }
}
