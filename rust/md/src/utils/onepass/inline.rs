use std::collections::HashSet;

use bumpalo::collections::String as ArenaString;
use textlane_core::mdast::{
    Break, Delete, Emphasis, FootnoteReference, Html, Image, ImageReference, InlineCode,
    InlineMath, Link, LinkReference, MdxExpression, MdxJsxElement, Node, ReferenceType,
    Strong, Text, TextDirective,
};
use textlane_core::{Allocator, ArenaVec, Error, Point, Position};

use crate::helpers::character::{
    CharacterKind, classify_character_unit, find_character_after, find_character_before,
};
use crate::helpers::decode::decode_string;
use crate::helpers::identifier::normalize_identifier;
use crate::helpers::literal::is_atext;
use crate::helpers::math::create_inline_math_data;
use crate::models::options::{Gfm, Math};
use crate::utils::onepass::block::strip_line_prefixes;
use crate::utils::onepass::directive::scan_head;
use crate::utils::onepass::literal::{LiteralKind, LiteralMemo, scan_literal};
use crate::utils::onepass::mdx::{
    Fixed, MdxBuild, MdxScan, ScannedTag, build_attributes, check_tag, empty_data,
    mismatch_error, stringify_point, tail_error, unclosed_error,
};
use crate::utils::onepass::scan::{
    HtmlMemo, is_line_ending, is_space_or_tab, scan_autolink, scan_destination,
    scan_entity, scan_html_text, scan_label, scan_title, skip_space_eol,
};

enum Item<'a> {
    /// Text from `start` to `end`; `value` when it differs from the source there.
    Text {
        start: usize,
        end: usize,
        value: Option<&'a str>,
    },
    Delim(Delim),
    Jsx(ScannedTag),
    Open {
        image: bool,
        start: usize,
        matched: bool,
    },
    Close {
        start: usize,
        end: usize,
        media: Option<Media<'a>>,
    },
    Node(Node<'a>),
}

struct Delim {
    marker: u8,
    start: usize,
    size: usize,
    left: usize,
    open: bool,
    close: bool,
    closes: Vec<u8>,
    opens: Vec<u8>,
}

enum Media<'a> {
    Resource {
        url: &'a str,
        title: Option<&'a str>,
    },
    Reference {
        kind: ReferenceType,
        label: &'a str,
        identifier: &'a str,
    },
}

struct Bracket {
    item: usize,
    image: bool,
    active: bool,
}

/// Phrasing content of one leaf: `text` is its lines joined, `segments` where each line
/// starts in it and in the source.
pub(crate) struct Inline<'a, 'd> {
    alloc: &'a Allocator,
    text: &'a str,
    bytes: &'a [u8],
    segments: &'d [(usize, usize)],
    identifiers: &'d HashSet<String>,
    longest_identifier: usize,
    gfm: Option<&'d Gfm>,
    footnotes: &'d HashSet<String>,
    in_table: bool,
    literal_memo: LiteralMemo,
    html_memo: HtmlMemo,
    code_memo: Vec<((u8, usize), usize)>,
    mdx: Option<&'d MdxBuild<'d>>,
    leaf: &'static str,
    directive: bool,
    keep_trailing: bool,
    math: Option<&'d Math>,
    items: Vec<Item<'a>>,
    brackets: Vec<Bracket>,
    delims: Vec<usize>,
}

/// What the inline pass of a leaf knows from the block pass.
pub(crate) struct InlineContext<'d> {
    pub(crate) identifiers: &'d HashSet<String>,
    pub(crate) longest_identifier: usize,
    pub(crate) gfm: Option<&'d Gfm>,
    pub(crate) math: Option<&'d Math>,
    /// GFM footnote labels, normalized as micromark compares them.
    pub(crate) footnotes: &'d HashSet<String>,
    /// Whether a task list check may open the text: a list item's first content.
    pub(crate) task: bool,
    pub(crate) in_table: bool,
    pub(crate) directive: bool,
    /// Whitespace at the end stays text, as in a directive's label.
    pub(crate) keep_trailing: bool,
    /// MDX's state, when it is on.
    pub(crate) mdx: Option<&'d MdxBuild<'d>>,
    /// The token type of the leaf this text is in, which MDX's errors name.
    pub(crate) leaf: &'static str,
}

/// Phrasing content, and the state of a task list check when one opened it.
pub(crate) struct InlineOutput<'a> {
    pub(crate) children: ArenaVec<'a, Node<'a>>,
    pub(crate) checked: Option<bool>,
}

// Every byte `scan` has an arm for; the rest are plain text it can skip in one run
const INLINE_SPECIAL: [bool; 256] = {
    let mut table = [false; 256];
    let bytes = b"\\&`$<{[:~!]*_\n\r";
    let mut index = 0;
    while index < bytes.len() {
        table[bytes[index] as usize] = true;
        index += 1;
    }
    table
};

pub(crate) fn parse_inline<'a>(
    alloc: &'a Allocator,
    text: &'a str,
    segments: &[(usize, usize)],
    context: &InlineContext,
) -> InlineOutput<'a> {
    let mut inline = Inline {
        alloc,
        text,
        bytes: text.as_bytes(),
        segments,
        identifiers: context.identifiers,
        longest_identifier: context.longest_identifier,
        gfm: context.gfm,
        footnotes: context.footnotes,
        in_table: context.in_table,
        literal_memo: LiteralMemo::default(),
        html_memo: HtmlMemo::default(),
        code_memo: Vec::new(),
        directive: context.directive,
        keep_trailing: context.keep_trailing,
        math: context.math,
        mdx: context.mdx,
        leaf: context.leaf,
        items: Vec::new(),
        brackets: Vec::new(),
        delims: Vec::new(),
    };
    let checked = if context.task {
        inline.scan_task_check()
    } else {
        None
    };
    inline.scan(if checked.is_some() { 3 } else { 0 });
    let all = std::mem::take(&mut inline.delims);
    inline.resolve_emphasis(&all);
    let mut children = inline.build();
    if checked.is_some() {
        strip_task_space(&mut children);
    }
    InlineOutput { children, checked }
}

impl<'a> Inline<'a, '_> {
    // `[ ]` or `[x]` and whitespace before more content: a task list check
    fn scan_task_check(&self) -> Option<bool> {
        let bytes = self.bytes;
        if bytes.first() != Some(&b'[') || bytes.get(2) != Some(&b']') {
            return None;
        }
        let checked = match bytes.get(1)? {
            b'\t' | b'\n' | b'\r' | b' ' => false,
            b'X' | b'x' => true,
            _ => return None,
        };
        match bytes.get(3)? {
            b'\n' | b'\r' => Some(checked),
            b'\t' | b' ' => {
                let mut index = 3;
                while bytes.get(index).is_some_and(|byte| is_space_or_tab(*byte)) {
                    index += 1;
                }
                (index < bytes.len()).then_some(checked)
            }
            _ => None,
        }
    }

    fn scan(&mut self, from: usize) {
        let bytes = self.bytes;
        let len = bytes.len();
        let mut pos = from;
        let mut text_start = from;
        while pos < len {
            let byte = bytes[pos];
            if self.gfm.is_some() && self.brackets.is_empty() && is_atext(byte) {
                let previous = match pos.checked_sub(1).map(|before| bytes[before]) {
                    Some(b'\r') => Some(b'\n'),
                    previous => previous,
                };
                if let Some((end, kind)) =
                    scan_literal(bytes, pos, previous, &mut self.literal_memo)
                {
                    self.push_text(text_start, pos);
                    self.push_literal(pos, end, kind);
                    pos = end;
                    text_start = pos;
                    continue;
                }
            }
            match byte {
                b'\\' => match bytes.get(pos + 1) {
                    Some(&next) if next.is_ascii_punctuation() => {
                        self.push_text(text_start, pos);
                        let value = &self.text[pos + 1..pos + 2];
                        self.items.push(Item::Text {
                            start: pos,
                            end: pos + 2,
                            value: Some(value),
                        });
                        pos += 2;
                        text_start = pos;
                    }
                    Some(&next) if is_line_ending(next) => {
                        self.push_text(text_start, pos);
                        let end = after_line_ending(bytes, pos + 1);
                        self.items.push(Item::Node(self.make_break(pos, end)));
                        pos = end;
                        while pos < len && is_space_or_tab(bytes[pos]) {
                            pos += 1;
                        }
                        text_start = pos;
                    }
                    _ => pos += 1,
                },
                b'&' => match scan_entity(bytes, pos) {
                    Some(end) => {
                        self.push_text(text_start, pos);
                        let value =
                            self.alloc.alloc_str(&decode_string(&self.text[pos..end]));
                        self.items.push(Item::Text {
                            start: pos,
                            end,
                            value: Some(value),
                        });
                        pos = end;
                        text_start = pos;
                    }
                    None => pos += 1,
                },
                b'`' => {
                    let mut run_end = pos;
                    while run_end < len && bytes[run_end] == b'`' {
                        run_end += 1;
                    }
                    match self.find_run_close(b'`', run_end, run_end - pos) {
                        Some(close) => {
                            self.push_text(text_start, pos);
                            let mut value = clean(
                                self.alloc,
                                strip_padding(&self.text[run_end..close]),
                            );
                            if self.in_table && value.contains("\\|") {
                                value = self.alloc.alloc_str(&unescape_pipes(value));
                            }
                            let end = close + (run_end - pos);
                            self.items.push(Item::Node(Node::InlineCode(InlineCode {
                                value,
                                position: Some(self.span(pos, end)),
                                data: None,
                            })));
                            pos = end;
                            text_start = pos;
                        }
                        None => pos = run_end,
                    }
                }
                b'$' if self.math.is_some() => {
                    let mut run_end = pos;
                    while run_end < len && bytes[run_end] == b'$' {
                        run_end += 1;
                    }
                    let size = run_end - pos;
                    let single =
                        self.math.is_some_and(|math| math.single_dollar_text_math);
                    let close = if size < 2 && !single {
                        None
                    } else {
                        self.find_run_close(b'$', run_end, size)
                    };
                    match close {
                        Some(close) => {
                            self.push_text(text_start, pos);
                            let value = clean(
                                self.alloc,
                                strip_padding(&self.text[run_end..close]),
                            );
                            let end = close + size;
                            self.items.push(Item::Node(Node::InlineMath(InlineMath {
                                value,
                                position: Some(self.span(pos, end)),
                                data: Some(create_inline_math_data(self.alloc, value)),
                            })));
                            pos = end;
                            text_start = pos;
                        }
                        None => pos = run_end,
                    }
                }
                b'<' if self.mdx.is_some() => {
                    self.push_text(text_start, pos);
                    text_start = pos;
                    let scan = MdxScan {
                        parser: self.mdx.and_then(|mdx| mdx.parser),
                        allow_lazy: true,
                    };
                    match scan.scan_tag(&mut Fixed(bytes), pos) {
                        Ok(Some(tag)) => {
                            pos = tag.end;
                            text_start = pos;
                            self.items.push(Item::Jsx(tag));
                        }
                        Ok(None) => pos += 1,
                        Err(error) => {
                            if let Some(mdx) = self.mdx {
                                mdx.errors.borrow_mut().crash(error);
                            }
                            return;
                        }
                    }
                }
                b'{' if self.mdx.is_some() => {
                    self.push_text(text_start, pos);
                    let scan = MdxScan {
                        parser: self.mdx.and_then(|mdx| mdx.parser),
                        allow_lazy: true,
                    };
                    match scan.scan_expression(&mut Fixed(bytes), pos, false, true) {
                        Ok((end, value)) => {
                            self.items.push(Item::Node(Node::MdxTextExpression(
                                MdxExpression {
                                    value: self.alloc.alloc_str(&value),
                                    position: Some(self.span(pos, end)),
                                    data: Some(empty_data(self.alloc)),
                                },
                            )));
                            pos = end;
                            text_start = pos;
                        }
                        Err(error) => {
                            if let Some(mdx) = self.mdx {
                                mdx.errors.borrow_mut().crash(error);
                            }
                            return;
                        }
                    }
                }
                b'<' => {
                    if let Some((end, email)) = scan_autolink(bytes, pos) {
                        self.push_text(text_start, pos);
                        let raw = &self.text[pos + 1..end - 1];
                        let url = if email {
                            self.alloc.alloc_str(&format!("mailto:{raw}"))
                        } else {
                            raw
                        };
                        let mut children = ArenaVec::with_capacity_in(1, self.alloc);
                        children.push(Node::Text(Text {
                            value: raw,
                            position: Some(self.span(pos + 1, end - 1)),
                            data: None,
                        }));
                        self.items.push(Item::Node(Node::Link(Link {
                            url,
                            title: None,
                            children,
                            position: Some(self.span(pos, end)),
                            data: None,
                        })));
                        pos = end;
                        text_start = pos;
                    } else if let Some(end) =
                        scan_html_text(bytes, pos, &mut self.html_memo)
                    {
                        self.push_text(text_start, pos);
                        self.items.push(Item::Node(Node::Html(Html {
                            value: match strip_line_prefixes(&self.text[pos..end]) {
                                std::borrow::Cow::Borrowed(raw) => clean(self.alloc, raw),
                                std::borrow::Cow::Owned(stripped) => {
                                    clean(self.alloc, self.alloc.alloc_str(&stripped))
                                }
                            },
                            position: Some(self.span(pos, end)),
                            data: None,
                        })));
                        pos = end;
                        text_start = pos;
                    } else {
                        pos += 1;
                    }
                }
                b'[' => {
                    self.push_text(text_start, pos);
                    if let Some(end) = self.scan_footnote_call(pos) {
                        pos = end;
                        text_start = pos;
                        continue;
                    }
                    self.open_bracket(pos, false);
                    pos += 1;
                    text_start = pos;
                }
                b':' if self.directive => {
                    self.push_text(text_start, pos);
                    text_start = pos;
                    match self.scan_text_directive(pos) {
                        Some(end) => {
                            pos = end;
                            text_start = pos;
                        }
                        None => pos += 1,
                    }
                }
                b'~' if self.gfm.is_some() => {
                    self.push_text(text_start, pos);
                    let mut end = pos;
                    while end < len && bytes[end] == b'~' {
                        end += 1;
                    }
                    let size = end - pos;
                    let single = self.gfm.is_none_or(|gfm| gfm.single_tilde);
                    if size > 2 || (size == 1 && !single) {
                        self.items.push(Item::Text {
                            start: pos,
                            end,
                            value: None,
                        });
                    } else {
                        let before =
                            classify_character_unit(find_character_before(bytes, pos));
                        let after =
                            classify_character_unit(find_character_after(bytes, end));
                        self.delims.push(self.items.len());
                        self.items.push(Item::Delim(Delim {
                            marker: b'~',
                            start: pos,
                            size,
                            left: size,
                            open: after == CharacterKind::Other
                                || (after == CharacterKind::Punctuation
                                    && before != CharacterKind::Other),
                            close: before == CharacterKind::Other
                                || (before == CharacterKind::Punctuation
                                    && after != CharacterKind::Other),
                            closes: Vec::new(),
                            opens: Vec::new(),
                        }));
                    }
                    pos = end;
                    text_start = pos;
                }
                b'!' if bytes.get(pos + 1) == Some(&b'[') => {
                    self.push_text(text_start, pos);
                    self.open_bracket(pos, true);
                    pos += 2;
                    text_start = pos;
                }
                b']' => {
                    self.push_text(text_start, pos);
                    pos = self.close_bracket(pos);
                    text_start = pos;
                }
                marker @ (b'*' | b'_') => {
                    self.push_text(text_start, pos);
                    let mut end = pos;
                    while end < len && bytes[end] == marker {
                        end += 1;
                    }
                    let before_char = find_character_before(bytes, pos);
                    let after_char = find_character_after(bytes, end);
                    let before = classify_character_unit(before_char);
                    let after = classify_character_unit(after_char);
                    // micromark lets a sequence open before and close after GFM's `~`
                    let gfm = self.gfm.is_some();
                    let open = after == CharacterKind::Other
                        || (after == CharacterKind::Punctuation
                            && before != CharacterKind::Other)
                        || (gfm && after_char == Some('~'));
                    let close = before == CharacterKind::Other
                        || (before == CharacterKind::Punctuation
                            && after != CharacterKind::Other)
                        || (gfm && before_char == Some('~'));
                    let (open, close) = if marker == b'_' {
                        (
                            open && (before != CharacterKind::Other || !close),
                            close && (after != CharacterKind::Other || !open),
                        )
                    } else {
                        (open, close)
                    };
                    self.delims.push(self.items.len());
                    self.items.push(Item::Delim(Delim {
                        marker,
                        start: pos,
                        size: end - pos,
                        left: end - pos,
                        open,
                        close,
                        closes: Vec::new(),
                        opens: Vec::new(),
                    }));
                    pos = end;
                    text_start = pos;
                }
                b'\n' | b'\r' => {
                    let mut suffix = pos;
                    let mut tabs = false;
                    while suffix > text_start && is_space_or_tab(bytes[suffix - 1]) {
                        tabs |= bytes[suffix - 1] == b'\t';
                        suffix -= 1;
                    }
                    self.push_text(text_start, suffix);
                    let end = after_line_ending(bytes, pos);
                    if !tabs && pos - suffix >= 2 {
                        self.items.push(Item::Node(self.make_break(suffix, end)));
                    } else {
                        self.items.push(Item::Text {
                            start: pos,
                            end,
                            value: None,
                        });
                    }
                    // The next line's indent is no text
                    pos = end;
                    while pos < len && is_space_or_tab(bytes[pos]) {
                        pos += 1;
                    }
                    text_start = pos;
                }
                _ if self.gfm.is_none() => {
                    pos += 1;
                    while pos < len && !INLINE_SPECIAL[bytes[pos] as usize] {
                        pos += 1;
                    }
                }
                _ => pos += 1,
            }
        }
        let mut suffix = len;
        while !self.keep_trailing
            && suffix > text_start
            && is_space_or_tab(bytes[suffix - 1])
        {
            suffix -= 1;
        }
        self.push_text(text_start, suffix);
    }

    // `:name[label]{attributes}`: a text directive, not right after another colon unless
    // that one is escaped
    fn scan_text_directive(&mut self, pos: usize) -> Option<usize> {
        let bytes = self.bytes;
        if pos > 0 && bytes[pos - 1] == b':' {
            let mut slashes = 0;
            while pos >= 2 + slashes && bytes[pos - 2 - slashes] == b'\\' {
                slashes += 1;
            }
            if slashes % 2 == 0 {
                return None;
            }
        }
        let head = scan_head(bytes, self.text, pos + 1, bytes.len(), true, true)?;
        let children = match head.label {
            Some(((start, end), _)) if end > start => self.parse_nested(start, end),
            _ => ArenaVec::new_in(self.alloc),
        };
        let mut attributes =
            ArenaVec::with_capacity_in(head.attributes.len(), self.alloc);
        for (key, value) in &head.attributes {
            attributes.push((
                &*self.alloc.alloc_str(key),
                clean(self.alloc, self.alloc.alloc_str(value)),
            ));
        }
        self.items
            .push(Item::Node(Node::TextDirective(TextDirective {
                name: &self.text[head.name.0..head.name.1],
                attributes,
                children,
                position: Some(self.span(pos, head.end)),
                data: None,
            })));
        Some(head.end)
    }

    // Phrasing inside `start..end` of this text, as its own inline pass: a label's content
    fn parse_nested(&self, start: usize, end: usize) -> ArenaVec<'a, Node<'a>> {
        let segments = shift_segments(self.segments, start, end);
        let context = InlineContext {
            identifiers: self.identifiers,
            longest_identifier: self.longest_identifier,
            gfm: self.gfm,
            math: self.math,
            footnotes: self.footnotes,
            task: false,
            in_table: self.in_table,
            directive: self.directive,
            keep_trailing: true,
            mdx: self.mdx,
            leaf: self.leaf,
        };
        parse_inline(self.alloc, &self.text[start..end], &segments, &context).children
    }

    fn push_text(&mut self, start: usize, end: usize) {
        if end > start {
            self.items.push(Item::Text {
                start,
                end,
                value: None,
            });
        }
    }

    fn make_break(&self, start: usize, end: usize) -> Node<'a> {
        Node::Break(Break {
            position: Some(self.span(start, end)),
            data: None,
        })
    }

    fn push_literal(&mut self, start: usize, end: usize, kind: LiteralKind) {
        let raw = clean(self.alloc, &self.text[start..end]);
        let url = match kind {
            LiteralKind::Email => self.alloc.alloc_str(&format!("mailto:{raw}")),
            LiteralKind::Www => self.alloc.alloc_str(&format!("http://{raw}")),
            LiteralKind::Http => raw,
        };
        let mut children = ArenaVec::with_capacity_in(1, self.alloc);
        children.push(Node::Text(Text {
            value: raw,
            position: Some(self.span(start, end)),
            data: None,
        }));
        self.items.push(Item::Node(Node::Link(Link {
            url,
            title: None,
            children,
            position: Some(self.span(start, end)),
            data: None,
        })));
    }

    // The label of a footnote call after `[^`: its end, when it names a definition
    fn footnote_label_end(&self, from: usize) -> Option<usize> {
        let bytes = self.bytes;
        let mut index = from;
        let mut size = 0;
        loop {
            match bytes.get(index) {
                None | Some(b'\t' | b'\n' | b'\r' | b' ' | b'[') => return None,
                _ if size > 999 => return None,
                Some(b']') if size == 0 => return None,
                Some(b']') => break,
                Some(b'\\')
                    if matches!(bytes.get(index + 1), Some(b'[' | b'\\' | b']')) =>
                {
                    index += 2;
                    size += 2;
                }
                Some(_) => {
                    index += 1;
                    size += 1;
                }
            }
        }
        let raw = &self.text[from..index];
        self.footnotes
            .contains(&normalize_identifier(raw))
            .then_some(index)
    }

    // `[^a]`: a call to a GFM footnote, tried before a link
    fn scan_footnote_call(&mut self, pos: usize) -> Option<usize> {
        if self.gfm.is_none()
            || self.footnotes.is_empty()
            || self.bytes.get(pos + 1) != Some(&b'^')
        {
            return None;
        }
        let end = self.footnote_label_end(pos + 2)?;
        self.push_footnote_call(pos, pos + 2, end);
        Some(end + 1)
    }

    fn push_footnote_call(&mut self, start: usize, label_start: usize, label_end: usize) {
        let raw = &self.text[label_start..label_end];
        self.items
            .push(Item::Node(Node::FootnoteReference(FootnoteReference {
                identifier: self
                    .alloc
                    .alloc_str(&normalize_identifier(raw).to_lowercase()),
                label: Some(clean(self.alloc, self.alloc.alloc_str(&decode_string(raw)))),
                position: Some(self.span(start, label_end + 1)),
                data: None,
            })));
    }

    // `]` after `![^a` that no image took: the `!` is text and `[^a]` a footnote call
    fn potential_footnote_call(&mut self, opener: usize, pos: usize) -> bool {
        let Some(Item::Open {
            start, image: true, ..
        }) = self.items.get(opener)
        else {
            return false;
        };
        let start = *start;
        if self.gfm.is_none()
            || self.footnotes.is_empty()
            || self.bytes.get(start + 2) != Some(&b'^')
            || start + 3 >= pos
            || matches!(self.bytes[start + 3], b'\t' | b'\n' | b'\r' | b' ')
            || self.footnote_label_end(start + 3) != Some(pos)
        {
            return false;
        }
        self.items.truncate(opener);
        let split = self.delims.partition_point(|delim| *delim < opener);
        self.delims.truncate(split);
        self.items.push(Item::Text {
            start,
            end: start + 1,
            value: None,
        });
        self.push_footnote_call(start + 1, start + 3, pos);
        true
    }

    fn open_bracket(&mut self, start: usize, image: bool) {
        self.brackets.push(Bracket {
            item: self.items.len(),
            image,
            active: true,
        });
        self.items.push(Item::Open {
            image,
            start,
            matched: false,
        });
    }

    // A `]`: a link or image when an opening bracket and what follows make one
    fn close_bracket(&mut self, pos: usize) -> usize {
        let Some(bracket) = self.brackets.last() else {
            self.items.push(Item::Text {
                start: pos,
                end: pos + 1,
                value: None,
            });
            return pos + 1;
        };
        let (opener, image, active) = (bracket.item, bracket.image, bracket.active);
        if !active {
            self.brackets.pop();
            self.items.push(Item::Text {
                start: pos,
                end: pos + 1,
                value: None,
            });
            return pos + 1;
        }
        let label_start = match self.items[opener] {
            Item::Open { start, image, .. } => start + if image { 2 } else { 1 },
            _ => pos,
        };
        let label = &self.text[label_start..pos];
        let found = self.match_media(label, pos + 1);
        let Some((media, end)) = found else {
            self.brackets.pop();
            if self.potential_footnote_call(opener, pos) {
                return pos + 1;
            }
            self.items.push(Item::Text {
                start: pos,
                end: pos + 1,
                value: None,
            });
            return pos + 1;
        };
        self.brackets.pop();
        if let Item::Open { matched, .. } = &mut self.items[opener] {
            *matched = true;
        }
        // Below an inactive link bracket all are inactive already, so the walk stops there
        if !image {
            for bracket in self.brackets.iter_mut().rev() {
                if !bracket.image {
                    if !bracket.active {
                        break;
                    }
                    bracket.active = false;
                }
            }
        }
        self.items.push(Item::Close {
            start: pos,
            end,
            media: Some(media),
        });
        end
    }

    fn is_defined(&self, raw: &str) -> bool {
        if self.identifiers.is_empty() {
            return false;
        }
        // Past the longest identifier, case mapping aside, no definition can match; this
        // keeps nested brackets from normalizing ever longer labels
        let cap = self.longest_identifier * 3 + 4;
        let mut size = 0;
        for byte in raw.bytes() {
            if !matches!(byte, b' ' | b'\t' | b'\n' | b'\r') {
                size += 1;
                if size > cap {
                    return false;
                }
            }
        }
        self.identifiers
            .contains(&normalize_identifier(raw).to_lowercase())
    }

    fn match_media(&self, label: &str, after: usize) -> Option<(Media<'a>, usize)> {
        let bytes = self.bytes;
        let defined = || self.is_defined(label);
        let shortcut = |kind| {
            Some((
                Media::Reference {
                    kind,
                    label: self.alloc.alloc_str(&decode_string(label)),
                    identifier: self
                        .alloc
                        .alloc_str(&normalize_identifier(label).to_lowercase()),
                },
                after,
            ))
        };
        match bytes.get(after) {
            Some(b'(') => {
                if let Some(found) = self.match_resource(after) {
                    return Some(found);
                }
                if defined() {
                    shortcut(ReferenceType::Shortcut)
                } else {
                    None
                }
            }
            Some(b'[') => {
                if let Some(end) = scan_label(bytes, after) {
                    let raw = &self.text[after + 1..end - 1];
                    if self.is_defined(raw) {
                        return Some((
                            Media::Reference {
                                kind: ReferenceType::Full,
                                label: self.alloc.alloc_str(&decode_string(raw)),
                                identifier: self
                                    .alloc
                                    .alloc_str(&normalize_identifier(raw).to_lowercase()),
                            },
                            end,
                        ));
                    }
                }
                if defined() && bytes.get(after + 1) == Some(&b']') {
                    let (media, _) = shortcut(ReferenceType::Collapsed)?;
                    return Some((media, after + 2));
                }
                None
            }
            _ => {
                if defined() {
                    shortcut(ReferenceType::Shortcut)
                } else {
                    None
                }
            }
        }
    }

    fn match_resource(&self, at: usize) -> Option<(Media<'a>, usize)> {
        let bytes = self.bytes;
        let (start, _) = skip_space_eol(bytes, at + 1);
        if bytes.get(start) == Some(&b')') {
            return Some((
                Media::Resource {
                    url: "",
                    title: None,
                },
                start + 1,
            ));
        }
        let (url_start, url_end, after_url) = scan_destination(bytes, start)?;
        let url = self.decode(url_start, url_end);
        let (next, _) = skip_space_eol(bytes, after_url);
        if bytes.get(next) == Some(&b')') {
            return Some((Media::Resource { url, title: None }, next + 1));
        }
        if next == after_url {
            return None;
        }
        let (title_start, title_end, after_title) = scan_title(bytes, next)?;
        let (close, _) = skip_space_eol(bytes, after_title);
        if bytes.get(close) != Some(&b')') {
            return None;
        }
        let title = (title_end > title_start).then(|| {
            let raw = &self.text[title_start..title_end];
            match strip_line_prefixes(raw) {
                std::borrow::Cow::Borrowed(_) => self.decode(title_start, title_end),
                std::borrow::Cow::Owned(stripped) => {
                    clean(self.alloc, self.alloc.alloc_str(&decode_string(&stripped)))
                }
            }
        });
        Some((Media::Resource { url, title }, close + 1))
    }

    fn decode(&self, start: usize, end: usize) -> &'a str {
        let raw = &self.text[start..end];
        if raw.bytes().any(|byte| byte == b'\\' || byte == b'&') {
            clean(self.alloc, self.alloc.alloc_str(&decode_string(raw)))
        } else {
            clean(self.alloc, raw)
        }
    }

    // Pair the runs at `delims`, item indices in order, as micromark's attention resolver:
    // each closer takes the nearest live opener, found at the top of a stack per bucket
    // Pair runs as micromark's resolvers do, after the whole text: the kind seen first
    // pairs first, inside each link; the other then pairs inside each link and each node
    // the first made
    fn resolve_emphasis(&mut self, delims: &[usize]) {
        let is_tilde = |inline: &Self, delim: usize| {
            inline
                .delim(delim)
                .is_some_and(|delim| delim.marker == b'~')
        };
        let Some(&first) = delims.first() else { return };
        let tilde_first = is_tilde(self, first);
        for pass in 0..2 {
            let tilde = tilde_first == (pass == 0);
            let pairs = pass == 1;
            for group in self.group_delims(delims, tilde, pairs, tilde_first) {
                if tilde {
                    self.resolve_strikethrough(&group);
                } else {
                    self.resolve_attention(&group);
                }
            }
        }
    }

    // The runs of one kind, grouped by the innermost link around them and, when `pairs`,
    // the innermost node the other kind made
    fn group_delims(
        &self,
        delims: &[usize],
        tilde: bool,
        pairs: bool,
        tilde_first: bool,
    ) -> Vec<Vec<usize>> {
        let mut groups: Vec<Vec<usize>> = vec![vec![]];
        let mut stack: Vec<usize> = vec![0];
        let first = delims[0];
        let last = delims[delims.len() - 1];
        let _ = first;
        for index in 0..=last {
            match &self.items[index] {
                Item::Open { matched: true, .. } => {
                    groups.push(vec![]);
                    stack.push(groups.len() - 1);
                }
                Item::Close { media: Some(_), .. } => {
                    if stack.len() > 1 {
                        stack.pop();
                    }
                }
                Item::Delim(delim) => {
                    let mine = (delim.marker == b'~') == tilde;
                    if mine {
                        if let Some(group) = stack.last() {
                            groups[*group].push(index);
                        }
                    } else if pairs && (delim.marker == b'~') == tilde_first {
                        for _ in &delim.closes {
                            if stack.len() > 1 {
                                stack.pop();
                            }
                        }
                        for _ in &delim.opens {
                            groups.push(vec![]);
                            stack.push(groups.len() - 1);
                        }
                    }
                }
                _ => {}
            }
        }
        groups.retain(|group| !group.is_empty());
        groups
    }

    // Pair `~` runs of one size, as micromark-extension-gfm-strikethrough: the nearest live
    // opener of the closer's size, every opener between it and the closer dead
    fn resolve_strikethrough(&mut self, delims: &[usize]) {
        let mut by_size: [Vec<usize>; 3] = Default::default();
        let mut openers: Vec<usize> = vec![];
        let mut live = std::collections::HashSet::new();
        for &close in delims {
            let Some(closer) = self.delim(close) else {
                continue;
            };
            let (size, can_open, can_close) = (closer.size, closer.open, closer.close);
            if can_close {
                let mut found = None;
                while let Some(top) = by_size[size].pop() {
                    if live.contains(&top) {
                        found = Some(top);
                        break;
                    }
                }
                if let Some(open) = found {
                    while let Some(top) = openers.pop_if(|top| *top >= open) {
                        live.remove(&top);
                    }
                    if let Item::Delim(opener) = &mut self.items[open] {
                        opener.left = 0;
                        opener.opens.push(size as u8);
                    }
                    if let Item::Delim(closer) = &mut self.items[close] {
                        closer.left = 0;
                        closer.closes.push(size as u8);
                    }
                    continue;
                }
            }
            if can_open {
                live.insert(close);
                by_size[size].push(close);
                openers.push(close);
            }
        }
    }

    fn resolve_attention(&mut self, delims: &[usize]) {
        let mut buckets: [Vec<usize>; 12] = Default::default();
        let mut openers: Vec<usize> = vec![];
        for &close in delims {
            while let Some(closer) = self.delim(close) {
                if !closer.close || closer.left == 0 {
                    break;
                }
                let (marker, closer_left, closer_open) =
                    (closer.marker, closer.left, closer.open);
                let mut best: Option<usize> = None;
                for size in 0..3 {
                    for can_close in [false, true] {
                        // The rule of three
                        if (can_close || closer_open)
                            && closer_left % 3 != 0
                            && (size + closer_left) % 3 == 0
                        {
                            continue;
                        }
                        let key = bucket_index(marker, size, can_close);
                        while let Some(&top) = buckets[key].last() {
                            match self.delim(top) {
                                Some(opener)
                                    if opener.open
                                        && opener.left > 0
                                        && bucket_of(opener) == key =>
                                {
                                    break;
                                }
                                _ => {
                                    buckets[key].pop();
                                }
                            }
                        }
                        if let Some(&top) = buckets[key].last()
                            && best.is_none_or(|best| top > best)
                        {
                            best = Some(top);
                        }
                    }
                }
                let Some(open) = best else { break };
                let take = match self.delim(open) {
                    Some(opener) if opener.left > 1 && closer_left > 1 => 2,
                    _ => 1,
                };
                if let Item::Delim(opener) = &mut self.items[open] {
                    opener.left -= take;
                    opener.opens.push(take as u8);
                }
                if let Item::Delim(closer) = &mut self.items[close] {
                    closer.left -= take;
                    closer.closes.push(take as u8);
                }
                while openers.last().is_some_and(|last| *last > open) {
                    if let Some(between) = openers.pop()
                        && let Item::Delim(delim) = &mut self.items[between]
                    {
                        delim.open = false;
                    }
                }
                if let Some(opener) = self.delim(open)
                    && opener.left > 0
                {
                    let key = bucket_of(opener);
                    buckets[key].push(open);
                }
            }
            if let Some(closer) = self.delim(close)
                && closer.open
                && closer.left > 0
            {
                let key = bucket_of(closer);
                buckets[key].push(close);
                openers.push(close);
            }
        }
    }

    fn delim(&self, index: usize) -> Option<&Delim> {
        match &self.items[index] {
            Item::Delim(delim) => Some(delim),
            _ => None,
        }
    }

    fn build(self) -> ArenaVec<'a, Node<'a>> {
        let mut builder = Builder {
            alloc: self.alloc,
            text: self.text,
            segments: self.segments,
            mdx: self.mdx,
            leaf: self.leaf,
            jsx_frames: vec![],
            frames: vec![Frame {
                kind: FrameKind::Root,
                start: 0,
                children: ArenaVec::new_in(self.alloc),
            }],
            pending: None,
        };
        for item in self.items {
            match item {
                Item::Text { start, end, value } => builder.add_text(start, end, value),
                Item::Jsx(tag) => {
                    builder.flush();
                    if !builder.jsx(tag) {
                        return ArenaVec::new_in(self.alloc);
                    }
                }
                Item::Node(node) => {
                    builder.flush();
                    builder.push(node);
                }
                Item::Open {
                    image,
                    start,
                    matched,
                } => {
                    if matched {
                        builder.flush();
                        builder.frames.push(Frame {
                            kind: FrameKind::Media { image },
                            start,
                            children: ArenaVec::new_in(builder.alloc),
                        });
                    } else {
                        builder.add_text(start, start + if image { 2 } else { 1 }, None);
                    }
                }
                Item::Close { start, end, media } => match media {
                    Some(media) => {
                        builder.flush();
                        builder.close_media(media, end);
                    }
                    None => builder.add_text(start, start + 1, None),
                },
                Item::Delim(delim) => {
                    let mut at = delim.start;
                    for take in &delim.closes {
                        at += *take as usize;
                        builder.flush();
                        builder.close_attention(at);
                    }
                    if delim.left > 0 {
                        builder.add_text(at, at + delim.left, None);
                    }
                    let mut at = delim.start + delim.size;
                    let total: usize =
                        delim.opens.iter().map(|take| *take as usize).sum();
                    at -= total;
                    if !delim.opens.is_empty() {
                        builder.flush();
                    }
                    for take in delim.opens.iter().rev() {
                        let take = *take as usize;
                        builder.frames.push(Frame {
                            kind: if delim.marker == b'~' {
                                FrameKind::Delete
                            } else if take == 2 {
                                FrameKind::Strong
                            } else {
                                FrameKind::Emphasis
                            },
                            start: at,
                            children: ArenaVec::new_in(builder.alloc),
                        });
                        at += take;
                    }
                }
            }
        }
        builder.flush();
        // An element still open where the leaf ends
        if let Some(Frame {
            kind: FrameKind::Jsx(tag),
            ..
        }) = builder.frames.last()
        {
            let message = unclosed_error(
                tag.name.as_deref(),
                &builder.place(tag.start, tag.end),
                Some(builder.leaf),
            );
            builder.fail(message);
        }
        builder
            .frames
            .into_iter()
            .next()
            .map(|frame| frame.children)
            .unwrap_or_else(|| ArenaVec::new_in(self.alloc))
    }

    fn span(&self, start: usize, end: usize) -> Position<'a> {
        Position::span(map_start(self.segments, start), map_end(self.segments, end))
    }
}

/// A value as micromark reads it: NUL is U+FFFD.
pub(crate) fn clean<'a>(alloc: &'a Allocator, value: &'a str) -> &'a str {
    if value.as_bytes().contains(&0) {
        alloc.alloc_str(&value.replace('\0', "\u{FFFD}"))
    } else {
        value
    }
}

/// Segments of a text's range `start..end`, counted from `start`.
pub(crate) fn shift_segments(
    segments: &[(usize, usize)],
    start: usize,
    end: usize,
) -> Vec<(usize, usize)> {
    let first = segments
        .partition_point(|segment| segment.0 <= start)
        .max(1)
        - 1;
    let mut shifted = Vec::with_capacity(4);
    for (index, &(virtual_start, source_start)) in segments.iter().enumerate().skip(first)
    {
        if virtual_start >= end && index > first {
            break;
        }
        if index == first {
            shifted.push((0, source_start + start - virtual_start));
        } else {
            shifted.push((virtual_start - start, source_start));
        }
    }
    shifted
}

fn bucket_index(marker: u8, size: usize, can_close: bool) -> usize {
    let marker = match marker {
        b'*' => 0,
        _ => 1,
    };
    marker * 6 + size * 2 + usize::from(can_close)
}

fn bucket_of(delim: &Delim) -> usize {
    bucket_index(delim.marker, delim.left % 3, delim.close)
}

fn after_line_ending(bytes: &[u8], at: usize) -> usize {
    if bytes.get(at) == Some(&b'\r') && bytes.get(at + 1) == Some(&b'\n') {
        at + 2
    } else {
        at + 1
    }
}

impl Inline<'_, '_> {
    // A run of `size` markers closing a code span or math; a failed search is
    // remembered, so later runs of that size give up at once
    fn find_run_close(&mut self, marker: u8, from: usize, size: usize) -> Option<usize> {
        let key = (marker, size);
        if let Some((_, failed)) = self.code_memo.iter().find(|(memo, _)| *memo == key)
            && from >= *failed
        {
            return None;
        }
        let found = find_code_close(self.bytes, marker, from, size);
        if found.is_none() {
            match self.code_memo.iter_mut().find(|(memo, _)| *memo == key) {
                Some(entry) => entry.1 = entry.1.min(from),
                None => self.code_memo.push((key, from)),
            }
        }
        found
    }
}

fn find_code_close(bytes: &[u8], marker: u8, from: usize, size: usize) -> Option<usize> {
    let mut index = from;
    while index < bytes.len() {
        if bytes[index] == marker {
            let start = index;
            while index < bytes.len() && bytes[index] == marker {
                index += 1;
            }
            if index - start == size {
                return Some(start);
            }
        } else {
            index += 1;
        }
    }
    None
}

pub(crate) fn map_start(segments: &[(usize, usize)], at: usize) -> Point {
    let index = segments.partition_point(|segment| segment.0 <= at).max(1) - 1;
    let (virtual_start, source_start) = segments[index];
    Point::new(source_start + at - virtual_start, 0)
}

// An end at a line's start is the end of the line ending before it
pub(crate) fn map_end(segments: &[(usize, usize)], at: usize) -> Point {
    let index = segments.partition_point(|segment| segment.0 < at).max(1) - 1;
    let (virtual_start, source_start) = segments[index];
    Point::new(source_start + at - virtual_start, 0)
}

enum FrameKind {
    Root,
    Jsx(ScannedTag),
    Delete,
    Emphasis,
    Strong,
    Media { image: bool },
}

struct Frame<'a> {
    kind: FrameKind,
    start: usize,
    children: ArenaVec<'a, Node<'a>>,
}

struct Pending<'a> {
    start: usize,
    end: usize,
    owned: Option<ArenaString<'a>>,
}

struct Builder<'a, 'd> {
    alloc: &'a Allocator,
    text: &'a str,
    segments: &'d [(usize, usize)],
    mdx: Option<&'d MdxBuild<'d>>,
    leaf: &'static str,
    // Where the JSX frames are among `frames`
    jsx_frames: Vec<usize>,
    frames: Vec<Frame<'a>>,
    pending: Option<Pending<'a>>,
}

impl<'a> Builder<'a, '_> {
    fn fail(&self, error: Error) {
        if let Some(mdx) = self.mdx {
            mdx.errors.borrow_mut().compile(error);
        }
    }

    fn place(&self, start: usize, end: usize) -> String {
        let source = self.mdx.map_or("", |mdx| mdx.source);
        format!(
            "{}-{}",
            stringify_point(source, map_start(self.segments, start)),
            stringify_point(source, map_end(self.segments, end))
        )
    }

    // An element left open inside emphasis or a link that closes: upstream's error
    fn jsx_open_at(&mut self, _end: usize) -> bool {
        let Some(Frame {
            kind: FrameKind::Jsx(tag),
            ..
        }) = self.frames.last()
        else {
            return false;
        };
        let closing = self.frames.iter().rev().find_map(|frame| match frame.kind {
            FrameKind::Emphasis => Some("emphasis"),
            FrameKind::Strong => Some("strong"),
            FrameKind::Delete => Some("strikethrough"),
            FrameKind::Media { image: false } => Some("link"),
            FrameKind::Media { image: true } => Some("image"),
            _ => None,
        });
        let message = unclosed_error(
            tag.name.as_deref(),
            &self.place(tag.start, tag.end),
            closing,
        );
        self.fail(message);
        true
    }

    // A JSX tag: it opens an element, closes one, or both; `false` once it fails
    fn jsx(&mut self, tag: ScannedTag) -> bool {
        let flow_count = self.mdx.map_or(0, |mdx| mdx.flow_count);
        if let Some(error) =
            check_tag(&tag, !self.jsx_frames.is_empty() || flow_count > 0)
        {
            self.fail(error);
            return false;
        }
        if tag.close {
            let text_innermost = self.jsx_frames.last().and_then(|index| {
                match &self.frames[*index].kind {
                    FrameKind::Jsx(open) => {
                        Some((open.name.clone(), Some((open.start, open.end))))
                    }
                    _ => None,
                }
            });
            let innermost = text_innermost.or_else(|| {
                self.mdx
                    .and_then(|mdx| mdx.flow_open.clone())
                    .map(|(name, _, _)| (name, None::<(usize, usize)>))
            });
            if let Some((name, span)) = innermost
                && name != tag.name
            {
                let place = match span {
                    Some((start, end)) => self.place(start, end),
                    None => self.flow_place(),
                };
                let message = mismatch_error(&tag, name.as_deref(), &place);
                self.fail(message);
                return false;
            }
        }
        if !tag.close {
            self.jsx_frames.push(self.frames.len());
            self.frames.push(Frame {
                kind: FrameKind::Jsx(tag.clone()),
                start: tag.start,
                children: ArenaVec::new_in(self.alloc),
            });
        }
        if tag.self_closing || tag.close {
            let tail = self.frames.last().map(|frame| match frame.kind {
                FrameKind::Jsx(_) => None,
                FrameKind::Root => Some((self.leaf, self.leaf_end(), self.point(0))),
                FrameKind::Emphasis => {
                    Some(("emphasis", String::new(), self.point(frame.start)))
                }
                FrameKind::Strong => {
                    Some(("strong", String::new(), self.point(frame.start)))
                }
                FrameKind::Delete => {
                    Some(("strikethrough", String::new(), self.point(frame.start)))
                }
                FrameKind::Media { .. } => {
                    Some(("label", String::new(), self.point(frame.start)))
                }
            });
            match tail {
                Some(None) => {}
                Some(Some((kind, exit, enter))) => {
                    let message = tail_error(&tag, kind, &exit, &enter);
                    self.fail(message);
                    return false;
                }
                None => return false,
            }
            let Some(frame) = self.frames.pop() else {
                return false;
            };
            let FrameKind::Jsx(open) = frame.kind else {
                return false;
            };
            self.jsx_frames.pop();
            let segments = self.segments;
            let span = |start: usize, end: usize| {
                Position::span(map_start(segments, start), map_end(segments, end))
            };
            let attributes = build_attributes(self.alloc, &open, self.text, span);
            let name = open
                .name
                .as_deref()
                .map(|name| &*self.alloc.alloc_str(name));
            let node = Node::MdxJsxTextElement(MdxJsxElement {
                name,
                attributes,
                children: frame.children,
                position: Some(self.span(open.start, tag.end)),
                data: None,
            });
            self.push(node);
        }
        true
    }

    // The place of the innermost flow element's tag, in the source
    fn flow_place(&self) -> String {
        match self.mdx.and_then(|mdx| {
            mdx.flow_open
                .as_ref()
                .map(|open| (mdx.source, open.1, open.2))
        }) {
            Some((source, start, end)) => format!(
                "{}-{}",
                stringify_point(source, Point::new(start, 0)),
                stringify_point(source, Point::new(end, 0))
            ),
            None => String::new(),
        }
    }

    fn point(&self, at: usize) -> String {
        let source = self.mdx.map_or("", |mdx| mdx.source);
        stringify_point(source, map_start(self.segments, at))
    }

    fn leaf_end(&self) -> String {
        let source = self.mdx.map_or("", |mdx| mdx.source);
        stringify_point(source, map_end(self.segments, self.text.len()))
    }

    fn span(&self, start: usize, end: usize) -> Position<'a> {
        Position::span(map_start(self.segments, start), map_end(self.segments, end))
    }

    fn push(&mut self, node: Node<'a>) {
        if let Some(frame) = self.frames.last_mut() {
            frame.children.push(node);
        }
    }

    fn add_text(&mut self, start: usize, end: usize, value: Option<&str>) {
        match &mut self.pending {
            None => {
                self.pending = Some(Pending {
                    start,
                    end,
                    owned: value.map(|value| {
                        let mut owned =
                            ArenaString::with_capacity_in(value.len(), self.alloc);
                        owned.push_str(value);
                        owned
                    }),
                });
            }
            Some(pending) => {
                if pending.owned.is_none() && value.is_none() && pending.end == start {
                    pending.end = end;
                    return;
                }
                let owned = pending.owned.get_or_insert_with(|| {
                    let mut owned = ArenaString::with_capacity_in(
                        pending.end - pending.start + 16,
                        self.alloc,
                    );
                    owned.push_str(&self.text[pending.start..pending.end]);
                    owned
                });
                owned.push_str(value.unwrap_or(&self.text[start..end]));
                pending.end = end;
            }
        }
    }

    fn flush(&mut self) {
        if let Some(pending) = self.pending.take() {
            let value = match pending.owned {
                Some(owned) => owned.into_bump_str(),
                None => &self.text[pending.start..pending.end],
            };
            let value = clean(self.alloc, value);
            let position = Some(self.span(pending.start, pending.end));
            self.push(Node::Text(Text {
                value,
                position,
                data: None,
            }));
        }
    }

    fn close_attention(&mut self, end: usize) {
        if self.jsx_open_at(end) {
            return;
        }
        let Some(frame) = self.frames.pop() else {
            return;
        };
        let position = Some(self.span(frame.start, end));
        let node = match frame.kind {
            FrameKind::Strong => Node::Strong(Strong {
                children: frame.children,
                position,
                data: None,
            }),
            FrameKind::Delete => Node::Delete(Delete {
                children: frame.children,
                position,
                data: None,
            }),
            _ => Node::Emphasis(Emphasis {
                children: frame.children,
                position,
                data: None,
            }),
        };
        self.push(node);
    }

    fn close_media(&mut self, media: Media<'a>, end: usize) {
        if self.jsx_open_at(end) {
            return;
        }
        let Some(frame) = self.frames.pop() else {
            return;
        };
        let position = Some(self.span(frame.start, end));
        let image = matches!(frame.kind, FrameKind::Media { image: true });
        let node = match (media, image) {
            (Media::Resource { url, title }, false) => Node::Link(Link {
                url,
                title,
                children: frame.children,
                position,
                data: None,
            }),
            (Media::Resource { url, title }, true) => Node::Image(Image {
                url,
                title,
                alt: Some(self.text_of(&frame.children)),
                position,
                data: None,
            }),
            (
                Media::Reference {
                    kind,
                    label,
                    identifier,
                },
                false,
            ) => Node::LinkReference(LinkReference {
                identifier,
                label: Some(label),
                reference_type: kind,
                children: frame.children,
                position,
                data: None,
            }),
            (
                Media::Reference {
                    kind,
                    label,
                    identifier,
                },
                true,
            ) => Node::ImageReference(ImageReference {
                identifier,
                label: Some(label),
                reference_type: kind,
                alt: Some(self.text_of(&frame.children)),
                position,
                data: None,
            }),
        };
        self.push(node);
    }

    fn text_of(&self, children: &[Node<'a>]) -> &'a str {
        let mut value = ArenaString::new_in(self.alloc);
        let mut stack: Vec<&Node<'a>> = children.iter().rev().collect();
        while let Some(node) = stack.pop() {
            match node {
                Node::Text(text) => value.push_str(text.value),
                Node::InlineCode(code) => value.push_str(code.value),
                Node::Image(image) => value.push_str(image.alt.unwrap_or("")),
                Node::ImageReference(image) => value.push_str(image.alt.unwrap_or("")),
                _ => {}
            }
            if let Some(children) = node.children() {
                stack.extend(children.iter().rev());
            }
        }
        value.into_bump_str()
    }
}

// Drop the whitespace after a task list check from the first text, as
// mdast-util-gfm-task-list-item does; its start moves one column on, even into a tab
fn strip_task_space(children: &mut ArenaVec<'_, Node<'_>>) {
    let Some(Node::Text(head)) = children.first_mut() else {
        return;
    };
    let space = head.value.starts_with(' ');
    let first = head.value.chars().next().map_or(0, char::len_utf8);
    head.value = &head.value[first..];
    if head.value.is_empty() {
        children.remove(0);
        return;
    }
    if let Some(Position::Span { start, .. }) = &mut head.position {
        *start = if space {
            Point::new(start.offset() + 1, 0)
        } else {
            Point::new(start.offset(), start.virtual_spaces() + 1)
        };
    }
}

// As mdast-util-gfm-table: `\|` is `|` in a table, while `\\` keeps both backslashes
fn unescape_pipes(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\\'
            && let Some(next @ ('\\' | '|')) = chars.peek().copied()
        {
            if next == '\\' {
                result.push(character);
            }
            result.push(next);
            chars.next();
            continue;
        }
        result.push(character);
    }
    result
}

// As micromark's code text resolver: padding goes when both sides have it and the code is
// not only padding
pub(crate) fn strip_padding(value: &str) -> &str {
    let is_padding = |c: char| matches!(c, ' ' | '\n' | '\r');
    if value.chars().all(is_padding) {
        return value;
    }
    let strip_start = |text: &str| -> Option<usize> {
        match text.as_bytes() {
            [b'\r', b'\n', ..] => Some(2),
            [b' ' | b'\n' | b'\r', ..] => Some(1),
            _ => None,
        }
    };
    let strip_end = |text: &str| -> Option<usize> {
        match text.as_bytes() {
            [.., b'\r', b'\n'] => Some(2),
            [.., b' ' | b'\n' | b'\r'] => Some(1),
            _ => None,
        }
    };
    match (strip_start(value), strip_end(value)) {
        (Some(start), Some(end)) => &value[start..value.len() - end],
        _ => value,
    }
}
