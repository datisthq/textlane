use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::models::error::Error;
use crate::models::file::File;
use crate::models::nlcst::Node;
use crate::models::position::Position;
use crate::models::tree::{Tree, TreeKind};
use crate::{Allocator, ArenaVec};

/// A pipeline stage: attached once when a processor freezes, where it registers what it does.
pub trait Plugin: Send + Sync {
    fn attach(&self, context: &mut Attach) -> Result<(), Error>;
}

/// Turns input text into a tree.
pub trait Parser: Send + Sync {
    fn output(&self) -> TreeKind;

    /// The part of `input` the parser reads and positions count from: all of it by default.
    fn source<'a>(&self, input: &'a str) -> &'a str {
        input
    }

    fn parse<'a>(
        &self,
        input: &'a str,
        file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error>;
}

/// Changes a tree, or bridges it into another kind of tree.
pub trait Transform: Send + Sync {
    fn input(&self) -> TreeKind;
    fn output(&self) -> TreeKind;
    fn run<'a>(
        &self,
        tree: Tree<'a>,
        file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error>;

    /// Start the stage: a finished tree, or a job when its work is asynchronous.
    fn start<'a>(
        &self,
        tree: Tree<'a>,
        file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Step<'a>, Error> {
        self.run(tree, file, alloc).map(Step::Done)
    }

    /// The id of the binding function this stage stands for, which takes and returns
    /// whatever tree reaches it; `None` for a stage the crate runs.
    fn host(&self) -> Option<u32> {
        None
    }
}

/// Deferred work a stage hands back, finished only by the asynchronous API.
pub type Job<'a> = Box<dyn FnOnce(&mut File) -> Result<Tree<'a>, Error> + 'a>;

/// What starting a stage gives: its tree, or a job that yields it.
pub enum Step<'a> {
    Done(Tree<'a>),
    Job(Job<'a>),
}

/// Turns a tree into output text, whose kind `extension` names (`html`, `md`).
pub trait Compiler: Send + Sync {
    fn input(&self) -> TreeKind;
    fn extension(&self) -> &'static str;
    fn compile(&self, tree: &Tree<'_>, file: &mut File) -> Result<String, Error>;
}

/// A natural-language parser, as parse-latin's instance is to mdast-util-to-nlcst and
/// hast-util-to-nlcst: bridges into nlcst tokenize text with it and run its plugins.
pub trait NaturalLanguageParser: Send + Sync {
    /// Split `value` into sentence content: words, white space, punctuation and symbols, each
    /// spanning its bytes from `offset` on.
    fn tokenize<'a>(
        &self,
        value: &'a str,
        offset: usize,
        alloc: &'a Allocator,
    ) -> ArenaVec<'a, Node<'a>>;

    /// Run the sentence plugins over `content` as one sentence, split it into sentences after
    /// terminal markers and run the paragraph plugins: a `ParagraphNode`. The sentence and the
    /// paragraph start at `position`; without one, at their children's.
    fn tokenize_paragraph<'a>(
        &self,
        content: ArenaVec<'a, Node<'a>>,
        position: Option<Position<'a>>,
        alloc: &'a Allocator,
    ) -> Node<'a>;

    /// Split a paragraph into paragraphs at white space holding a line ending and run the
    /// root plugins: a `RootNode`, at `position` or its children's.
    fn tokenize_root<'a>(
        &self,
        paragraph: Node<'a>,
        position: Option<Position<'a>>,
        alloc: &'a Allocator,
    ) -> Node<'a>;
}

/// The slot holding the natural-language parser bridges into nlcst read when their stage is built; the
/// textlane processor seeds it with `nl.parse`'s Latin parser before plugins attach.
#[derive(Clone, Default)]
pub struct NaturalLanguageParserSlot {
    pub parser: Option<Arc<dyn NaturalLanguageParser>>,
}

/// A JavaScript parser, as acorn is to micromark's MDX extensions: `md.mdx` asks it where
/// an expression or a block of imports and exports is valid, and keeps only the source.
pub trait ScriptParser: Send + Sync {
    /// Parse `source` whole as one expression (JSX included), comments and white space
    /// around it allowed, and describe it.
    fn parse_expression(&self, source: &str) -> Result<ScriptExpression, ScriptError>;

    /// Parse `source` as a module and list its top-level statements.
    fn parse_module(&self, source: &str) -> Result<Vec<ScriptStatement>, ScriptError>;
}

/// What an expression is, as far as MDX checks it: the byte offset where it ends, before
/// any comments after it, and, for an object literal (parentheses dropped), the ESTree type
/// and byte offset of each property.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptExpression {
    pub end: usize,
    pub object: Option<Vec<(String, usize)>>,
}

/// A top-level statement of a module: its ESTree type, where it starts and, for an
/// import, the local names it declares.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptStatement {
    pub kind: String,
    pub offset: usize,
    pub imports: Vec<String>,
}

/// A syntax error: the byte span of what the parser stopped at, which ends at the end of
/// the source when more source might still make it valid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptError {
    pub start: usize,
    pub end: usize,
    pub reason: String,
}

/// The slot holding the script parser `md.mdx` validates JavaScript with; the textlane
/// processor seeds it when built with `mdx`. Without one, expressions end at the brace
/// that balances their opening one and imports and exports at a blank line.
#[derive(Clone, Default)]
pub struct ScriptParserSlot {
    pub parser: Option<Arc<dyn ScriptParser>>,
}

type ParserFactory = Box<dyn FnOnce(&Slots) -> Result<Box<dyn Parser>, Error>>;
type CompilerFactory = Box<dyn FnOnce(&Slots) -> Result<Box<dyn Compiler>, Error>>;

/// What plugins register while a processor freezes.
#[derive(Default)]
pub struct Attach {
    parser: Option<ParserFactory>,
    compiler: Option<CompilerFactory>,
    transforms: Vec<Box<dyn Transform>>,
    slots: Slots,
}

/// Data plugins share while attaching, one value per type.
#[derive(Default)]
pub struct Slots {
    values: HashMap<TypeId, Box<dyn Any>>,
}

impl Attach {
    /// Set the parser, built once every plugin has attached, so it sees all their slots.
    pub fn set_parser(
        &mut self,
        factory: impl FnOnce(&Slots) -> Result<Box<dyn Parser>, Error> + 'static,
    ) {
        self.parser = Some(Box::new(factory));
    }

    /// Set the compiler, built once every plugin has attached.
    pub fn set_compiler(
        &mut self,
        factory: impl FnOnce(&Slots) -> Result<Box<dyn Compiler>, Error> + 'static,
    ) {
        self.compiler = Some(Box::new(factory));
    }

    pub fn add_transform(&mut self, transform: Box<dyn Transform>) {
        self.transforms.push(transform);
    }

    pub fn slots_mut(&mut self) -> &mut Slots {
        &mut self.slots
    }

    /// Build the parser and compiler and hand back every stage.
    #[allow(clippy::type_complexity)]
    pub fn finish(
        self,
    ) -> Result<
        (
            Option<Box<dyn Parser>>,
            Vec<Box<dyn Transform>>,
            Option<Box<dyn Compiler>>,
        ),
        Error,
    > {
        let parser = self
            .parser
            .map(|factory| factory(&self.slots))
            .transpose()?;
        let compiler = self
            .compiler
            .map(|factory| factory(&self.slots))
            .transpose()?;
        Ok((parser, self.transforms, compiler))
    }
}

impl Slots {
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.values
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref())
    }

    pub fn get_or_default<T: Default + 'static>(&mut self) -> &mut T {
        let value = self
            .values
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(T::default()));
        match value.downcast_mut() {
            Some(value) => value,
            None => unreachable!("a slot holds the type it is keyed by"),
        }
    }
}
