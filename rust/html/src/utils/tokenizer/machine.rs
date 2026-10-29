use std::collections::{HashSet, VecDeque};

use crate::helpers::tag::{TagId, find_tag_id};
use crate::models::parse_error::{ErrorCode, ParseError};
use crate::models::token::{
    Attribute, CharacterKind, CharacterToken, CommentToken, DoctypeToken, Span, TagToken,
    Token,
};
use crate::utils::tokenizer::entity::{DecodeMode, decode_reference};
use crate::utils::tokenizer::preprocess::Preprocessor;

/// parse5's tokenizer states; the tree builder sets the first five and `CdataSection`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum State {
    Data,
    Rcdata,
    Rawtext,
    ScriptData,
    Plaintext,
    TagOpen,
    EndTagOpen,
    TagName,
    RcdataLessThanSign,
    RcdataEndTagOpen,
    RcdataEndTagName,
    RawtextLessThanSign,
    RawtextEndTagOpen,
    RawtextEndTagName,
    ScriptDataLessThanSign,
    ScriptDataEndTagOpen,
    ScriptDataEndTagName,
    ScriptDataEscapeStart,
    ScriptDataEscapeStartDash,
    ScriptDataEscaped,
    ScriptDataEscapedDash,
    ScriptDataEscapedDashDash,
    ScriptDataEscapedLessThanSign,
    ScriptDataEscapedEndTagOpen,
    ScriptDataEscapedEndTagName,
    ScriptDataDoubleEscapeStart,
    ScriptDataDoubleEscaped,
    ScriptDataDoubleEscapedDash,
    ScriptDataDoubleEscapedDashDash,
    ScriptDataDoubleEscapedLessThanSign,
    ScriptDataDoubleEscapeEnd,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    BogusComment,
    MarkupDeclarationOpen,
    CommentStart,
    CommentStartDash,
    Comment,
    CommentLessThanSign,
    CommentLessThanSignBang,
    CommentLessThanSignBangDash,
    CommentLessThanSignBangDashDash,
    CommentEndDash,
    CommentEnd,
    CommentEndBang,
    Doctype,
    BeforeDoctypeName,
    DoctypeName,
    AfterDoctypeName,
    AfterDoctypePublicKeyword,
    BeforeDoctypePublicIdentifier,
    DoctypePublicIdentifierDoubleQuoted,
    DoctypePublicIdentifierSingleQuoted,
    AfterDoctypePublicIdentifier,
    BetweenDoctypePublicAndSystemIdentifiers,
    AfterDoctypeSystemKeyword,
    BeforeDoctypeSystemIdentifier,
    DoctypeSystemIdentifierDoubleQuoted,
    DoctypeSystemIdentifierSingleQuoted,
    AfterDoctypeSystemIdentifier,
    BogusDoctype,
    CdataSection,
    CdataSectionBracket,
    CdataSectionEnd,
    CharacterReference,
    AmbiguousAmpersand,
}

/// parse5's HTML tokenizer, pulled one token at a time so the tree builder can switch
/// `state` and `in_foreign_node` between tokens, as parse5's callbacks do.
#[derive(Debug)]
pub(crate) struct Tokenizer<'s> {
    pub(crate) state: State,
    /// Whether the adjusted current node is foreign, which lets `<![CDATA[` open a section.
    pub(crate) in_foreign_node: bool,
    pub(crate) last_start_tag_name: String,
    /// Tokenizer and preprocessor errors, in the order parse5 reports them.
    pub(crate) errors: Vec<ParseError>,
    preprocessor: Preprocessor<'s>,
    return_state: State,
    queue: VecDeque<Token>,
    eof: Option<Span>,
    character: Option<CharacterToken>,
    /// parse5's `currentLocation`: where the next character token or doctype starts.
    location: usize,
    // Whether parse5's `currentLocation` is set: hast-util-raw clears it, and a reset
    // drops pending characters that were read without one
    located: bool,
    character_located: bool,
    tag: TagToken,
    end_tag: bool,
    attribute_name: String,
    /// The attribute being read, unless its name repeats an earlier one.
    attribute_index: Option<usize>,
    attribute_names: HashSet<String>,
    comment: CommentToken,
    doctype: DoctypeToken,
    reference_start: usize,
}

impl<'s> Tokenizer<'s> {
    pub(crate) fn new(input: &'s str) -> Self {
        Self {
            state: State::Data,
            in_foreign_node: false,
            last_start_tag_name: String::new(),
            errors: Vec::new(),
            preprocessor: Preprocessor::new(input),
            return_state: State::Data,
            queue: VecDeque::new(),
            eof: None,
            character: None,
            location: 0,
            located: true,
            character_located: true,
            tag: create_tag(0),
            end_tag: false,
            attribute_name: String::new(),
            attribute_index: None,
            attribute_names: HashSet::new(),
            comment: CommentToken {
                data: String::new(),
                span: Span::default(),
            },
            doctype: create_doctype(None, 0),
            reference_start: 0,
        }
    }

    /// The next token; `Eof` again once the input is done.
    pub(crate) fn next_token(&mut self) -> Token {
        loop {
            if let Some(token) = self.queue.pop_front() {
                return token;
            }
            if let Some(span) = self.eof {
                return Token::Eof(span);
            }
            let char = self.consume();
            self.call_state(char);
        }
    }

    /// The next token of a chunk written with `write`; `None` once the chunk is used up
    /// and the tokenizer waits for more, keeping its state, as parse5 hibernates.
    pub(crate) fn next_chunk_token(&mut self) -> Option<Token> {
        loop {
            if let Some(token) = self.queue.pop_front() {
                return Some(token);
            }
            if self.preprocessor.chunk_end {
                return None;
            }
            let char = self.consume();
            if self.preprocessor.chunk_end {
                return None;
            }
            self.call_state(char);
        }
    }

    /// Continue on a new chunk that is not the last, dropping what was left of the last
    /// one, as hast-util-raw resets parse5's preprocessor for each raw node; a chunk with
    /// a position sets parse5's `currentLocation`.
    pub(crate) fn write(&mut self, chunk: &'s str, located: bool) {
        self.preprocessor = Preprocessor::new(chunk);
        self.preprocessor.last_chunk = false;
        self.location = 0;
        self.located = self.located || located;
        if self.state == State::CharacterReference {
            self.state = self.return_state;
        }
    }

    /// hast-util-raw's `resetTokenizer`: drop the token being built and hand back the
    /// pending characters, unless read without a location, keeping `state`,
    /// `in_foreign_node` and `last_start_tag_name`.
    pub(crate) fn reset(&mut self) -> Option<CharacterToken> {
        let pos = self.pos();
        let located = self.character_located;
        let character = self
            .character
            .take()
            .filter(|_| located)
            .map(|mut character| {
                character.span.end = pos;
                character
            });
        self.located = false;
        self.queue.clear();
        self.return_state = State::Data;
        self.tag = create_tag(0);
        self.end_tag = false;
        self.attribute_name.clear();
        self.attribute_index = None;
        self.attribute_names.clear();
        self.comment = CommentToken {
            data: String::new(),
            span: Span::default(),
        };
        self.doctype = create_doctype(None, 0);
        character
    }

    fn consume(&mut self) -> Option<char> {
        self.preprocessor.advance(&mut self.errors)
    }

    fn pos(&self) -> usize {
        self.preprocessor.pos
    }

    fn error(&mut self, code: ErrorCode) {
        self.error_at(code, self.pos());
    }

    fn error_at(&mut self, code: ErrorCode, pos: usize) {
        self.errors.push(ParseError {
            code,
            start: pos,
            end: pos,
        });
    }

    fn consume_sequence(&mut self, pattern: &str, case_sensitive: bool) -> bool {
        if self
            .preprocessor
            .match_sequence(pattern, case_sensitive)
            .is_none()
        {
            return false;
        }
        for _ in 1..pattern.encode_utf16().count() {
            self.consume();
        }
        true
    }

    // Token creation and emission

    fn create_start_tag(&mut self) {
        self.tag = create_tag(self.pos().saturating_sub(1));
        self.end_tag = false;
        self.attribute_names.clear();
    }

    fn create_end_tag(&mut self) {
        self.tag = create_tag(self.pos().saturating_sub(2));
        self.end_tag = true;
        self.attribute_names.clear();
    }

    fn create_comment(&mut self, offset: usize) {
        self.comment = CommentToken {
            data: String::new(),
            span: Span {
                start: self.pos().saturating_sub(offset),
                end: 0,
            },
        };
    }

    fn create_doctype(&mut self, name: Option<String>) {
        self.doctype = create_doctype(name, self.location);
    }

    fn create_attribute(&mut self, first: &str) {
        self.attribute_name.clear();
        self.attribute_name.push_str(first);
        self.attribute_index = None;
        self.location = self.pos();
        self.located = true;
    }

    fn leave_attribute_name(&mut self) {
        if self.attribute_names.contains(&self.attribute_name) {
            self.error(ErrorCode::DuplicateAttribute);
            return;
        }
        let name = std::mem::take(&mut self.attribute_name);
        self.attribute_names.insert(name.clone());
        self.attribute_index = Some(self.tag.attrs.len());
        self.tag.attrs.push(Attribute {
            name,
            namespace: None,
            prefix: None,
            value: String::new(),
            span: Span {
                start: self.location,
                end: self.pos(),
            },
        });
    }

    fn leave_attribute_value(&mut self) {
        let pos = self.pos();
        if let Some(attribute) = self.current_attribute() {
            attribute.span.end = pos;
        }
    }

    fn current_attribute(&mut self) -> Option<&mut Attribute> {
        self.attribute_index
            .and_then(|index| self.tag.attrs.get_mut(index))
    }

    fn push_attribute_value(&mut self, char: char) {
        if let Some(attribute) = self.current_attribute() {
            attribute.value.push(char);
        }
    }

    /// Flushes pending characters up to `start`, where the token being emitted starts,
    /// and returns where that token ends.
    fn prepare_token(&mut self, start: usize) -> usize {
        self.flush_character(start);
        let end = self.pos() + 1;
        self.location = end;
        self.located = true;
        end
    }

    fn emit_tag(&mut self) {
        let mut tag = std::mem::replace(&mut self.tag, create_tag(0));
        tag.span.end = self.prepare_token(tag.span.start);
        tag.tag_id = find_tag_id(&tag.tag_name);
        if self.end_tag {
            if !tag.attrs.is_empty() {
                self.error(ErrorCode::EndTagWithAttributes);
            }
            if tag.self_closing {
                self.error(ErrorCode::EndTagWithTrailingSolidus);
            }
            self.queue.push_back(Token::EndTag(tag));
        } else {
            self.last_start_tag_name.clone_from(&tag.tag_name);
            self.queue.push_back(Token::StartTag(tag));
        }
    }

    fn emit_comment(&mut self) {
        let mut comment = std::mem::replace(
            &mut self.comment,
            CommentToken {
                data: String::new(),
                span: Span::default(),
            },
        );
        comment.span.end = self.prepare_token(comment.span.start);
        self.queue.push_back(Token::Comment(comment));
    }

    fn emit_doctype(&mut self) {
        let mut doctype = std::mem::replace(&mut self.doctype, create_doctype(None, 0));
        doctype.span.end = self.prepare_token(doctype.span.start);
        self.queue.push_back(Token::Doctype(doctype));
    }

    fn emit_eof(&mut self) {
        let pos = self.pos();
        self.flush_character(pos);
        let span = Span {
            start: pos,
            end: pos,
        };
        self.queue.push_back(Token::Eof(span));
        self.eof = Some(span);
    }

    fn flush_character(&mut self, end: usize) {
        if let Some(mut character) = self.character.take() {
            character.span.end = end;
            self.queue.push_back(Token::Character(character));
        }
    }

    fn append_character(&mut self, kind: CharacterKind, chars: &str) {
        if let Some(character) = &mut self.character {
            if character.kind == kind {
                character.chars.push_str(chars);
                return;
            }
            self.location = self.pos();
            self.located = true;
            self.flush_character(self.location);
        }
        self.character_located = self.located;
        self.character = Some(CharacterToken {
            kind,
            chars: chars.to_owned(),
            span: Span {
                start: self.location,
                end: 0,
            },
        });
    }

    fn emit_char(&mut self, char: char) {
        let kind = match char {
            '\t' | '\n' | '\x0c' | ' ' => CharacterKind::Whitespace,
            '\0' => CharacterKind::Null,
            _ => CharacterKind::Character,
        };
        self.append_character(kind, char.encode_utf8(&mut [0; 4]));
    }

    fn emit_chars(&mut self, chars: &str) {
        self.append_character(CharacterKind::Character, chars);
    }

    // Character references

    fn start_character_reference(&mut self) {
        self.return_state = self.state;
        self.state = State::CharacterReference;
        self.reference_start = self.pos();
    }

    fn is_reference_in_attribute(&self) -> bool {
        matches!(
            self.return_state,
            State::AttributeValueDoubleQuoted
                | State::AttributeValueSingleQuoted
                | State::AttributeValueUnquoted
        )
    }

    fn flush_reference_char(&mut self, char: char) {
        if self.is_reference_in_attribute() {
            self.push_attribute_value(char);
        } else {
            self.emit_char(char);
        }
    }

    fn call_state(&mut self, char: Option<char>) {
        match self.state {
            State::Data => self.data(char),
            State::Rcdata => self.rcdata(char),
            State::Rawtext => self.rawtext(char),
            State::ScriptData => self.script_data(char),
            State::Plaintext => self.plaintext(char),
            State::TagOpen => self.tag_open(char),
            State::EndTagOpen => self.end_tag_open(char),
            State::TagName => self.tag_name(char),
            State::RcdataLessThanSign => {
                self.text_less_than_sign(char, State::Rcdata, State::RcdataEndTagOpen);
            }
            State::RcdataEndTagOpen => {
                self.text_end_tag_open(char, State::Rcdata, State::RcdataEndTagName);
            }
            State::RcdataEndTagName => self.text_end_tag_name(char, State::Rcdata),
            State::RawtextLessThanSign => {
                self.text_less_than_sign(char, State::Rawtext, State::RawtextEndTagOpen);
            }
            State::RawtextEndTagOpen => {
                self.text_end_tag_open(char, State::Rawtext, State::RawtextEndTagName);
            }
            State::RawtextEndTagName => self.text_end_tag_name(char, State::Rawtext),
            State::ScriptDataLessThanSign => self.script_data_less_than_sign(char),
            State::ScriptDataEndTagOpen => {
                self.text_end_tag_open(
                    char,
                    State::ScriptData,
                    State::ScriptDataEndTagName,
                );
            }
            State::ScriptDataEndTagName => {
                self.text_end_tag_name(char, State::ScriptData)
            }
            State::ScriptDataEscapeStart => self.script_data_escape_start(char),
            State::ScriptDataEscapeStartDash => self.script_data_escape_start_dash(char),
            State::ScriptDataEscaped => self.script_data_escaped(char),
            State::ScriptDataEscapedDash => self.script_data_escaped_dash(char),
            State::ScriptDataEscapedDashDash => self.script_data_escaped_dash_dash(char),
            State::ScriptDataEscapedLessThanSign => {
                self.script_data_escaped_less_than_sign(char);
            }
            State::ScriptDataEscapedEndTagOpen => self.text_end_tag_open(
                char,
                State::ScriptDataEscaped,
                State::ScriptDataEscapedEndTagName,
            ),
            State::ScriptDataEscapedEndTagName => {
                self.text_end_tag_name(char, State::ScriptDataEscaped);
            }
            State::ScriptDataDoubleEscapeStart => {
                self.script_data_double_escape_start(char)
            }
            State::ScriptDataDoubleEscaped => self.script_data_double_escaped(char),
            State::ScriptDataDoubleEscapedDash => {
                self.script_data_double_escaped_dash(char)
            }
            State::ScriptDataDoubleEscapedDashDash => {
                self.script_data_double_escaped_dash_dash(char);
            }
            State::ScriptDataDoubleEscapedLessThanSign => {
                self.script_data_double_escaped_less_than_sign(char);
            }
            State::ScriptDataDoubleEscapeEnd => self.script_data_double_escape_end(char),
            State::BeforeAttributeName => self.before_attribute_name(char),
            State::AttributeName => self.attribute_name(char),
            State::AfterAttributeName => self.after_attribute_name(char),
            State::BeforeAttributeValue => self.before_attribute_value(char),
            State::AttributeValueDoubleQuoted => self.attribute_value_quoted(char, '"'),
            State::AttributeValueSingleQuoted => self.attribute_value_quoted(char, '\''),
            State::AttributeValueUnquoted => self.attribute_value_unquoted(char),
            State::AfterAttributeValueQuoted => self.after_attribute_value_quoted(char),
            State::SelfClosingStartTag => self.self_closing_start_tag(char),
            State::BogusComment => self.bogus_comment(char),
            State::MarkupDeclarationOpen => self.markup_declaration_open(char),
            State::CommentStart => self.comment_start(char),
            State::CommentStartDash => self.comment_start_dash(char),
            State::Comment => self.comment(char),
            State::CommentLessThanSign => self.comment_less_than_sign(char),
            State::CommentLessThanSignBang => self.comment_less_than_sign_bang(char),
            State::CommentLessThanSignBangDash => {
                self.comment_less_than_sign_bang_dash(char)
            }
            State::CommentLessThanSignBangDashDash => {
                self.comment_less_than_sign_bang_dash_dash(char);
            }
            State::CommentEndDash => self.comment_end_dash(char),
            State::CommentEnd => self.comment_end(char),
            State::CommentEndBang => self.comment_end_bang(char),
            State::Doctype => self.doctype(char),
            State::BeforeDoctypeName => self.before_doctype_name(char),
            State::DoctypeName => self.doctype_name(char),
            State::AfterDoctypeName => self.after_doctype_name(char),
            State::AfterDoctypePublicKeyword => self.after_doctype_keyword(char, true),
            State::BeforeDoctypePublicIdentifier => {
                self.before_doctype_identifier(char, true);
            }
            State::DoctypePublicIdentifierDoubleQuoted => {
                self.doctype_identifier_quoted(char, '"', true);
            }
            State::DoctypePublicIdentifierSingleQuoted => {
                self.doctype_identifier_quoted(char, '\'', true);
            }
            State::AfterDoctypePublicIdentifier => {
                self.after_doctype_public_identifier(char)
            }
            State::BetweenDoctypePublicAndSystemIdentifiers => {
                self.between_doctype_public_and_system_identifiers(char);
            }
            State::AfterDoctypeSystemKeyword => self.after_doctype_keyword(char, false),
            State::BeforeDoctypeSystemIdentifier => {
                self.before_doctype_identifier(char, false);
            }
            State::DoctypeSystemIdentifierDoubleQuoted => {
                self.doctype_identifier_quoted(char, '"', false);
            }
            State::DoctypeSystemIdentifierSingleQuoted => {
                self.doctype_identifier_quoted(char, '\'', false);
            }
            State::AfterDoctypeSystemIdentifier => {
                self.after_doctype_system_identifier(char)
            }
            State::BogusDoctype => self.bogus_doctype(char),
            State::CdataSection => self.cdata_section(char),
            State::CdataSectionBracket => self.cdata_section_bracket(char),
            State::CdataSectionEnd => self.cdata_section_end(char),
            State::CharacterReference => self.character_reference(),
            State::AmbiguousAmpersand => self.ambiguous_ampersand(char),
        }
    }

    // Text states

    fn data(&mut self, char: Option<char>) {
        match char {
            Some('<') => self.state = State::TagOpen,
            Some('&') => self.start_character_reference(),
            Some('\0') => {
                self.error(ErrorCode::UnexpectedNullCharacter);
                self.emit_char('\0');
            }
            Some(char) => self.emit_char(char),
            None => self.emit_eof(),
        }
    }

    fn rcdata(&mut self, char: Option<char>) {
        match char {
            Some('&') => self.start_character_reference(),
            Some('<') => self.state = State::RcdataLessThanSign,
            _ => self.text(char),
        }
    }

    fn rawtext(&mut self, char: Option<char>) {
        match char {
            Some('<') => self.state = State::RawtextLessThanSign,
            _ => self.text(char),
        }
    }

    fn script_data(&mut self, char: Option<char>) {
        match char {
            Some('<') => self.state = State::ScriptDataLessThanSign,
            _ => self.text(char),
        }
    }

    fn plaintext(&mut self, char: Option<char>) {
        self.text(char);
    }

    /// What RCDATA, RAWTEXT, script data and PLAINTEXT do with any other character.
    fn text(&mut self, char: Option<char>) {
        match char {
            Some('\0') => {
                self.error(ErrorCode::UnexpectedNullCharacter);
                self.emit_chars("\u{fffd}");
            }
            Some(char) => self.emit_char(char),
            None => self.emit_eof(),
        }
    }

    fn call_text_state(&mut self, state: State, char: Option<char>) {
        self.state = state;
        self.call_state(char);
    }

    // Tags

    fn tag_open(&mut self, char: Option<char>) {
        match char {
            Some(letter) if letter.is_ascii_alphabetic() => {
                self.create_start_tag();
                self.state = State::TagName;
                self.tag_name(char);
            }
            Some('!') => self.state = State::MarkupDeclarationOpen,
            Some('/') => self.state = State::EndTagOpen,
            Some('?') => {
                self.error(ErrorCode::UnexpectedQuestionMarkInsteadOfTagName);
                self.create_comment(1);
                self.state = State::BogusComment;
                self.bogus_comment(char);
            }
            None => {
                self.error(ErrorCode::EofBeforeTagName);
                self.emit_chars("<");
                self.emit_eof();
            }
            Some(_) => {
                self.error(ErrorCode::InvalidFirstCharacterOfTagName);
                self.emit_chars("<");
                self.state = State::Data;
                self.data(char);
            }
        }
    }

    fn end_tag_open(&mut self, char: Option<char>) {
        match char {
            Some(letter) if letter.is_ascii_alphabetic() => {
                self.create_end_tag();
                self.state = State::TagName;
                self.tag_name(char);
            }
            Some('>') => {
                self.error(ErrorCode::MissingEndTagName);
                self.state = State::Data;
            }
            None => {
                self.error(ErrorCode::EofBeforeTagName);
                self.emit_chars("</");
                self.emit_eof();
            }
            Some(_) => {
                self.error(ErrorCode::InvalidFirstCharacterOfTagName);
                self.create_comment(2);
                self.state = State::BogusComment;
                self.bogus_comment(char);
            }
        }
    }

    fn tag_name(&mut self, char: Option<char>) {
        match char {
            Some('\t' | '\n' | '\x0c' | ' ') => self.state = State::BeforeAttributeName,
            Some('/') => self.state = State::SelfClosingStartTag,
            Some('>') => {
                self.state = State::Data;
                self.emit_tag();
            }
            Some('\0') => {
                self.error(ErrorCode::UnexpectedNullCharacter);
                self.tag.tag_name.push('\u{fffd}');
            }
            Some(char) => self.tag.tag_name.push(char.to_ascii_lowercase()),
            None => {
                self.error(ErrorCode::EofInTag);
                self.emit_eof();
            }
        }
    }

    // RCDATA, RAWTEXT and script data end tags

    fn text_less_than_sign(
        &mut self,
        char: Option<char>,
        text: State,
        end_tag_open: State,
    ) {
        if char == Some('/') {
            self.state = end_tag_open;
        } else {
            self.emit_chars("<");
            self.call_text_state(text, char);
        }
    }

    fn text_end_tag_open(
        &mut self,
        char: Option<char>,
        text: State,
        end_tag_name: State,
    ) {
        if char.is_some_and(|char| char.is_ascii_alphabetic()) {
            self.state = end_tag_name;
            self.text_end_tag_name(char, text);
        } else {
            self.emit_chars("</");
            self.call_text_state(text, char);
        }
    }

    fn text_end_tag_name(&mut self, char: Option<char>, text: State) {
        if !self.match_special_end_tag() && !self.preprocessor.chunk_end {
            self.emit_chars("</");
            self.call_text_state(text, char);
        }
    }

    /// parse5's `handleSpecialEndTag`: an end tag for the last start tag ends the text.
    fn match_special_end_tag(&mut self) -> bool {
        let name = self.last_start_tag_name.clone();
        let Some(end) = self.preprocessor.match_sequence(&name, false) else {
            return false;
        };
        self.create_end_tag();
        self.tag.tag_name.clone_from(&name);
        let next = match self.preprocessor.peek_at(end) {
            Some('\t' | '\n' | '\x0c' | ' ') => State::BeforeAttributeName,
            Some('/') => State::SelfClosingStartTag,
            Some('>') => State::Data,
            _ => return false,
        };
        for _ in 0..name.encode_utf16().count() {
            self.consume();
        }
        if next == State::Data {
            self.emit_tag();
        }
        self.state = next;
        true
    }

    // Script data escapes

    fn script_data_less_than_sign(&mut self, char: Option<char>) {
        match char {
            Some('/') => self.state = State::ScriptDataEndTagOpen,
            Some('!') => {
                self.state = State::ScriptDataEscapeStart;
                self.emit_chars("<!");
            }
            _ => {
                self.emit_chars("<");
                self.call_text_state(State::ScriptData, char);
            }
        }
    }

    fn script_data_escape_start(&mut self, char: Option<char>) {
        if char == Some('-') {
            self.state = State::ScriptDataEscapeStartDash;
            self.emit_chars("-");
        } else {
            self.call_text_state(State::ScriptData, char);
        }
    }

    fn script_data_escape_start_dash(&mut self, char: Option<char>) {
        if char == Some('-') {
            self.state = State::ScriptDataEscapedDashDash;
            self.emit_chars("-");
        } else {
            self.call_text_state(State::ScriptData, char);
        }
    }

    fn script_data_escaped(&mut self, char: Option<char>) {
        match char {
            Some('-') => {
                self.state = State::ScriptDataEscapedDash;
                self.emit_chars("-");
            }
            Some('<') => self.state = State::ScriptDataEscapedLessThanSign,
            _ => self.script_escaped_text(char),
        }
    }

    fn script_data_escaped_dash(&mut self, char: Option<char>) {
        match char {
            Some('-') => {
                self.state = State::ScriptDataEscapedDashDash;
                self.emit_chars("-");
            }
            Some('<') => self.state = State::ScriptDataEscapedLessThanSign,
            _ => {
                if char.is_some() {
                    self.state = State::ScriptDataEscaped;
                }
                self.script_escaped_text(char);
            }
        }
    }

    fn script_data_escaped_dash_dash(&mut self, char: Option<char>) {
        match char {
            Some('-') => self.emit_chars("-"),
            Some('<') => self.state = State::ScriptDataEscapedLessThanSign,
            Some('>') => {
                self.state = State::ScriptData;
                self.emit_chars(">");
            }
            _ => {
                if char.is_some() {
                    self.state = State::ScriptDataEscaped;
                }
                self.script_escaped_text(char);
            }
        }
    }

    /// What the escaped and double-escaped script states do with any other character.
    fn script_escaped_text(&mut self, char: Option<char>) {
        match char {
            Some('\0') => {
                self.error(ErrorCode::UnexpectedNullCharacter);
                self.emit_chars("\u{fffd}");
            }
            Some(char) => self.emit_char(char),
            None => {
                self.error(ErrorCode::EofInScriptHtmlCommentLikeText);
                self.emit_eof();
            }
        }
    }

    fn script_data_escaped_less_than_sign(&mut self, char: Option<char>) {
        match char {
            Some('/') => self.state = State::ScriptDataEscapedEndTagOpen,
            Some(letter) if letter.is_ascii_alphabetic() => {
                self.emit_chars("<");
                self.state = State::ScriptDataDoubleEscapeStart;
                self.script_data_double_escape_start(char);
            }
            _ => {
                self.emit_chars("<");
                self.call_text_state(State::ScriptDataEscaped, char);
            }
        }
    }

    /// Emits `script` and the character after it when the input continues with them.
    fn consume_script_sequence(&mut self, char: Option<char>) -> bool {
        let end = self.preprocessor.match_sequence("script", false);
        let ends = end
            .and_then(|end| self.preprocessor.peek_at(end))
            .is_some_and(|next| matches!(next, '\t' | '\n' | '\x0c' | ' ' | '/' | '>'));
        if !ends {
            return false;
        }
        if let Some(char) = char {
            self.emit_char(char);
        }
        for _ in 0.."script".len() {
            if let Some(char) = self.consume() {
                self.emit_char(char);
            }
        }
        true
    }

    fn script_data_double_escape_start(&mut self, char: Option<char>) {
        if self.consume_script_sequence(char) {
            self.state = State::ScriptDataDoubleEscaped;
        } else if !self.preprocessor.chunk_end {
            self.call_text_state(State::ScriptDataEscaped, char);
        }
    }

    fn script_data_double_escaped(&mut self, char: Option<char>) {
        match char {
            Some('-') => {
                self.state = State::ScriptDataDoubleEscapedDash;
                self.emit_chars("-");
            }
            Some('<') => {
                self.state = State::ScriptDataDoubleEscapedLessThanSign;
                self.emit_chars("<");
            }
            _ => self.script_escaped_text(char),
        }
    }

    fn script_data_double_escaped_dash(&mut self, char: Option<char>) {
        match char {
            Some('-') => {
                self.state = State::ScriptDataDoubleEscapedDashDash;
                self.emit_chars("-");
            }
            Some('<') => {
                self.state = State::ScriptDataDoubleEscapedLessThanSign;
                self.emit_chars("<");
            }
            _ => {
                if char.is_some() {
                    self.state = State::ScriptDataDoubleEscaped;
                }
                self.script_escaped_text(char);
            }
        }
    }

    fn script_data_double_escaped_dash_dash(&mut self, char: Option<char>) {
        match char {
            Some('-') => self.emit_chars("-"),
            Some('<') => {
                self.state = State::ScriptDataDoubleEscapedLessThanSign;
                self.emit_chars("<");
            }
            Some('>') => {
                self.state = State::ScriptData;
                self.emit_chars(">");
            }
            _ => {
                if char.is_some() {
                    self.state = State::ScriptDataDoubleEscaped;
                }
                self.script_escaped_text(char);
            }
        }
    }

    fn script_data_double_escaped_less_than_sign(&mut self, char: Option<char>) {
        if char == Some('/') {
            self.state = State::ScriptDataDoubleEscapeEnd;
            self.emit_chars("/");
        } else {
            self.call_text_state(State::ScriptDataDoubleEscaped, char);
        }
    }

    fn script_data_double_escape_end(&mut self, char: Option<char>) {
        if self.consume_script_sequence(char) {
            self.state = State::ScriptDataEscaped;
        } else if !self.preprocessor.chunk_end {
            self.call_text_state(State::ScriptDataDoubleEscaped, char);
        }
    }

    // Attributes

    fn before_attribute_name(&mut self, char: Option<char>) {
        match char {
            Some('\t' | '\n' | '\x0c' | ' ') => {}
            Some('/' | '>') | None => {
                self.state = State::AfterAttributeName;
                self.after_attribute_name(char);
            }
            Some('=') => {
                self.error(ErrorCode::UnexpectedEqualsSignBeforeAttributeName);
                self.create_attribute("=");
                self.state = State::AttributeName;
            }
            Some(_) => {
                self.create_attribute("");
                self.state = State::AttributeName;
                self.attribute_name(char);
            }
        }
    }

    fn attribute_name(&mut self, char: Option<char>) {
        match char {
            Some('\t' | '\n' | '\x0c' | ' ' | '/' | '>') | None => {
                self.leave_attribute_name();
                self.state = State::AfterAttributeName;
                self.after_attribute_name(char);
            }
            Some('=') => {
                self.leave_attribute_name();
                self.state = State::BeforeAttributeValue;
            }
            Some(char @ ('"' | '\'' | '<')) => {
                self.error(ErrorCode::UnexpectedCharacterInAttributeName);
                self.attribute_name.push(char);
            }
            Some('\0') => {
                self.error(ErrorCode::UnexpectedNullCharacter);
                self.attribute_name.push('\u{fffd}');
            }
            Some(char) => self.attribute_name.push(char.to_ascii_lowercase()),
        }
    }

    fn after_attribute_name(&mut self, char: Option<char>) {
        match char {
            Some('\t' | '\n' | '\x0c' | ' ') => {}
            Some('/') => self.state = State::SelfClosingStartTag,
            Some('=') => self.state = State::BeforeAttributeValue,
            Some('>') => {
                self.state = State::Data;
                self.emit_tag();
            }
            None => {
                self.error(ErrorCode::EofInTag);
                self.emit_eof();
            }
            Some(_) => {
                self.create_attribute("");
                self.state = State::AttributeName;
                self.attribute_name(char);
            }
        }
    }

    fn before_attribute_value(&mut self, char: Option<char>) {
        match char {
            Some('\t' | '\n' | '\x0c' | ' ') => {}
            Some('"') => self.state = State::AttributeValueDoubleQuoted,
            Some('\'') => self.state = State::AttributeValueSingleQuoted,
            Some('>') => {
                self.error(ErrorCode::MissingAttributeValue);
                self.state = State::Data;
                self.emit_tag();
            }
            _ => {
                self.state = State::AttributeValueUnquoted;
                self.attribute_value_unquoted(char);
            }
        }
    }

    fn attribute_value_quoted(&mut self, char: Option<char>, quote: char) {
        match char {
            Some(char) if char == quote => self.state = State::AfterAttributeValueQuoted,
            Some('&') => self.start_character_reference(),
            Some('\0') => {
                self.error(ErrorCode::UnexpectedNullCharacter);
                self.push_attribute_value('\u{fffd}');
            }
            Some(char) => self.push_attribute_value(char),
            None => {
                self.error(ErrorCode::EofInTag);
                self.emit_eof();
            }
        }
    }

    fn attribute_value_unquoted(&mut self, char: Option<char>) {
        match char {
            Some('\t' | '\n' | '\x0c' | ' ') => {
                self.leave_attribute_value();
                self.state = State::BeforeAttributeName;
            }
            Some('&') => self.start_character_reference(),
            Some('>') => {
                self.leave_attribute_value();
                self.state = State::Data;
                self.emit_tag();
            }
            Some('\0') => {
                self.error(ErrorCode::UnexpectedNullCharacter);
                self.push_attribute_value('\u{fffd}');
            }
            Some(char @ ('"' | '\'' | '<' | '=' | '`')) => {
                self.error(ErrorCode::UnexpectedCharacterInUnquotedAttributeValue);
                self.push_attribute_value(char);
            }
            Some(char) => self.push_attribute_value(char),
            None => {
                self.error(ErrorCode::EofInTag);
                self.emit_eof();
            }
        }
    }

    fn after_attribute_value_quoted(&mut self, char: Option<char>) {
        match char {
            Some('\t' | '\n' | '\x0c' | ' ') => {
                self.leave_attribute_value();
                self.state = State::BeforeAttributeName;
            }
            Some('/') => {
                self.leave_attribute_value();
                self.state = State::SelfClosingStartTag;
            }
            Some('>') => {
                self.leave_attribute_value();
                self.state = State::Data;
                self.emit_tag();
            }
            None => {
                self.error(ErrorCode::EofInTag);
                self.emit_eof();
            }
            Some(_) => {
                self.error(ErrorCode::MissingWhitespaceBetweenAttributes);
                self.state = State::BeforeAttributeName;
                self.before_attribute_name(char);
            }
        }
    }

    fn self_closing_start_tag(&mut self, char: Option<char>) {
        match char {
            Some('>') => {
                self.tag.self_closing = true;
                self.state = State::Data;
                self.emit_tag();
            }
            None => {
                self.error(ErrorCode::EofInTag);
                self.emit_eof();
            }
            Some(_) => {
                self.error(ErrorCode::UnexpectedSolidusInTag);
                self.state = State::BeforeAttributeName;
                self.before_attribute_name(char);
            }
        }
    }

    // Comments

    fn bogus_comment(&mut self, char: Option<char>) {
        match char {
            Some('>') => {
                self.state = State::Data;
                self.emit_comment();
            }
            None => {
                self.emit_comment();
                self.emit_eof();
            }
            Some('\0') => {
                self.error(ErrorCode::UnexpectedNullCharacter);
                self.comment.data.push('\u{fffd}');
            }
            Some(char) => self.comment.data.push(char),
        }
    }

    fn markup_declaration_open(&mut self, char: Option<char>) {
        if self.consume_sequence("--", true) {
            self.create_comment(3);
            self.state = State::CommentStart;
        } else if self.consume_sequence("doctype", false) {
            self.location = self.pos().saturating_sub(8);
            self.located = true;
            self.state = State::Doctype;
        } else if self.consume_sequence("[CDATA[", true) {
            if self.in_foreign_node {
                self.state = State::CdataSection;
            } else {
                self.error(ErrorCode::CdataInHtmlContent);
                self.create_comment(8);
                self.comment.data.push_str("[CDATA[");
                self.state = State::BogusComment;
            }
        } else if !self.preprocessor.chunk_end {
            self.error(ErrorCode::IncorrectlyOpenedComment);
            self.create_comment(2);
            self.state = State::BogusComment;
            self.bogus_comment(char);
        }
    }

    fn comment_start(&mut self, char: Option<char>) {
        match char {
            Some('-') => self.state = State::CommentStartDash,
            Some('>') => {
                self.error(ErrorCode::AbruptClosingOfEmptyComment);
                self.state = State::Data;
                self.emit_comment();
            }
            _ => {
                self.state = State::Comment;
                self.comment(char);
            }
        }
    }

    fn comment_start_dash(&mut self, char: Option<char>) {
        match char {
            Some('-') => self.state = State::CommentEnd,
            Some('>') => {
                self.error(ErrorCode::AbruptClosingOfEmptyComment);
                self.state = State::Data;
                self.emit_comment();
            }
            None => self.eof_in_comment(),
            Some(_) => {
                self.comment.data.push('-');
                self.state = State::Comment;
                self.comment(char);
            }
        }
    }

    fn eof_in_comment(&mut self) {
        self.error(ErrorCode::EofInComment);
        self.emit_comment();
        self.emit_eof();
    }

    fn comment(&mut self, char: Option<char>) {
        match char {
            Some('-') => self.state = State::CommentEndDash,
            Some('<') => {
                self.comment.data.push('<');
                self.state = State::CommentLessThanSign;
            }
            Some('\0') => {
                self.error(ErrorCode::UnexpectedNullCharacter);
                self.comment.data.push('\u{fffd}');
            }
            Some(char) => self.comment.data.push(char),
            None => self.eof_in_comment(),
        }
    }

    fn comment_less_than_sign(&mut self, char: Option<char>) {
        match char {
            Some('!') => {
                self.comment.data.push('!');
                self.state = State::CommentLessThanSignBang;
            }
            Some('<') => self.comment.data.push('<'),
            _ => {
                self.state = State::Comment;
                self.comment(char);
            }
        }
    }

    fn comment_less_than_sign_bang(&mut self, char: Option<char>) {
        if char == Some('-') {
            self.state = State::CommentLessThanSignBangDash;
        } else {
            self.state = State::Comment;
            self.comment(char);
        }
    }

    fn comment_less_than_sign_bang_dash(&mut self, char: Option<char>) {
        if char == Some('-') {
            self.state = State::CommentLessThanSignBangDashDash;
        } else {
            self.state = State::CommentEndDash;
            self.comment_end_dash(char);
        }
    }

    fn comment_less_than_sign_bang_dash_dash(&mut self, char: Option<char>) {
        if char.is_some_and(|char| char != '>') {
            self.error(ErrorCode::NestedComment);
        }
        self.state = State::CommentEnd;
        self.comment_end(char);
    }

    fn comment_end_dash(&mut self, char: Option<char>) {
        match char {
            Some('-') => self.state = State::CommentEnd,
            None => self.eof_in_comment(),
            Some(_) => {
                self.comment.data.push('-');
                self.state = State::Comment;
                self.comment(char);
            }
        }
    }

    fn comment_end(&mut self, char: Option<char>) {
        match char {
            Some('>') => {
                self.state = State::Data;
                self.emit_comment();
            }
            Some('!') => self.state = State::CommentEndBang,
            Some('-') => self.comment.data.push('-'),
            None => self.eof_in_comment(),
            Some(_) => {
                self.comment.data.push_str("--");
                self.state = State::Comment;
                self.comment(char);
            }
        }
    }

    fn comment_end_bang(&mut self, char: Option<char>) {
        match char {
            Some('-') => {
                self.comment.data.push_str("--!");
                self.state = State::CommentEndDash;
            }
            Some('>') => {
                self.error(ErrorCode::IncorrectlyClosedComment);
                self.state = State::Data;
                self.emit_comment();
            }
            None => self.eof_in_comment(),
            Some(_) => {
                self.comment.data.push_str("--!");
                self.state = State::Comment;
                self.comment(char);
            }
        }
    }

    // Doctypes

    fn eof_in_doctype(&mut self) {
        self.error(ErrorCode::EofInDoctype);
        self.doctype.force_quirks = true;
        self.emit_doctype();
        self.emit_eof();
    }

    fn doctype(&mut self, char: Option<char>) {
        match char {
            Some('\t' | '\n' | '\x0c' | ' ') => self.state = State::BeforeDoctypeName,
            Some('>') => {
                self.state = State::BeforeDoctypeName;
                self.before_doctype_name(char);
            }
            None => {
                self.create_doctype(None);
                self.eof_in_doctype();
            }
            Some(_) => {
                self.error(ErrorCode::MissingWhitespaceBeforeDoctypeName);
                self.state = State::BeforeDoctypeName;
                self.before_doctype_name(char);
            }
        }
    }

    fn before_doctype_name(&mut self, char: Option<char>) {
        match char {
            Some('\t' | '\n' | '\x0c' | ' ') => {}
            Some('\0') => {
                self.error(ErrorCode::UnexpectedNullCharacter);
                self.create_doctype(Some("\u{fffd}".to_owned()));
                self.state = State::DoctypeName;
            }
            Some('>') => {
                self.error(ErrorCode::MissingDoctypeName);
                self.create_doctype(None);
                self.doctype.force_quirks = true;
                self.emit_doctype();
                self.state = State::Data;
            }
            None => {
                self.create_doctype(None);
                self.eof_in_doctype();
            }
            Some(char) => {
                self.create_doctype(Some(char.to_ascii_lowercase().to_string()));
                self.state = State::DoctypeName;
            }
        }
    }

    fn doctype_name(&mut self, char: Option<char>) {
        match char {
            Some('\t' | '\n' | '\x0c' | ' ') => self.state = State::AfterDoctypeName,
            Some('>') => {
                self.state = State::Data;
                self.emit_doctype();
            }
            Some('\0') => {
                self.error(ErrorCode::UnexpectedNullCharacter);
                self.doctype.name.get_or_insert_default().push('\u{fffd}');
            }
            Some(char) => {
                self.doctype
                    .name
                    .get_or_insert_default()
                    .push(char.to_ascii_lowercase());
            }
            None => self.eof_in_doctype(),
        }
    }

    fn after_doctype_name(&mut self, char: Option<char>) {
        match char {
            Some('\t' | '\n' | '\x0c' | ' ') => {}
            Some('>') => {
                self.state = State::Data;
                self.emit_doctype();
            }
            None => self.eof_in_doctype(),
            Some(_) => {
                if self.consume_sequence("public", false) {
                    self.state = State::AfterDoctypePublicKeyword;
                } else if self.consume_sequence("system", false) {
                    self.state = State::AfterDoctypeSystemKeyword;
                } else if !self.preprocessor.chunk_end {
                    self.error(ErrorCode::InvalidCharacterSequenceAfterDoctypeName);
                    self.doctype.force_quirks = true;
                    self.state = State::BogusDoctype;
                    self.bogus_doctype(char);
                }
            }
        }
    }

    fn start_doctype_identifier(&mut self, public: bool, quote: char) {
        let (identifier, state) = match (public, quote) {
            (true, '"') => (
                &mut self.doctype.public_id,
                State::DoctypePublicIdentifierDoubleQuoted,
            ),
            (true, _) => (
                &mut self.doctype.public_id,
                State::DoctypePublicIdentifierSingleQuoted,
            ),
            (false, '"') => (
                &mut self.doctype.system_id,
                State::DoctypeSystemIdentifierDoubleQuoted,
            ),
            (false, _) => (
                &mut self.doctype.system_id,
                State::DoctypeSystemIdentifierSingleQuoted,
            ),
        };
        *identifier = Some(String::new());
        self.state = state;
    }

    fn missing_doctype_quote(&mut self, char: Option<char>, public: bool) {
        self.error(if public {
            ErrorCode::MissingQuoteBeforeDoctypePublicIdentifier
        } else {
            ErrorCode::MissingQuoteBeforeDoctypeSystemIdentifier
        });
        self.doctype.force_quirks = true;
        self.state = State::BogusDoctype;
        self.bogus_doctype(char);
    }

    fn missing_doctype_identifier(&mut self, public: bool) {
        self.error(if public {
            ErrorCode::MissingDoctypePublicIdentifier
        } else {
            ErrorCode::MissingDoctypeSystemIdentifier
        });
        self.doctype.force_quirks = true;
        self.state = State::Data;
        self.emit_doctype();
    }

    fn after_doctype_keyword(&mut self, char: Option<char>, public: bool) {
        match char {
            Some('\t' | '\n' | '\x0c' | ' ') => {
                self.state = if public {
                    State::BeforeDoctypePublicIdentifier
                } else {
                    State::BeforeDoctypeSystemIdentifier
                };
            }
            Some(quote @ ('"' | '\'')) => {
                self.error(if public {
                    ErrorCode::MissingWhitespaceAfterDoctypePublicKeyword
                } else {
                    ErrorCode::MissingWhitespaceAfterDoctypeSystemKeyword
                });
                self.start_doctype_identifier(public, quote);
            }
            Some('>') => self.missing_doctype_identifier(public),
            None => self.eof_in_doctype(),
            Some(_) => self.missing_doctype_quote(char, public),
        }
    }

    fn before_doctype_identifier(&mut self, char: Option<char>, public: bool) {
        match char {
            Some('\t' | '\n' | '\x0c' | ' ') => {}
            Some(quote @ ('"' | '\'')) => self.start_doctype_identifier(public, quote),
            Some('>') => self.missing_doctype_identifier(public),
            None => self.eof_in_doctype(),
            Some(_) => self.missing_doctype_quote(char, public),
        }
    }

    fn doctype_identifier(&mut self, public: bool) -> &mut String {
        let identifier = if public {
            &mut self.doctype.public_id
        } else {
            &mut self.doctype.system_id
        };
        identifier.get_or_insert_default()
    }

    fn doctype_identifier_quoted(
        &mut self,
        char: Option<char>,
        quote: char,
        public: bool,
    ) {
        match char {
            Some(char) if char == quote => {
                self.state = if public {
                    State::AfterDoctypePublicIdentifier
                } else {
                    State::AfterDoctypeSystemIdentifier
                };
            }
            Some('\0') => {
                self.error(ErrorCode::UnexpectedNullCharacter);
                self.doctype_identifier(public).push('\u{fffd}');
            }
            Some('>') => {
                self.error(if public {
                    ErrorCode::AbruptDoctypePublicIdentifier
                } else {
                    ErrorCode::AbruptDoctypeSystemIdentifier
                });
                self.doctype.force_quirks = true;
                self.emit_doctype();
                self.state = State::Data;
            }
            None => self.eof_in_doctype(),
            Some(char) => self.doctype_identifier(public).push(char),
        }
    }

    fn after_doctype_public_identifier(&mut self, char: Option<char>) {
        match char {
            Some('\t' | '\n' | '\x0c' | ' ') => {
                self.state = State::BetweenDoctypePublicAndSystemIdentifiers;
            }
            Some('>') => {
                self.state = State::Data;
                self.emit_doctype();
            }
            Some(quote @ ('"' | '\'')) => {
                self.error(
                    ErrorCode::MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers,
                );
                self.start_doctype_identifier(false, quote);
            }
            None => self.eof_in_doctype(),
            Some(_) => self.missing_doctype_quote(char, false),
        }
    }

    fn between_doctype_public_and_system_identifiers(&mut self, char: Option<char>) {
        match char {
            Some('\t' | '\n' | '\x0c' | ' ') => {}
            Some('>') => {
                self.emit_doctype();
                self.state = State::Data;
            }
            Some(quote @ ('"' | '\'')) => self.start_doctype_identifier(false, quote),
            None => self.eof_in_doctype(),
            Some(_) => self.missing_doctype_quote(char, false),
        }
    }

    fn after_doctype_system_identifier(&mut self, char: Option<char>) {
        match char {
            Some('\t' | '\n' | '\x0c' | ' ') => {}
            Some('>') => {
                self.emit_doctype();
                self.state = State::Data;
            }
            None => self.eof_in_doctype(),
            Some(_) => {
                self.error(ErrorCode::UnexpectedCharacterAfterDoctypeSystemIdentifier);
                self.state = State::BogusDoctype;
                self.bogus_doctype(char);
            }
        }
    }

    fn bogus_doctype(&mut self, char: Option<char>) {
        match char {
            Some('>') => {
                self.emit_doctype();
                self.state = State::Data;
            }
            Some('\0') => self.error(ErrorCode::UnexpectedNullCharacter),
            Some(_) => {}
            None => {
                self.emit_doctype();
                self.emit_eof();
            }
        }
    }

    // CDATA sections

    fn cdata_section(&mut self, char: Option<char>) {
        match char {
            Some(']') => self.state = State::CdataSectionBracket,
            Some(char) => self.emit_char(char),
            None => {
                self.error(ErrorCode::EofInCdata);
                self.emit_eof();
            }
        }
    }

    fn cdata_section_bracket(&mut self, char: Option<char>) {
        if char == Some(']') {
            self.state = State::CdataSectionEnd;
        } else {
            self.emit_chars("]");
            self.state = State::CdataSection;
            self.cdata_section(char);
        }
    }

    fn cdata_section_end(&mut self, char: Option<char>) {
        match char {
            Some('>') => self.state = State::Data,
            Some(']') => self.emit_chars("]"),
            _ => {
                self.emit_chars("]]");
                self.state = State::CdataSection;
                self.cdata_section(char);
            }
        }
    }

    // Character reference states

    /// Decodes the reference from the raw input after its `&`, as `entities` does, then
    /// rewinds to its last character, or to the `&` when it is none.
    fn character_reference(&mut self) {
        let start = self.reference_start;
        let mode = if self.is_reference_in_attribute() {
            DecodeMode::Attribute
        } else {
            DecodeMode::Legacy
        };
        let rest = self.preprocessor.input.get(start + 1..).unwrap_or_default();
        // parse5 waits for the rest of a reference a chunk ends in, then drops it
        if !self.preprocessor.last_chunk && may_continue_reference(rest) {
            self.preprocessor.chunk_end = true;
            return;
        }
        let reference = decode_reference(rest, mode);
        if reference.consumed > 0 {
            self.preprocessor.set_pos(start + reference.consumed - 1);
            for char in reference.chars.into_iter().flatten() {
                self.flush_reference_char(char);
            }
        }
        for (code, offset) in reference.errors.into_iter().flatten() {
            self.error_at(code, start + offset);
        }
        if reference.consumed > 0 {
            self.state = self.return_state;
            return;
        }
        self.preprocessor.set_pos(start);
        self.flush_reference_char('&');
        let ambiguous = !self.is_reference_in_attribute()
            && self
                .preprocessor
                .peek_at(start + 1)
                .is_some_and(|char| char.is_ascii_alphanumeric());
        self.state = if ambiguous {
            State::AmbiguousAmpersand
        } else {
            self.return_state
        };
    }

    fn ambiguous_ampersand(&mut self, char: Option<char>) {
        match char {
            Some(char) if char.is_ascii_alphanumeric() => self.flush_reference_char(char),
            _ => {
                if char == Some(';') {
                    self.error(ErrorCode::UnknownNamedCharacterReference);
                }
                self.state = self.return_state;
                self.call_state(char);
            }
        }
    }
}

// Whether a reference running to the end of a chunk could still go on in the next one
fn may_continue_reference(rest: &str) -> bool {
    rest.bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'#')
}

fn create_tag(start: usize) -> TagToken {
    TagToken {
        tag_name: String::new(),
        tag_id: TagId::Unknown,
        self_closing: false,
        ack_self_closing: false,
        attrs: Vec::new(),
        span: Span { start, end: 0 },
    }
}

fn create_doctype(name: Option<String>, start: usize) -> DoctypeToken {
    DoctypeToken {
        name,
        force_quirks: false,
        public_id: None,
        system_id: None,
        span: Span { start, end: 0 },
    }
}

#[cfg(test)]
#[path = "machine_unit.rs"]
mod unit;
