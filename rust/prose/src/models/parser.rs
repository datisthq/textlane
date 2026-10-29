use textlane_core::nlcst::Node;
use textlane_core::{Allocator, ArenaVec, Position, ProseParser};

use crate::models::options::Language;
use crate::utils::parser::document::{build_paragraph, build_root};
use crate::utils::parser::store::{Store, read_position};
use crate::utils::parser::tokenize::tokenize_value;

/// The parser `prose.parse` uses, shared with bridges into nlcst through `ProseParserSlot`.
#[derive(Clone, Copy, Debug, Default)]
pub struct LanguageParser {
    pub language: Language,
}

impl LanguageParser {
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    fn is_english(&self) -> bool {
        self.language == Language::English
    }
}

impl ProseParser for LanguageParser {
    fn tokenize<'a>(
        &self,
        value: &'a str,
        offset: usize,
        alloc: &'a Allocator,
    ) -> ArenaVec<'a, Node<'a>> {
        let mut store = Store::default();
        let tokens = tokenize_value(&mut store, value, offset);
        let mut nodes = ArenaVec::with_capacity_in(tokens.len(), alloc);
        for id in tokens {
            nodes.push(store.export(id, alloc));
        }
        nodes
    }

    fn tokenize_paragraph<'a>(
        &self,
        content: ArenaVec<'a, Node<'a>>,
        position: Option<Position<'a>>,
        alloc: &'a Allocator,
    ) -> Node<'a> {
        let mut store = Store::default();
        let content = content.into_iter().map(|node| store.import(node)).collect();
        let paragraph = build_paragraph(
            &mut store,
            content,
            position.map(read_position),
            self.is_english(),
        );
        store.export(paragraph, alloc)
    }

    fn tokenize_root<'a>(
        &self,
        paragraph: Node<'a>,
        position: Option<Position<'a>>,
        alloc: &'a Allocator,
    ) -> Node<'a> {
        let mut store = Store::default();
        let paragraph = store.import(paragraph);
        let root = build_root(&mut store, paragraph, position.map(read_position));
        store.export(root, alloc)
    }
}
