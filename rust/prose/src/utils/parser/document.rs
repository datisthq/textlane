use std::collections::VecDeque;

use crate::helpers::classify::{is_new_line, is_terminal_marker};
use crate::utils::parser::paragraph::{
    run_paragraph_plugins, run_root_plugins, split_node,
};
use crate::utils::parser::sentence::run_sentence_plugins;
use crate::utils::parser::store::{Kind, Span, Store};

/// Sentence content into a paragraph, as parse-latin's `tokenizeParagraph` builds one.
pub(crate) fn build_paragraph(
    store: &mut Store<'_>,
    content: VecDeque<usize>,
    position: Option<Span>,
    english: bool,
) -> usize {
    let sentence = store.add_parent(Kind::Sentence, content, position);
    run_sentence_plugins(store, sentence, english);
    let sentences = split_node(store, sentence, Kind::Punctuation, is_terminal_marker);
    let paragraph = store.add_parent(Kind::Paragraph, sentences, position);
    run_paragraph_plugins(store, paragraph, english);
    paragraph
}

/// A paragraph into a root, as parse-latin's `tokenizeRoot` builds one.
pub(crate) fn build_root(
    store: &mut Store<'_>,
    paragraph: usize,
    position: Option<Span>,
) -> usize {
    let paragraphs = split_node(store, paragraph, Kind::WhiteSpace, is_new_line);
    let root = store.add_parent(Kind::Root, paragraphs, position);
    run_root_plugins(store, root);
    root
}
