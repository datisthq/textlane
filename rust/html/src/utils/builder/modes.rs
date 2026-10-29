use crate::helpers::doctype::{find_document_mode, is_conforming};
use crate::helpers::foreign::{
    adjust_mathml_attributes, adjust_svg_attributes, adjust_svg_tag_name,
    adjust_xml_attributes, causes_exit,
};
use crate::helpers::tag::TagId;
use crate::models::namespace::Namespace;
use crate::models::parse_error::ErrorCode;
use crate::models::token::{CharacterKind, Span, TagToken, Token};
use crate::utils::builder::dom::{DocumentMode, NodeId};
use crate::utils::builder::parser::{
    Closing, Mode, NUMBERED_HEADERS, Parser, TABLE_STRUCTURE,
};
use crate::utils::tokenizer::machine::State;

// Adoption agency loop limits
const OUTER_LOOP_LIMIT: usize = 8;
const INNER_LOOP_LIMIT: usize = 3;

const TABLE_VOID: [TagId; 9] = [
    TagId::Caption,
    TagId::Col,
    TagId::Colgroup,
    TagId::Tbody,
    TagId::Td,
    TagId::Tfoot,
    TagId::Th,
    TagId::Thead,
    TagId::Tr,
];

fn span_of(token: &Token) -> Span {
    match token {
        Token::Character(token) => token.span,
        Token::StartTag(token) | Token::EndTag(token) => token.span,
        Token::Comment(token) => token.span,
        Token::Doctype(token) => token.span,
        Token::Eof(span) => *span,
    }
}

fn tag_of(token: &Token) -> Option<&TagToken> {
    match token {
        Token::StartTag(token) | Token::EndTag(token) => Some(token),
        _ => None,
    }
}

fn id(token: &Token) -> TagId {
    tag_of(token).map_or(TagId::Unknown, |token| token.tag_id)
}

fn ack(token: &mut Token) {
    if let Token::StartTag(token) = token {
        token.ack_self_closing = true;
    }
}

fn insert(p: &mut Parser, token: &Token, namespace: Namespace) {
    if let Some(tag) = tag_of(token) {
        p.insert_element(tag, namespace);
    }
}

fn append(p: &mut Parser, token: &Token, namespace: Namespace) {
    if let Some(tag) = tag_of(token) {
        p.append_element(tag, namespace);
    }
}

fn err(p: &mut Parser, token: &Token, code: ErrorCode) {
    p.err(span_of(token), code, false);
}

fn is_hidden_input(token: &Token) -> bool {
    tag_of(token)
        .and_then(|tag| tag.find_attribute("type"))
        .is_some_and(|value| value.eq_ignore_ascii_case("hidden"))
}

impl Parser<'_> {
    /// Feed one token to the tree builder, as parse5's `on*` callbacks.
    pub(crate) fn dispatch(&mut self, token: Token) {
        let mut token = token;
        match &token {
            Token::Character(character) => match character.kind {
                CharacterKind::Character => on_character(self, &mut token),
                CharacterKind::Null => on_null_character(self, &mut token),
                CharacterKind::Whitespace => on_whitespace_character(self, &mut token),
            },
            Token::Comment(_) => on_comment(self, &mut token),
            Token::Doctype(_) => on_doctype(self, &mut token),
            Token::StartTag(_) => on_start_tag(self, &mut token),
            Token::EndTag(_) => on_end_tag(self, &mut token),
            Token::Eof(_) => on_eof(self, &mut token),
        }
    }

    /// Feed one token as parse5's `_processToken` does, for tokens no tokenizer read.
    pub(crate) fn process(&mut self, token: Token) {
        let mut token = token;
        process_token(self, &mut token);
    }
}

fn process_token(p: &mut Parser, token: &mut Token) {
    match token {
        Token::Character(character) => match character.kind {
            CharacterKind::Character => on_character(p, token),
            CharacterKind::Null => on_null_character(p, token),
            CharacterKind::Whitespace => on_whitespace_character(p, token),
        },
        Token::Comment(_) => on_comment(p, token),
        Token::Doctype(_) => on_doctype(p, token),
        Token::StartTag(_) => process_start_tag(p, token),
        Token::EndTag(_) => on_end_tag(p, token),
        Token::Eof(_) => on_eof(p, token),
    }
}

fn on_character(p: &mut Parser, token: &mut Token) {
    p.skip_next_new_line = false;
    if p.tokenizer.in_foreign_node {
        character_in_foreign_content(p, token);
        return;
    }
    match p.mode {
        Mode::Initial => token_in_initial_mode(p, token),
        Mode::BeforeHtml => token_before_html(p, token),
        Mode::BeforeHead => token_before_head(p, token),
        Mode::InHead => token_in_head(p, token),
        Mode::InHeadNoScript => token_in_head_no_script(p, token),
        Mode::AfterHead => token_after_head(p, token),
        Mode::InBody | Mode::InCaption | Mode::InCell | Mode::InTemplate => {
            character_in_body(p, token)
        }
        Mode::Text | Mode::InSelect | Mode::InSelectInTable => {
            insert_characters(p, token)
        }
        Mode::InTable | Mode::InTableBody | Mode::InRow => character_in_table(p, token),
        Mode::InTableText => character_in_table_text(p, token),
        Mode::InColumnGroup => token_in_column_group(p, token),
        Mode::AfterBody => token_after_body(p, token),
        Mode::AfterAfterBody => token_after_after_body(p, token),
        _ => {}
    }
}

fn on_null_character(p: &mut Parser, token: &mut Token) {
    p.skip_next_new_line = false;
    if p.tokenizer.in_foreign_node {
        null_character_in_foreign_content(p, token);
        return;
    }
    match p.mode {
        Mode::Initial => token_in_initial_mode(p, token),
        Mode::BeforeHtml => token_before_html(p, token),
        Mode::BeforeHead => token_before_head(p, token),
        Mode::InHead => token_in_head(p, token),
        Mode::InHeadNoScript => token_in_head_no_script(p, token),
        Mode::AfterHead => token_after_head(p, token),
        Mode::Text => insert_characters(p, token),
        Mode::InTable | Mode::InTableBody | Mode::InRow => character_in_table(p, token),
        Mode::InColumnGroup => token_in_column_group(p, token),
        Mode::AfterBody => token_after_body(p, token),
        Mode::AfterAfterBody => token_after_after_body(p, token),
        _ => {}
    }
}

fn on_comment(p: &mut Parser, token: &mut Token) {
    p.skip_next_new_line = false;
    if p.current_not_in_html {
        append_comment(p, token);
        return;
    }
    match p.mode {
        Mode::InTableText => token_in_table_text(p, token),
        Mode::AfterBody => {
            let root = p.stack.first().copied().unwrap_or(p.document);
            append_comment_to(p, token, root);
        }
        Mode::AfterAfterBody | Mode::AfterAfterFrameset => {
            let document = p.document;
            append_comment_to(p, token, document);
        }
        Mode::Text => {}
        _ => append_comment(p, token),
    }
}

fn on_doctype(p: &mut Parser, token: &mut Token) {
    p.skip_next_new_line = false;
    match p.mode {
        Mode::Initial => doctype_in_initial_mode(p, token),
        Mode::BeforeHead | Mode::InHead | Mode::InHeadNoScript | Mode::AfterHead => {
            err(p, token, ErrorCode::MisplacedDoctype)
        }
        Mode::InTableText => token_in_table_text(p, token),
        _ => {}
    }
}

fn on_start_tag(p: &mut Parser, token: &mut Token) {
    p.skip_next_new_line = false;
    p.closing = Some(Closing {
        end_tag_name: None,
        span: span_of(token),
    });
    process_start_tag(p, token);
    if let Token::StartTag(tag) = token
        && tag.self_closing
        && !tag.ack_self_closing
    {
        let span = tag.span;
        p.err(
            span,
            ErrorCode::NonVoidHtmlElementStartTagWithTrailingSolidus,
            false,
        );
    }
}

fn should_process_start_tag_in_foreign_content(p: &Parser, token: &Token) -> bool {
    if !p.current_not_in_html {
        return false;
    }
    let (current, current_tag) = if p.stack.len() == 1 && p.fragment_context.is_some() {
        (p.fragment_context, Some(p.fragment_context_id))
    } else {
        (p.current(), p.current_tag())
    };
    let tag = id(token);
    if tag == TagId::Svg
        && let Some(current) = current
        && p.dom.tag_name(current) == "annotation-xml"
        && p.dom.namespace(current) == Namespace::MathMl
    {
        return false;
    }
    p.tokenizer.in_foreign_node
        || (matches!(tag, TagId::Mglyph | TagId::Malignmark)
            && match (current_tag, current) {
                (Some(current_tag), Some(current)) => {
                    !p.is_integration_point(current_tag, current, Some(Namespace::Html))
                }
                _ => false,
            })
}

fn process_start_tag(p: &mut Parser, token: &mut Token) {
    if should_process_start_tag_in_foreign_content(p, token) {
        start_tag_in_foreign_content(p, token);
    } else {
        start_tag_outside_foreign_content(p, token);
    }
}

fn start_tag_outside_foreign_content(p: &mut Parser, token: &mut Token) {
    match p.mode {
        Mode::Initial => token_in_initial_mode(p, token),
        Mode::BeforeHtml => start_tag_before_html(p, token),
        Mode::BeforeHead => start_tag_before_head(p, token),
        Mode::InHead => start_tag_in_head(p, token),
        Mode::InHeadNoScript => start_tag_in_head_no_script(p, token),
        Mode::AfterHead => start_tag_after_head(p, token),
        Mode::InBody => start_tag_in_body(p, token),
        Mode::InTable => start_tag_in_table(p, token),
        Mode::InTableText => token_in_table_text(p, token),
        Mode::InCaption => start_tag_in_caption(p, token),
        Mode::InColumnGroup => start_tag_in_column_group(p, token),
        Mode::InTableBody => start_tag_in_table_body(p, token),
        Mode::InRow => start_tag_in_row(p, token),
        Mode::InCell => start_tag_in_cell(p, token),
        Mode::InSelect => start_tag_in_select(p, token),
        Mode::InSelectInTable => start_tag_in_select_in_table(p, token),
        Mode::InTemplate => start_tag_in_template(p, token),
        Mode::AfterBody => start_tag_after_body(p, token),
        Mode::InFrameset => start_tag_in_frameset(p, token),
        Mode::AfterFrameset => start_tag_after_frameset(p, token),
        Mode::AfterAfterBody => start_tag_after_after_body(p, token),
        Mode::AfterAfterFrameset => start_tag_after_after_frameset(p, token),
        Mode::Text => {}
    }
}

fn on_end_tag(p: &mut Parser, token: &mut Token) {
    p.skip_next_new_line = false;
    let name = tag_of(token).map(|tag| tag.tag_name.clone());
    p.closing = Some(Closing {
        end_tag_name: name,
        span: span_of(token),
    });
    if p.current_not_in_html {
        end_tag_in_foreign_content(p, token);
    } else {
        end_tag_outside_foreign_content(p, token);
    }
}

fn end_tag_outside_foreign_content(p: &mut Parser, token: &mut Token) {
    match p.mode {
        Mode::Initial => token_in_initial_mode(p, token),
        Mode::BeforeHtml => end_tag_before_html(p, token),
        Mode::BeforeHead => end_tag_before_head(p, token),
        Mode::InHead => end_tag_in_head(p, token),
        Mode::InHeadNoScript => end_tag_in_head_no_script(p, token),
        Mode::AfterHead => end_tag_after_head(p, token),
        Mode::InBody => end_tag_in_body(p, token),
        Mode::Text => end_tag_in_text(p),
        Mode::InTable => end_tag_in_table(p, token),
        Mode::InTableText => token_in_table_text(p, token),
        Mode::InCaption => end_tag_in_caption(p, token),
        Mode::InColumnGroup => end_tag_in_column_group(p, token),
        Mode::InTableBody => end_tag_in_table_body(p, token),
        Mode::InRow => end_tag_in_row(p, token),
        Mode::InCell => end_tag_in_cell(p, token),
        Mode::InSelect => end_tag_in_select(p, token),
        Mode::InSelectInTable => end_tag_in_select_in_table(p, token),
        Mode::InTemplate => end_tag_in_template(p, token),
        Mode::AfterBody => end_tag_after_body(p, token),
        Mode::InFrameset => end_tag_in_frameset(p, token),
        Mode::AfterFrameset => end_tag_after_frameset(p, token),
        Mode::AfterAfterBody => token_after_after_body(p, token),
        Mode::AfterAfterFrameset => {}
    }
}

fn on_eof(p: &mut Parser, token: &mut Token) {
    match p.mode {
        Mode::Initial => token_in_initial_mode(p, token),
        Mode::BeforeHtml => token_before_html(p, token),
        Mode::BeforeHead => token_before_head(p, token),
        Mode::InHead => token_in_head(p, token),
        Mode::InHeadNoScript => token_in_head_no_script(p, token),
        Mode::AfterHead => token_after_head(p, token),
        Mode::InBody
        | Mode::InTable
        | Mode::InCaption
        | Mode::InColumnGroup
        | Mode::InTableBody
        | Mode::InRow
        | Mode::InCell
        | Mode::InSelect
        | Mode::InSelectInTable => eof_in_body(p, token),
        Mode::Text => eof_in_text(p, token),
        Mode::InTableText => token_in_table_text(p, token),
        Mode::InTemplate => eof_in_template(p, token),
        Mode::AfterBody
        | Mode::InFrameset
        | Mode::AfterFrameset
        | Mode::AfterAfterBody
        | Mode::AfterAfterFrameset => stop_parsing(p, token),
    }
}

fn on_whitespace_character(p: &mut Parser, token: &mut Token) {
    if p.skip_next_new_line {
        p.skip_next_new_line = false;
        if let Token::Character(character) = token
            && character.chars.starts_with('\n')
        {
            if character.chars.len() == 1 {
                return;
            }
            character.chars.remove(0);
        }
    }
    if p.tokenizer.in_foreign_node {
        insert_characters(p, token);
        return;
    }
    match p.mode {
        Mode::InHead
        | Mode::InHeadNoScript
        | Mode::AfterHead
        | Mode::Text
        | Mode::InColumnGroup
        | Mode::InSelect
        | Mode::InSelectInTable
        | Mode::InFrameset
        | Mode::AfterFrameset => insert_characters(p, token),
        Mode::InBody
        | Mode::InCaption
        | Mode::InCell
        | Mode::InTemplate
        | Mode::AfterBody
        | Mode::AfterAfterBody
        | Mode::AfterAfterFrameset => whitespace_character_in_body(p, token),
        Mode::InTable | Mode::InTableBody | Mode::InRow => character_in_table(p, token),
        Mode::InTableText => whitespace_character_in_table_text(p, token),
        _ => {}
    }
}

fn insert_characters(p: &mut Parser, token: &Token) {
    if let Token::Character(character) = token {
        p.insert_characters(character);
    }
}

// Adoption agency

fn aa_obtain_formatting_element_entry(
    p: &mut Parser,
    token: &mut Token,
) -> Option<crate::utils::builder::formatting::ElementEntry> {
    let name = tag_of(token)
        .map(|tag| tag.tag_name.clone())
        .unwrap_or_default();
    match p.formatting.find_in_scope(&p.dom, &name) {
        Some(entry) => {
            if !p.contains(entry.element) {
                p.formatting.remove_entry(entry.id);
                None
            } else if !p.has_in_scope(id(token)) {
                None
            } else {
                Some(entry)
            }
        }
        None => {
            generic_end_tag_in_body(p, token);
            None
        }
    }
}

fn aa_obtain_furthest_block(
    p: &mut Parser,
    entry: &crate::utils::builder::formatting::ElementEntry,
) -> Option<NodeId> {
    let mut furthest = None;
    let mut index = p.stack.len();
    while index > 0 {
        index -= 1;
        let element = p.stack[index];
        if element == entry.element {
            break;
        }
        if p.is_special(element, p.stack_tags[index]) {
            furthest = Some(element);
        }
    }
    if furthest.is_none() {
        p.shorten_to(index);
        p.formatting.remove_entry(entry.id);
    }
    furthest
}

fn aa_inner_loop(p: &mut Parser, furthest: NodeId, formatting_element: NodeId) -> NodeId {
    let mut last = furthest;
    let mut next = p.common_ancestor(furthest);
    let mut counter = 0;
    while let Some(mut element) = next {
        if element == formatting_element {
            break;
        }
        next = p.common_ancestor(element);
        let entry = p.formatting.find_by_element(element);
        let overflow = entry.is_some() && counter >= INNER_LOOP_LIMIT;
        if entry.is_none() || overflow {
            if overflow && let Some(entry) = &entry {
                p.formatting.remove_entry(entry.id);
            }
            p.remove_from_stack(element);
        } else if let Some(entry) = entry {
            element = aa_recreate_element_from_entry(p, &entry);
            if last == furthest {
                p.formatting.bookmark = Some(entry.id);
            }
            p.dom.detach(last);
            p.dom.append_child(element, last);
            last = element;
        }
        counter += 1;
    }
    last
}

fn aa_recreate_element_from_entry(
    p: &mut Parser,
    entry: &crate::utils::builder::formatting::ElementEntry,
) -> NodeId {
    let namespace = p.dom.namespace(entry.element);
    let element =
        p.dom
            .create_element(&entry.token.tag_name, namespace, entry.token.attrs.clone());
    p.replace_on_stack(entry.element, element);
    p.formatting.set_element(entry.id, element);
    element
}

fn aa_insert_last_node_in_common_ancestor(
    p: &mut Parser,
    ancestor: NodeId,
    last: NodeId,
) {
    let tag = crate::helpers::tag::find_tag_id(p.dom.tag_name(ancestor));
    if TABLE_STRUCTURE.contains(&tag) {
        p.foster_parent_element(last);
    } else {
        let mut parent = ancestor;
        if tag == TagId::Template && p.dom.namespace(ancestor) == Namespace::Html {
            parent = p.dom.template_content(ancestor).unwrap_or(ancestor);
        }
        p.dom.append_child(parent, last);
    }
}

fn aa_replace_formatting_element(
    p: &mut Parser,
    furthest: NodeId,
    entry: &crate::utils::builder::formatting::ElementEntry,
) {
    let namespace = p.dom.namespace(entry.element);
    let element =
        p.dom
            .create_element(&entry.token.tag_name, namespace, entry.token.attrs.clone());
    p.dom.adopt_children(furthest, element);
    p.dom.append_child(furthest, element);
    p.formatting
        .insert_element_after_bookmark(&p.dom, element, entry.token.clone());
    p.formatting.remove_entry(entry.id);
    p.remove_from_stack(entry.element);
    p.insert_after_on_stack(furthest, element, entry.token.tag_id);
}

fn call_adoption_agency(p: &mut Parser, token: &mut Token) {
    for _ in 0..OUTER_LOOP_LIMIT {
        let Some(entry) = aa_obtain_formatting_element_entry(p, token) else {
            break;
        };
        let Some(furthest) = aa_obtain_furthest_block(p, &entry) else {
            break;
        };
        p.formatting.bookmark = Some(entry.id);
        let last = aa_inner_loop(p, furthest, entry.element);
        let ancestor = p.common_ancestor(entry.element);
        p.dom.detach(last);
        if let Some(ancestor) = ancestor {
            aa_insert_last_node_in_common_ancestor(p, ancestor, last);
        }
        let entry = p.formatting.find_by_id(entry.id).unwrap_or(entry);
        aa_replace_formatting_element(p, furthest, &entry);
    }
}

// Comments and the end

fn append_comment(p: &mut Parser, token: &Token) {
    let parent = p.current_parent();
    append_comment_to(p, token, parent);
}

fn append_comment_to(p: &mut Parser, token: &Token, parent: NodeId) {
    if let Token::Comment(comment) = token {
        p.append_comment(comment, parent);
    }
}

fn stop_parsing(p: &mut Parser, token: &Token) {
    p.stopped = true;
    let closing = Closing {
        end_tag_name: None,
        span: span_of(token),
    };
    let target = if p.fragment_context.is_some() { 0 } else { 2 };
    for index in (target..p.stack.len()).rev() {
        let node = p.stack[index];
        p.set_end_location(node, &closing);
    }
    if p.fragment_context.is_none()
        && let Some(html) = p.stack.first().copied()
        && p.dom.nodes[html]
            .location
            .is_some_and(|location| location.end_tag.is_none())
    {
        p.set_end_location(html, &closing);
        if let Some(body) = p.stack.get(1).copied()
            && p.dom.nodes[body]
                .location
                .is_some_and(|location| location.end_tag.is_none())
        {
            p.set_end_location(body, &closing);
        }
    }
}

// Initial, before html, before head, in head, after head

fn doctype_in_initial_mode(p: &mut Parser, token: &mut Token) {
    let Token::Doctype(doctype) = token else {
        return;
    };
    p.set_document_type(doctype.span);
    let mode = if doctype.force_quirks {
        DocumentMode::Quirks
    } else {
        find_document_mode(doctype)
    };
    if !is_conforming(doctype) {
        let span = doctype.span;
        p.err(span, ErrorCode::NonConformingDoctype, false);
    }
    p.set_document_mode(mode);
    p.mode = Mode::BeforeHtml;
}

fn token_in_initial_mode(p: &mut Parser, token: &mut Token) {
    p.err(span_of(token), ErrorCode::MissingDoctype, true);
    p.set_document_mode(DocumentMode::Quirks);
    p.mode = Mode::BeforeHtml;
    process_token(p, token);
}

fn start_tag_before_html(p: &mut Parser, token: &mut Token) {
    if id(token) == TagId::Html {
        insert(p, token, Namespace::Html);
        p.mode = Mode::BeforeHead;
    } else {
        token_before_html(p, token);
    }
}

fn end_tag_before_html(p: &mut Parser, token: &mut Token) {
    if matches!(
        id(token),
        TagId::Html | TagId::Head | TagId::Body | TagId::Br
    ) {
        token_before_html(p, token);
    }
}

fn token_before_html(p: &mut Parser, token: &mut Token) {
    p.insert_fake_root_element();
    p.mode = Mode::BeforeHead;
    process_token(p, token);
}

fn start_tag_before_head(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Html => start_tag_in_body(p, token),
        TagId::Head => {
            insert(p, token, Namespace::Html);
            p.head_element = p.current();
            p.mode = Mode::InHead;
        }
        _ => token_before_head(p, token),
    }
}

fn end_tag_before_head(p: &mut Parser, token: &mut Token) {
    if matches!(
        id(token),
        TagId::Head | TagId::Body | TagId::Html | TagId::Br
    ) {
        token_before_head(p, token);
    } else {
        err(p, token, ErrorCode::EndTagWithoutMatchingOpenElement);
    }
}

fn token_before_head(p: &mut Parser, token: &mut Token) {
    p.insert_fake_element("head", TagId::Head);
    p.head_element = p.current();
    p.mode = Mode::InHead;
    process_token(p, token);
}

fn start_tag_in_head(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Html => start_tag_in_body(p, token),
        TagId::Base | TagId::Basefont | TagId::Bgsound | TagId::Link | TagId::Meta => {
            append(p, token, Namespace::Html);
            ack(token);
        }
        TagId::Title => switch_to_text_parsing(p, token, State::Rcdata),
        // Scripting is off, as hast-util-from-html parses
        TagId::Noscript => {
            insert(p, token, Namespace::Html);
            p.mode = Mode::InHeadNoScript;
        }
        TagId::Noframes | TagId::Style => {
            switch_to_text_parsing(p, token, State::Rawtext)
        }
        TagId::Script => switch_to_text_parsing(p, token, State::ScriptData),
        TagId::Template => {
            if let Some(tag) = tag_of(token) {
                p.insert_template(tag);
            }
            p.formatting.insert_marker();
            p.frameset_ok = false;
            p.mode = Mode::InTemplate;
            p.template_modes.push(Mode::InTemplate);
        }
        TagId::Head => err(p, token, ErrorCode::MisplacedStartTagForHeadElement),
        _ => token_in_head(p, token),
    }
}

fn switch_to_text_parsing(p: &mut Parser, token: &Token, state: State) {
    if let Some(tag) = tag_of(token) {
        p.switch_to_text_parsing(tag, state);
    }
}

fn end_tag_in_head(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Head => {
            p.pop();
            p.mode = Mode::AfterHead;
        }
        TagId::Body | TagId::Br | TagId::Html => token_in_head(p, token),
        TagId::Template => template_end_tag_in_head(p, token),
        _ => err(p, token, ErrorCode::EndTagWithoutMatchingOpenElement),
    }
}

fn template_end_tag_in_head(p: &mut Parser, token: &mut Token) {
    if p.template_count > 0 {
        p.generate_implied_end_tags_thoroughly();
        if p.current_tag() != Some(TagId::Template) {
            err(p, token, ErrorCode::ClosingOfElementWithOpenChildElements);
        }
        p.pop_until_tag_popped(TagId::Template);
        p.formatting.clear_to_last_marker();
        p.template_modes.pop();
        p.reset_insertion_mode();
    } else {
        err(p, token, ErrorCode::EndTagWithoutMatchingOpenElement);
    }
}

fn token_in_head(p: &mut Parser, token: &mut Token) {
    p.pop();
    p.mode = Mode::AfterHead;
    process_token(p, token);
}

fn start_tag_in_head_no_script(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Html => start_tag_in_body(p, token),
        TagId::Basefont
        | TagId::Bgsound
        | TagId::Head
        | TagId::Link
        | TagId::Meta
        | TagId::Noframes
        | TagId::Style => start_tag_in_head(p, token),
        TagId::Noscript => err(p, token, ErrorCode::NestedNoscriptInHead),
        _ => token_in_head_no_script(p, token),
    }
}

fn end_tag_in_head_no_script(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Noscript => {
            p.pop();
            p.mode = Mode::InHead;
        }
        TagId::Br => token_in_head_no_script(p, token),
        _ => err(p, token, ErrorCode::EndTagWithoutMatchingOpenElement),
    }
}

fn token_in_head_no_script(p: &mut Parser, token: &mut Token) {
    let code = if matches!(token, Token::Eof(_)) {
        ErrorCode::OpenElementsLeftAfterEof
    } else {
        ErrorCode::DisallowedContentInNoscriptInHead
    };
    err(p, token, code);
    p.pop();
    p.mode = Mode::InHead;
    process_token(p, token);
}

fn start_tag_after_head(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Html => start_tag_in_body(p, token),
        TagId::Body => {
            insert(p, token, Namespace::Html);
            p.frameset_ok = false;
            p.mode = Mode::InBody;
        }
        TagId::Frameset => {
            insert(p, token, Namespace::Html);
            p.mode = Mode::InFrameset;
        }
        TagId::Base
        | TagId::Basefont
        | TagId::Bgsound
        | TagId::Link
        | TagId::Meta
        | TagId::Noframes
        | TagId::Script
        | TagId::Style
        | TagId::Template
        | TagId::Title => {
            err(p, token, ErrorCode::AbandonedHeadElementChild);
            if let Some(head) = p.head_element {
                p.push(head, TagId::Head);
                start_tag_in_head(p, token);
                p.remove_from_stack(head);
            }
        }
        TagId::Head => err(p, token, ErrorCode::MisplacedStartTagForHeadElement),
        _ => token_after_head(p, token),
    }
}

fn end_tag_after_head(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Body | TagId::Html | TagId::Br => token_after_head(p, token),
        TagId::Template => template_end_tag_in_head(p, token),
        _ => err(p, token, ErrorCode::EndTagWithoutMatchingOpenElement),
    }
}

fn token_after_head(p: &mut Parser, token: &mut Token) {
    p.insert_fake_element("body", TagId::Body);
    p.mode = Mode::InBody;
    mode_in_body(p, token);
}

// In body

fn mode_in_body(p: &mut Parser, token: &mut Token) {
    match token {
        Token::Character(character) => match character.kind {
            CharacterKind::Character => character_in_body(p, token),
            CharacterKind::Whitespace => whitespace_character_in_body(p, token),
            CharacterKind::Null => {}
        },
        Token::Comment(_) => append_comment(p, token),
        Token::StartTag(_) => start_tag_in_body(p, token),
        Token::EndTag(_) => end_tag_in_body(p, token),
        Token::Eof(_) => eof_in_body(p, token),
        Token::Doctype(_) => {}
    }
}

fn whitespace_character_in_body(p: &mut Parser, token: &mut Token) {
    p.reconstruct_active_formatting_elements();
    insert_characters(p, token);
}

fn character_in_body(p: &mut Parser, token: &mut Token) {
    p.reconstruct_active_formatting_elements();
    insert_characters(p, token);
    p.frameset_ok = false;
}

fn adopt_attributes(p: &mut Parser, recipient: NodeId, token: &Token) {
    let Some(tag) = tag_of(token) else {
        return;
    };
    if let Some(element) = p.dom.element_mut(recipient) {
        for attribute in &tag.attrs {
            if !element.attrs.iter().any(|each| each.name == attribute.name) {
                element.attrs.push(attribute.clone());
            }
        }
    }
}

fn html_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    if p.template_count == 0
        && let Some(html) = p.stack.first().copied()
    {
        adopt_attributes(p, html, token);
    }
}

fn body_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    if let Some(body) = p.properly_nested_body()
        && p.template_count == 0
    {
        p.frameset_ok = false;
        adopt_attributes(p, body, token);
    }
}

fn frameset_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    if p.frameset_ok
        && let Some(body) = p.properly_nested_body()
    {
        p.dom.detach(body);
        p.pop_all_up_to_html_element();
        insert(p, token, Namespace::Html);
        p.mode = Mode::InFrameset;
    }
}

fn address_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    if p.has_in_button_scope(TagId::P) {
        p.close_p_element();
    }
    insert(p, token, Namespace::Html);
}

fn numbered_header_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    if p.has_in_button_scope(TagId::P) {
        p.close_p_element();
    }
    if p.current_tag()
        .is_some_and(|tag| NUMBERED_HEADERS.contains(&tag))
    {
        p.pop();
    }
    insert(p, token, Namespace::Html);
}

fn pre_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    if p.has_in_button_scope(TagId::P) {
        p.close_p_element();
    }
    insert(p, token, Namespace::Html);
    p.skip_next_new_line = true;
    p.frameset_ok = false;
}

fn form_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    let in_template = p.template_count > 0;
    if p.form_element.is_none() || in_template {
        if p.has_in_button_scope(TagId::P) {
            p.close_p_element();
        }
        insert(p, token, Namespace::Html);
        if !in_template {
            p.form_element = p.current();
        }
    }
}

fn list_item_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    p.frameset_ok = false;
    let tag = id(token);
    for index in (0..p.stack.len()).rev() {
        let element_tag = p.stack_tags[index];
        if (tag == TagId::Li && element_tag == TagId::Li)
            || (matches!(tag, TagId::Dd | TagId::Dt)
                && matches!(element_tag, TagId::Dd | TagId::Dt))
        {
            p.generate_implied_end_tags_except(element_tag);
            p.pop_until_tag_popped(element_tag);
            break;
        }
        if !matches!(element_tag, TagId::Address | TagId::Div | TagId::P)
            && p.is_special(p.stack[index], element_tag)
        {
            break;
        }
    }
    if p.has_in_button_scope(TagId::P) {
        p.close_p_element();
    }
    insert(p, token, Namespace::Html);
}

fn plaintext_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    if p.has_in_button_scope(TagId::P) {
        p.close_p_element();
    }
    insert(p, token, Namespace::Html);
    p.tokenizer.state = State::Plaintext;
}

fn button_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    if p.has_in_scope(TagId::Button) {
        p.generate_implied_end_tags();
        p.pop_until_tag_popped(TagId::Button);
    }
    p.reconstruct_active_formatting_elements();
    insert(p, token, Namespace::Html);
    p.frameset_ok = false;
}

fn push_formatting(p: &mut Parser, token: &Token) {
    if let (Some(current), Some(tag)) = (p.current(), tag_of(token)) {
        let tag = tag.clone();
        p.formatting.push_element(&p.dom, current, tag);
    }
}

fn a_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    if let Some(entry) = p.formatting.find_in_scope(&p.dom, "a") {
        call_adoption_agency(p, token);
        let element = p
            .formatting
            .find_by_id(entry.id)
            .map_or(entry.element, |entry| entry.element);
        p.remove_from_stack(element);
        p.formatting.remove_entry(entry.id);
    }
    p.reconstruct_active_formatting_elements();
    insert(p, token, Namespace::Html);
    push_formatting(p, token);
}

fn b_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    p.reconstruct_active_formatting_elements();
    insert(p, token, Namespace::Html);
    push_formatting(p, token);
}

fn nobr_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    p.reconstruct_active_formatting_elements();
    if p.has_in_scope(TagId::Nobr) {
        call_adoption_agency(p, token);
        p.reconstruct_active_formatting_elements();
    }
    insert(p, token, Namespace::Html);
    push_formatting(p, token);
}

fn applet_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    p.reconstruct_active_formatting_elements();
    insert(p, token, Namespace::Html);
    p.formatting.insert_marker();
    p.frameset_ok = false;
}

fn table_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    if p.document_mode() != DocumentMode::Quirks && p.has_in_button_scope(TagId::P) {
        p.close_p_element();
    }
    insert(p, token, Namespace::Html);
    p.frameset_ok = false;
    p.mode = Mode::InTable;
}

fn area_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    p.reconstruct_active_formatting_elements();
    append(p, token, Namespace::Html);
    p.frameset_ok = false;
    ack(token);
}

fn input_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    p.reconstruct_active_formatting_elements();
    append(p, token, Namespace::Html);
    if !is_hidden_input(token) {
        p.frameset_ok = false;
    }
    ack(token);
}

fn param_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    append(p, token, Namespace::Html);
    ack(token);
}

fn hr_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    if p.has_in_button_scope(TagId::P) {
        p.close_p_element();
    }
    append(p, token, Namespace::Html);
    p.frameset_ok = false;
    ack(token);
}

fn image_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    if let Token::StartTag(tag) = token {
        tag.tag_name = "img".to_string();
        tag.tag_id = TagId::Img;
    }
    area_start_tag_in_body(p, token);
}

fn textarea_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    insert(p, token, Namespace::Html);
    p.skip_next_new_line = true;
    p.tokenizer.state = State::Rcdata;
    p.original_mode = p.mode;
    p.frameset_ok = false;
    p.mode = Mode::Text;
}

fn xmp_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    if p.has_in_button_scope(TagId::P) {
        p.close_p_element();
    }
    p.reconstruct_active_formatting_elements();
    p.frameset_ok = false;
    switch_to_text_parsing(p, token, State::Rawtext);
}

fn iframe_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    p.frameset_ok = false;
    switch_to_text_parsing(p, token, State::Rawtext);
}

fn select_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    p.reconstruct_active_formatting_elements();
    insert(p, token, Namespace::Html);
    p.frameset_ok = false;
    p.mode = if matches!(
        p.mode,
        Mode::InTable | Mode::InCaption | Mode::InTableBody | Mode::InRow | Mode::InCell
    ) {
        Mode::InSelectInTable
    } else {
        Mode::InSelect
    };
}

fn optgroup_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    if p.current_tag() == Some(TagId::Option) {
        p.pop();
    }
    p.reconstruct_active_formatting_elements();
    insert(p, token, Namespace::Html);
}

fn rb_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    if p.has_in_scope(TagId::Ruby) {
        p.generate_implied_end_tags();
    }
    insert(p, token, Namespace::Html);
}

fn rt_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    if p.has_in_scope(TagId::Ruby) {
        p.generate_implied_end_tags_except(TagId::Rtc);
    }
    insert(p, token, Namespace::Html);
}

fn foreign_start_tag_in_body(p: &mut Parser, token: &mut Token, namespace: Namespace) {
    p.reconstruct_active_formatting_elements();
    if let Token::StartTag(tag) = token {
        if namespace == Namespace::MathMl {
            adjust_mathml_attributes(tag);
        } else {
            adjust_svg_attributes(tag);
        }
        adjust_xml_attributes(tag);
        if tag.self_closing {
            p.append_element(tag, namespace);
        } else {
            p.insert_element(tag, namespace);
        }
        tag.ack_self_closing = true;
    }
}

fn generic_start_tag_in_body(p: &mut Parser, token: &mut Token) {
    p.reconstruct_active_formatting_elements();
    insert(p, token, Namespace::Html);
}

fn start_tag_in_body(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::I
        | TagId::S
        | TagId::B
        | TagId::U
        | TagId::Em
        | TagId::Tt
        | TagId::Big
        | TagId::Code
        | TagId::Font
        | TagId::Small
        | TagId::Strike
        | TagId::Strong => b_start_tag_in_body(p, token),
        TagId::A => a_start_tag_in_body(p, token),
        TagId::H1 | TagId::H2 | TagId::H3 | TagId::H4 | TagId::H5 | TagId::H6 => {
            numbered_header_start_tag_in_body(p, token)
        }
        TagId::P
        | TagId::Dl
        | TagId::Ol
        | TagId::Ul
        | TagId::Div
        | TagId::Dir
        | TagId::Nav
        | TagId::Main
        | TagId::Menu
        | TagId::Aside
        | TagId::Center
        | TagId::Figure
        | TagId::Footer
        | TagId::Header
        | TagId::Hgroup
        | TagId::Dialog
        | TagId::Details
        | TagId::Address
        | TagId::Article
        | TagId::Search
        | TagId::Section
        | TagId::Summary
        | TagId::Fieldset
        | TagId::Blockquote
        | TagId::Figcaption => address_start_tag_in_body(p, token),
        TagId::Li | TagId::Dd | TagId::Dt => list_item_start_tag_in_body(p, token),
        TagId::Br
        | TagId::Img
        | TagId::Wbr
        | TagId::Area
        | TagId::Embed
        | TagId::Keygen => area_start_tag_in_body(p, token),
        TagId::Hr => hr_start_tag_in_body(p, token),
        TagId::Rb | TagId::Rtc => rb_start_tag_in_body(p, token),
        TagId::Rt | TagId::Rp => rt_start_tag_in_body(p, token),
        TagId::Pre | TagId::Listing => pre_start_tag_in_body(p, token),
        TagId::Xmp => xmp_start_tag_in_body(p, token),
        TagId::Svg => foreign_start_tag_in_body(p, token, Namespace::Svg),
        TagId::Html => html_start_tag_in_body(p, token),
        TagId::Base
        | TagId::Link
        | TagId::Meta
        | TagId::Style
        | TagId::Title
        | TagId::Script
        | TagId::Bgsound
        | TagId::Basefont
        | TagId::Template => start_tag_in_head(p, token),
        TagId::Body => body_start_tag_in_body(p, token),
        TagId::Form => form_start_tag_in_body(p, token),
        TagId::Nobr => nobr_start_tag_in_body(p, token),
        TagId::Math => foreign_start_tag_in_body(p, token, Namespace::MathMl),
        TagId::Table => table_start_tag_in_body(p, token),
        TagId::Input => input_start_tag_in_body(p, token),
        TagId::Param | TagId::Track | TagId::Source => param_start_tag_in_body(p, token),
        TagId::Image => image_start_tag_in_body(p, token),
        TagId::Button => button_start_tag_in_body(p, token),
        TagId::Applet | TagId::Object | TagId::Marquee => {
            applet_start_tag_in_body(p, token)
        }
        TagId::Iframe => iframe_start_tag_in_body(p, token),
        TagId::Select => select_start_tag_in_body(p, token),
        TagId::Option | TagId::Optgroup => optgroup_start_tag_in_body(p, token),
        TagId::Noembed | TagId::Noframes => {
            switch_to_text_parsing(p, token, State::Rawtext)
        }
        TagId::Frameset => frameset_start_tag_in_body(p, token),
        TagId::Textarea => textarea_start_tag_in_body(p, token),
        TagId::Noscript => generic_start_tag_in_body(p, token),
        TagId::Plaintext => plaintext_start_tag_in_body(p, token),
        TagId::Col
        | TagId::Th
        | TagId::Td
        | TagId::Tr
        | TagId::Head
        | TagId::Frame
        | TagId::Tbody
        | TagId::Tfoot
        | TagId::Thead
        | TagId::Caption
        | TagId::Colgroup => {}
        _ => generic_start_tag_in_body(p, token),
    }
}

fn body_end_tag_in_body(p: &mut Parser, token: &mut Token) {
    if p.has_in_scope(TagId::Body) {
        p.mode = Mode::AfterBody;
        if let Some(body) = p.properly_nested_body() {
            let closing = closing_of(token);
            p.set_end_location(body, &closing);
        }
    }
}

fn closing_of(token: &Token) -> Closing {
    let end_tag_name = match token {
        Token::EndTag(tag) => Some(tag.tag_name.clone()),
        _ => None,
    };
    Closing {
        end_tag_name,
        span: span_of(token),
    }
}

fn html_end_tag_in_body(p: &mut Parser, token: &mut Token) {
    if p.has_in_scope(TagId::Body) {
        p.mode = Mode::AfterBody;
        end_tag_after_body(p, token);
    }
}

fn address_end_tag_in_body(p: &mut Parser, token: &mut Token) {
    let tag = id(token);
    if p.has_in_scope(tag) {
        p.generate_implied_end_tags();
        p.pop_until_tag_popped(tag);
    }
}

fn form_end_tag_in_body(p: &mut Parser) {
    let in_template = p.template_count > 0;
    let form = p.form_element;
    if !in_template {
        p.form_element = None;
    }
    if (form.is_some() || in_template) && p.has_in_scope(TagId::Form) {
        p.generate_implied_end_tags();
        if in_template {
            p.pop_until_tag_popped(TagId::Form);
        } else if let Some(form) = form {
            p.remove_from_stack(form);
        }
    }
}

fn p_end_tag_in_body(p: &mut Parser) {
    if !p.has_in_button_scope(TagId::P) {
        p.insert_fake_element("p", TagId::P);
    }
    p.close_p_element();
}

fn li_end_tag_in_body(p: &mut Parser) {
    if p.has_in_list_item_scope(TagId::Li) {
        p.generate_implied_end_tags_except(TagId::Li);
        p.pop_until_tag_popped(TagId::Li);
    }
}

fn dd_end_tag_in_body(p: &mut Parser, token: &mut Token) {
    let tag = id(token);
    if p.has_in_scope(tag) {
        p.generate_implied_end_tags_except(tag);
        p.pop_until_tag_popped(tag);
    }
}

fn numbered_header_end_tag_in_body(p: &mut Parser) {
    if p.has_numbered_header_in_scope() {
        p.generate_implied_end_tags();
        p.pop_until_numbered_header_popped();
    }
}

fn applet_end_tag_in_body(p: &mut Parser, token: &mut Token) {
    let tag = id(token);
    if p.has_in_scope(tag) {
        p.generate_implied_end_tags();
        p.pop_until_tag_popped(tag);
        p.formatting.clear_to_last_marker();
    }
}

fn br_end_tag_in_body(p: &mut Parser) {
    p.reconstruct_active_formatting_elements();
    p.insert_fake_element("br", TagId::Br);
    p.pop();
    p.frameset_ok = false;
}

fn generic_end_tag_in_body(p: &mut Parser, token: &mut Token) {
    let Some(tag) = tag_of(token) else {
        return;
    };
    let (name, tag) = (tag.tag_name.clone(), tag.tag_id);
    if !p.has_open_tag(tag, &name) {
        return;
    }
    for index in (1..p.stack.len()).rev() {
        let element = p.stack[index];
        let element_tag = p.stack_tags[index];
        if tag == element_tag
            && (tag != TagId::Unknown || p.dom.tag_name(element) == name)
        {
            p.generate_implied_end_tags_except(tag);
            if p.stack.len() > index {
                p.shorten_to(index);
            }
            break;
        }
        if p.is_special(element, element_tag) {
            break;
        }
    }
}

fn end_tag_in_body(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::A
        | TagId::B
        | TagId::I
        | TagId::S
        | TagId::U
        | TagId::Em
        | TagId::Tt
        | TagId::Big
        | TagId::Code
        | TagId::Font
        | TagId::Nobr
        | TagId::Small
        | TagId::Strike
        | TagId::Strong => call_adoption_agency(p, token),
        TagId::P => p_end_tag_in_body(p),
        TagId::Dl
        | TagId::Ul
        | TagId::Ol
        | TagId::Dir
        | TagId::Div
        | TagId::Nav
        | TagId::Pre
        | TagId::Main
        | TagId::Menu
        | TagId::Aside
        | TagId::Button
        | TagId::Center
        | TagId::Figure
        | TagId::Footer
        | TagId::Header
        | TagId::Hgroup
        | TagId::Dialog
        | TagId::Address
        | TagId::Article
        | TagId::Details
        | TagId::Search
        | TagId::Section
        | TagId::Summary
        | TagId::Listing
        | TagId::Fieldset
        | TagId::Blockquote
        | TagId::Figcaption => address_end_tag_in_body(p, token),
        TagId::Li => li_end_tag_in_body(p),
        TagId::Dd | TagId::Dt => dd_end_tag_in_body(p, token),
        TagId::H1 | TagId::H2 | TagId::H3 | TagId::H4 | TagId::H5 | TagId::H6 => {
            numbered_header_end_tag_in_body(p)
        }
        TagId::Br => br_end_tag_in_body(p),
        TagId::Body => body_end_tag_in_body(p, token),
        TagId::Html => html_end_tag_in_body(p, token),
        TagId::Form => form_end_tag_in_body(p),
        TagId::Applet | TagId::Object | TagId::Marquee => {
            applet_end_tag_in_body(p, token)
        }
        TagId::Template => template_end_tag_in_head(p, token),
        _ => generic_end_tag_in_body(p, token),
    }
}

fn eof_in_body(p: &mut Parser, token: &mut Token) {
    if p.template_modes.is_empty() {
        stop_parsing(p, token);
    } else {
        eof_in_template(p, token);
    }
}

fn end_tag_in_text(p: &mut Parser) {
    p.pop();
    p.mode = p.original_mode;
}

fn eof_in_text(p: &mut Parser, token: &mut Token) {
    err(p, token, ErrorCode::EofInElementThatCanContainOnlyText);
    p.pop();
    p.mode = p.original_mode;
    on_eof(p, token);
}

// In table

fn character_in_table(p: &mut Parser, token: &mut Token) {
    if p.current_tag()
        .is_some_and(|tag| TABLE_STRUCTURE.contains(&tag))
    {
        p.pending_characters.clear();
        p.has_non_whitespace_pending = false;
        p.original_mode = p.mode;
        p.mode = Mode::InTableText;
        match token {
            Token::Character(character) if character.kind == CharacterKind::Character => {
                character_in_table_text(p, token)
            }
            Token::Character(character)
                if character.kind == CharacterKind::Whitespace =>
            {
                whitespace_character_in_table_text(p, token)
            }
            _ => {}
        }
    } else {
        token_in_table(p, token);
    }
}

fn caption_start_tag_in_table(p: &mut Parser, token: &mut Token) {
    p.clear_back_to_table_context();
    p.formatting.insert_marker();
    insert(p, token, Namespace::Html);
    p.mode = Mode::InCaption;
}

fn colgroup_start_tag_in_table(p: &mut Parser, token: &mut Token) {
    p.clear_back_to_table_context();
    insert(p, token, Namespace::Html);
    p.mode = Mode::InColumnGroup;
}

fn col_start_tag_in_table(p: &mut Parser, token: &mut Token) {
    p.clear_back_to_table_context();
    p.insert_fake_element("colgroup", TagId::Colgroup);
    p.mode = Mode::InColumnGroup;
    start_tag_in_column_group(p, token);
}

fn tbody_start_tag_in_table(p: &mut Parser, token: &mut Token) {
    p.clear_back_to_table_context();
    insert(p, token, Namespace::Html);
    p.mode = Mode::InTableBody;
}

fn td_start_tag_in_table(p: &mut Parser, token: &mut Token) {
    p.clear_back_to_table_context();
    p.insert_fake_element("tbody", TagId::Tbody);
    p.mode = Mode::InTableBody;
    start_tag_in_table_body(p, token);
}

fn table_start_tag_in_table(p: &mut Parser, token: &mut Token) {
    if p.has_in_table_scope(TagId::Table) {
        p.pop_until_tag_popped(TagId::Table);
        p.reset_insertion_mode();
        process_start_tag(p, token);
    }
}

fn input_start_tag_in_table(p: &mut Parser, token: &mut Token) {
    if is_hidden_input(token) {
        append(p, token, Namespace::Html);
    } else {
        token_in_table(p, token);
    }
    ack(token);
}

fn form_start_tag_in_table(p: &mut Parser, token: &mut Token) {
    if p.form_element.is_none() && p.template_count == 0 {
        insert(p, token, Namespace::Html);
        p.form_element = p.current();
        p.pop();
    }
}

fn start_tag_in_table(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Td | TagId::Th | TagId::Tr => td_start_tag_in_table(p, token),
        TagId::Style | TagId::Script | TagId::Template => start_tag_in_head(p, token),
        TagId::Col => col_start_tag_in_table(p, token),
        TagId::Form => form_start_tag_in_table(p, token),
        TagId::Table => table_start_tag_in_table(p, token),
        TagId::Tbody | TagId::Tfoot | TagId::Thead => tbody_start_tag_in_table(p, token),
        TagId::Input => input_start_tag_in_table(p, token),
        TagId::Caption => caption_start_tag_in_table(p, token),
        TagId::Colgroup => colgroup_start_tag_in_table(p, token),
        _ => token_in_table(p, token),
    }
}

fn end_tag_in_table(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Table => {
            if p.has_in_table_scope(TagId::Table) {
                p.pop_until_tag_popped(TagId::Table);
                p.reset_insertion_mode();
            }
        }
        TagId::Template => template_end_tag_in_head(p, token),
        TagId::Body
        | TagId::Caption
        | TagId::Col
        | TagId::Colgroup
        | TagId::Html
        | TagId::Tbody
        | TagId::Td
        | TagId::Tfoot
        | TagId::Th
        | TagId::Thead
        | TagId::Tr => {}
        _ => token_in_table(p, token),
    }
}

fn token_in_table(p: &mut Parser, token: &mut Token) {
    let saved = p.foster_parenting;
    p.foster_parenting = true;
    mode_in_body(p, token);
    p.foster_parenting = saved;
}

fn whitespace_character_in_table_text(p: &mut Parser, token: &mut Token) {
    if let Token::Character(character) = token {
        p.pending_characters.push(character.clone());
    }
}

fn character_in_table_text(p: &mut Parser, token: &mut Token) {
    if let Token::Character(character) = token {
        p.pending_characters.push(character.clone());
        p.has_non_whitespace_pending = true;
    }
}

fn token_in_table_text(p: &mut Parser, token: &mut Token) {
    let pending = std::mem::take(&mut p.pending_characters);
    if p.has_non_whitespace_pending {
        for character in pending {
            token_in_table(p, &mut Token::Character(character));
        }
    } else {
        for character in &pending {
            p.insert_characters(character);
        }
    }
    p.mode = p.original_mode;
    process_token(p, token);
}

fn start_tag_in_caption(p: &mut Parser, token: &mut Token) {
    if TABLE_VOID.contains(&id(token)) {
        if p.has_in_table_scope(TagId::Caption) {
            p.generate_implied_end_tags();
            p.pop_until_tag_popped(TagId::Caption);
            p.formatting.clear_to_last_marker();
            p.mode = Mode::InTable;
            start_tag_in_table(p, token);
        }
    } else {
        start_tag_in_body(p, token);
    }
}

fn end_tag_in_caption(p: &mut Parser, token: &mut Token) {
    let tag = id(token);
    match tag {
        TagId::Caption | TagId::Table => {
            if p.has_in_table_scope(TagId::Caption) {
                p.generate_implied_end_tags();
                p.pop_until_tag_popped(TagId::Caption);
                p.formatting.clear_to_last_marker();
                p.mode = Mode::InTable;
                if tag == TagId::Table {
                    end_tag_in_table(p, token);
                }
            }
        }
        TagId::Body
        | TagId::Col
        | TagId::Colgroup
        | TagId::Html
        | TagId::Tbody
        | TagId::Td
        | TagId::Tfoot
        | TagId::Th
        | TagId::Thead
        | TagId::Tr => {}
        _ => end_tag_in_body(p, token),
    }
}

fn start_tag_in_column_group(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Html => start_tag_in_body(p, token),
        TagId::Col => {
            append(p, token, Namespace::Html);
            ack(token);
        }
        TagId::Template => start_tag_in_head(p, token),
        _ => token_in_column_group(p, token),
    }
}

fn end_tag_in_column_group(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Colgroup => {
            if p.current_tag() == Some(TagId::Colgroup) {
                p.pop();
                p.mode = Mode::InTable;
            }
        }
        TagId::Template => template_end_tag_in_head(p, token),
        TagId::Col => {}
        _ => token_in_column_group(p, token),
    }
}

fn token_in_column_group(p: &mut Parser, token: &mut Token) {
    if p.current_tag() == Some(TagId::Colgroup) {
        p.pop();
        p.mode = Mode::InTable;
        process_token(p, token);
    }
}

fn start_tag_in_table_body(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Tr => {
            p.clear_back_to_table_body_context();
            insert(p, token, Namespace::Html);
            p.mode = Mode::InRow;
        }
        TagId::Th | TagId::Td => {
            p.clear_back_to_table_body_context();
            p.insert_fake_element("tr", TagId::Tr);
            p.mode = Mode::InRow;
            start_tag_in_row(p, token);
        }
        TagId::Caption
        | TagId::Col
        | TagId::Colgroup
        | TagId::Tbody
        | TagId::Tfoot
        | TagId::Thead => {
            if p.has_table_body_context_in_table_scope() {
                p.clear_back_to_table_body_context();
                p.pop();
                p.mode = Mode::InTable;
                start_tag_in_table(p, token);
            }
        }
        _ => start_tag_in_table(p, token),
    }
}

fn end_tag_in_table_body(p: &mut Parser, token: &mut Token) {
    let tag = id(token);
    match tag {
        TagId::Tbody | TagId::Tfoot | TagId::Thead => {
            if p.has_in_table_scope(tag) {
                p.clear_back_to_table_body_context();
                p.pop();
                p.mode = Mode::InTable;
            }
        }
        TagId::Table => {
            if p.has_table_body_context_in_table_scope() {
                p.clear_back_to_table_body_context();
                p.pop();
                p.mode = Mode::InTable;
                end_tag_in_table(p, token);
            }
        }
        TagId::Body
        | TagId::Caption
        | TagId::Col
        | TagId::Colgroup
        | TagId::Html
        | TagId::Td
        | TagId::Th
        | TagId::Tr => {}
        _ => end_tag_in_table(p, token),
    }
}

fn start_tag_in_row(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Th | TagId::Td => {
            p.clear_back_to_table_row_context();
            insert(p, token, Namespace::Html);
            p.mode = Mode::InCell;
            p.formatting.insert_marker();
        }
        TagId::Caption
        | TagId::Col
        | TagId::Colgroup
        | TagId::Tbody
        | TagId::Tfoot
        | TagId::Thead
        | TagId::Tr => {
            if p.has_in_table_scope(TagId::Tr) {
                p.clear_back_to_table_row_context();
                p.pop();
                p.mode = Mode::InTableBody;
                start_tag_in_table_body(p, token);
            }
        }
        _ => start_tag_in_table(p, token),
    }
}

fn end_tag_in_row(p: &mut Parser, token: &mut Token) {
    let tag = id(token);
    match tag {
        TagId::Tr => {
            if p.has_in_table_scope(TagId::Tr) {
                p.clear_back_to_table_row_context();
                p.pop();
                p.mode = Mode::InTableBody;
            }
        }
        TagId::Table => {
            if p.has_in_table_scope(TagId::Tr) {
                p.clear_back_to_table_row_context();
                p.pop();
                p.mode = Mode::InTableBody;
                end_tag_in_table_body(p, token);
            }
        }
        TagId::Tbody | TagId::Tfoot | TagId::Thead => {
            if p.has_in_table_scope(tag) || p.has_in_table_scope(TagId::Tr) {
                p.clear_back_to_table_row_context();
                p.pop();
                p.mode = Mode::InTableBody;
                end_tag_in_table_body(p, token);
            }
        }
        TagId::Body
        | TagId::Caption
        | TagId::Col
        | TagId::Colgroup
        | TagId::Html
        | TagId::Td
        | TagId::Th => {}
        _ => end_tag_in_table(p, token),
    }
}

fn start_tag_in_cell(p: &mut Parser, token: &mut Token) {
    if TABLE_VOID.contains(&id(token)) {
        if p.has_in_table_scope(TagId::Td) || p.has_in_table_scope(TagId::Th) {
            p.close_table_cell();
            start_tag_in_row(p, token);
        }
    } else {
        start_tag_in_body(p, token);
    }
}

fn end_tag_in_cell(p: &mut Parser, token: &mut Token) {
    let tag = id(token);
    match tag {
        TagId::Td | TagId::Th => {
            if p.has_in_table_scope(tag) {
                p.generate_implied_end_tags();
                p.pop_until_tag_popped(tag);
                p.formatting.clear_to_last_marker();
                p.mode = Mode::InRow;
            }
        }
        TagId::Table | TagId::Tbody | TagId::Tfoot | TagId::Thead | TagId::Tr => {
            if p.has_in_table_scope(tag) {
                p.close_table_cell();
                end_tag_in_row(p, token);
            }
        }
        TagId::Body | TagId::Caption | TagId::Col | TagId::Colgroup | TagId::Html => {}
        _ => end_tag_in_body(p, token),
    }
}

// In select, template, after body, frameset

fn start_tag_in_select(p: &mut Parser, token: &mut Token) {
    let tag = id(token);
    match tag {
        TagId::Html => start_tag_in_body(p, token),
        TagId::Option => {
            if p.current_tag() == Some(TagId::Option) {
                p.pop();
            }
            insert(p, token, Namespace::Html);
        }
        TagId::Optgroup => {
            if p.current_tag() == Some(TagId::Option) {
                p.pop();
            }
            if p.current_tag() == Some(TagId::Optgroup) {
                p.pop();
            }
            insert(p, token, Namespace::Html);
        }
        TagId::Hr => {
            if p.current_tag() == Some(TagId::Option) {
                p.pop();
            }
            if p.current_tag() == Some(TagId::Optgroup) {
                p.pop();
            }
            append(p, token, Namespace::Html);
            ack(token);
        }
        TagId::Input | TagId::Keygen | TagId::Textarea | TagId::Select => {
            if p.has_in_select_scope(TagId::Select) {
                p.pop_until_tag_popped(TagId::Select);
                p.reset_insertion_mode();
                if tag != TagId::Select {
                    process_start_tag(p, token);
                }
            }
        }
        TagId::Script | TagId::Template => start_tag_in_head(p, token),
        _ => {}
    }
}

fn end_tag_in_select(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Optgroup => {
            if p.stack.len() > 1
                && p.current_tag() == Some(TagId::Option)
                && p.stack_tags[p.stack.len() - 2] == TagId::Optgroup
            {
                p.pop();
            }
            if p.current_tag() == Some(TagId::Optgroup) {
                p.pop();
            }
        }
        TagId::Option => {
            if p.current_tag() == Some(TagId::Option) {
                p.pop();
            }
        }
        TagId::Select => {
            if p.has_in_select_scope(TagId::Select) {
                p.pop_until_tag_popped(TagId::Select);
                p.reset_insertion_mode();
            }
        }
        TagId::Template => template_end_tag_in_head(p, token),
        _ => {}
    }
}

const SELECT_IN_TABLE_EXITS: [TagId; 8] = [
    TagId::Caption,
    TagId::Table,
    TagId::Tbody,
    TagId::Tfoot,
    TagId::Thead,
    TagId::Tr,
    TagId::Td,
    TagId::Th,
];

fn start_tag_in_select_in_table(p: &mut Parser, token: &mut Token) {
    if SELECT_IN_TABLE_EXITS.contains(&id(token)) {
        p.pop_until_tag_popped(TagId::Select);
        p.reset_insertion_mode();
        process_start_tag(p, token);
    } else {
        start_tag_in_select(p, token);
    }
}

fn end_tag_in_select_in_table(p: &mut Parser, token: &mut Token) {
    let tag = id(token);
    if SELECT_IN_TABLE_EXITS.contains(&tag) {
        if p.has_in_table_scope(tag) {
            p.pop_until_tag_popped(TagId::Select);
            p.reset_insertion_mode();
            on_end_tag(p, token);
        }
    } else {
        end_tag_in_select(p, token);
    }
}

fn set_template_mode(p: &mut Parser, mode: Mode) {
    if let Some(front) = p.template_modes.last_mut() {
        *front = mode;
    }
    p.mode = mode;
}

fn start_tag_in_template(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Base
        | TagId::Basefont
        | TagId::Bgsound
        | TagId::Link
        | TagId::Meta
        | TagId::Noframes
        | TagId::Script
        | TagId::Style
        | TagId::Template
        | TagId::Title => start_tag_in_head(p, token),
        TagId::Caption | TagId::Colgroup | TagId::Tbody | TagId::Tfoot | TagId::Thead => {
            set_template_mode(p, Mode::InTable);
            start_tag_in_table(p, token);
        }
        TagId::Col => {
            set_template_mode(p, Mode::InColumnGroup);
            start_tag_in_column_group(p, token);
        }
        TagId::Tr => {
            set_template_mode(p, Mode::InTableBody);
            start_tag_in_table_body(p, token);
        }
        TagId::Td | TagId::Th => {
            set_template_mode(p, Mode::InRow);
            start_tag_in_row(p, token);
        }
        _ => {
            set_template_mode(p, Mode::InBody);
            start_tag_in_body(p, token);
        }
    }
}

fn end_tag_in_template(p: &mut Parser, token: &mut Token) {
    if id(token) == TagId::Template {
        template_end_tag_in_head(p, token);
    }
}

// parse5 recurses through `onEof` once per open template; nested templates loop here
fn eof_in_template(p: &mut Parser, token: &mut Token) {
    while p.template_count > 0 {
        p.pop_until_tag_popped(TagId::Template);
        p.formatting.clear_to_last_marker();
        p.template_modes.pop();
        p.reset_insertion_mode();
        // Modes whose end of file would come back here
        let loops = match p.mode {
            Mode::InTemplate => true,
            Mode::InBody
            | Mode::InTable
            | Mode::InCaption
            | Mode::InColumnGroup
            | Mode::InTableBody
            | Mode::InRow
            | Mode::InCell
            | Mode::InSelect
            | Mode::InSelectInTable => !p.template_modes.is_empty(),
            _ => false,
        };
        if !loops {
            on_eof(p, token);
            return;
        }
    }
    stop_parsing(p, token);
}

fn start_tag_after_body(p: &mut Parser, token: &mut Token) {
    if id(token) == TagId::Html {
        start_tag_in_body(p, token);
    } else {
        token_after_body(p, token);
    }
}

fn end_tag_after_body(p: &mut Parser, token: &mut Token) {
    if id(token) == TagId::Html {
        if p.fragment_context.is_none() {
            p.mode = Mode::AfterAfterBody;
        }
        if p.stack_tags.first() == Some(&TagId::Html) {
            let closing = closing_of(token);
            let html = p.stack[0];
            p.set_end_location(html, &closing);
            if let Some(body) = p.stack.get(1).copied()
                && !p.dom.nodes[body]
                    .location
                    .is_some_and(|location| location.end_tag.is_some())
            {
                p.set_end_location(body, &closing);
            }
        }
    } else {
        token_after_body(p, token);
    }
}

fn token_after_body(p: &mut Parser, token: &mut Token) {
    p.mode = Mode::InBody;
    mode_in_body(p, token);
}

fn start_tag_in_frameset(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Html => start_tag_in_body(p, token),
        TagId::Frameset => insert(p, token, Namespace::Html),
        TagId::Frame => {
            append(p, token, Namespace::Html);
            ack(token);
        }
        TagId::Noframes => start_tag_in_head(p, token),
        _ => {}
    }
}

fn end_tag_in_frameset(p: &mut Parser, token: &mut Token) {
    if id(token) == TagId::Frameset && !p.is_root_html_current() {
        p.pop();
        if p.fragment_context.is_none() && p.current_tag() != Some(TagId::Frameset) {
            p.mode = Mode::AfterFrameset;
        }
    }
}

fn start_tag_after_frameset(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Html => start_tag_in_body(p, token),
        TagId::Noframes => start_tag_in_head(p, token),
        _ => {}
    }
}

fn end_tag_after_frameset(p: &mut Parser, token: &mut Token) {
    if id(token) == TagId::Html {
        p.mode = Mode::AfterAfterFrameset;
    }
}

fn start_tag_after_after_body(p: &mut Parser, token: &mut Token) {
    if id(token) == TagId::Html {
        start_tag_in_body(p, token);
    } else {
        token_after_after_body(p, token);
    }
}

fn token_after_after_body(p: &mut Parser, token: &mut Token) {
    p.mode = Mode::InBody;
    mode_in_body(p, token);
}

fn start_tag_after_after_frameset(p: &mut Parser, token: &mut Token) {
    match id(token) {
        TagId::Html => start_tag_in_body(p, token),
        TagId::Noframes => start_tag_in_head(p, token),
        _ => {}
    }
}

// Foreign content

fn null_character_in_foreign_content(p: &mut Parser, token: &mut Token) {
    if let Token::Character(character) = token {
        character.chars = "\u{FFFD}".to_string();
    }
    insert_characters(p, token);
}

fn character_in_foreign_content(p: &mut Parser, token: &mut Token) {
    insert_characters(p, token);
    p.frameset_ok = false;
}

fn pop_until_html_or_integration_point(p: &mut Parser) {
    while let (Some(current), Some(tag)) = (p.current(), p.current_tag()) {
        if p.dom.namespace(current) == Namespace::Html
            || p.is_integration_point(tag, current, None)
        {
            break;
        }
        p.pop();
    }
}

fn start_tag_in_foreign_content(p: &mut Parser, token: &mut Token) {
    let exits = tag_of(token).is_some_and(causes_exit);
    if exits {
        pop_until_html_or_integration_point(p);
        start_tag_outside_foreign_content(p, token);
        return;
    }
    let namespace = p
        .adjusted_current()
        .map_or(Namespace::Html, |node| p.dom.namespace(node));
    if let Token::StartTag(tag) = token {
        if namespace == Namespace::MathMl {
            adjust_mathml_attributes(tag);
        } else if namespace == Namespace::Svg {
            adjust_svg_tag_name(tag);
            adjust_svg_attributes(tag);
        }
        adjust_xml_attributes(tag);
        if tag.self_closing {
            p.append_element(tag, namespace);
        } else {
            p.insert_element(tag, namespace);
        }
        tag.ack_self_closing = true;
    }
}

fn end_tag_in_foreign_content(p: &mut Parser, token: &mut Token) {
    if matches!(id(token), TagId::P | TagId::Br) {
        pop_until_html_or_integration_point(p);
        end_tag_outside_foreign_content(p, token);
        return;
    }
    for index in (1..p.stack.len()).rev() {
        let element = p.stack[index];
        if p.dom.namespace(element) == Namespace::Html {
            end_tag_outside_foreign_content(p, token);
            break;
        }
        let name = p.dom.tag_name(element).to_string();
        if let Token::EndTag(tag) = token
            && name.to_lowercase() == tag.tag_name
        {
            tag.tag_name = name.clone();
            if let Some(closing) = &mut p.closing {
                closing.end_tag_name = Some(name);
            }
            p.shorten_to(index);
            break;
        }
    }
}
