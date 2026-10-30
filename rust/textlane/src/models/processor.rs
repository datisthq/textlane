use textlane_core::{
    Allocator, Attach, Compiler, Error, File, Parser, Plugin, Step, Transform, Tree,
    TreeKind,
};

use crate::actions::allocator::reuse::with_allocator;

/// A pipeline being composed: `.with()` adds a plugin, as unified's `.use()` does.
#[derive(Default)]
pub struct Processor {
    plugins: Vec<Box<dyn Plugin>>,
}

/// A processor whose plugins are attached: it runs any number of documents, in parallel.
pub struct FrozenProcessor {
    parser: Option<Box<dyn Parser>>,
    transforms: Vec<Box<dyn Transform>>,
    compiler: Option<Box<dyn Compiler>>,
}

/// What processing one document produced: the output and the kind of text it is.
#[derive(Debug)]
pub struct Output {
    pub value: String,
    pub extension: &'static str,
    pub file: File,
}

impl Processor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a plugin; parsers and compilers apply wherever they are, transforms in order.
    pub fn with(mut self, plugin: impl Plugin + 'static) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    pub(crate) fn with_boxed(mut self, plugin: Box<dyn Plugin>) -> Self {
        self.plugins.push(plugin);
        self
    }

    /// Attach every plugin once and check that each stage takes the tree the one before
    /// returns.
    pub fn freeze(self) -> Result<FrozenProcessor, Error> {
        self.freeze_with(|_| {}, |_| {})
    }

    // `prepare` runs before the first plugin attaches and `collect` after the last, so a
    // caller can seed and read back the slots plugins share
    pub(crate) fn freeze_with(
        self,
        prepare: impl FnOnce(&mut Attach),
        collect: impl FnOnce(&mut Attach),
    ) -> Result<FrozenProcessor, Error> {
        let mut context = Attach::default();
        seed_slots(&mut context);
        prepare(&mut context);
        for plugin in &self.plugins {
            plugin.attach(&mut context)?;
        }
        collect(&mut context);
        let (parser, transforms, compiler) = context.finish()?;
        let mut kind = parser.as_ref().map(|parser| parser.output());
        for transform in &transforms {
            check_kind(kind, transform.input())?;
            kind = Some(transform.output());
        }
        if let Some(compiler) = &compiler {
            check_kind(kind, compiler.input())?;
        }
        Ok(FrozenProcessor {
            parser,
            transforms,
            compiler,
        })
    }
}

impl FrozenProcessor {
    /// Parse, run and stringify one document.
    pub fn process(&self, input: &str) -> Result<Output, Error> {
        let mut file = File::new();
        let value = with_allocator(|alloc| {
            let tree = self.parse(input, &mut file, alloc)?;
            let tree = self.run(tree, &mut file, alloc)?;
            self.stringify(&tree, &mut file)
        })?;
        Ok(Output {
            value,
            extension: self.extension().unwrap_or_default(),
            file,
        })
    }

    pub fn parse<'a>(
        &self,
        input: &'a str,
        file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        match &self.parser {
            Some(parser) => {
                let source = parser.source(input);
                file.value = Some(source.to_string());
                parser.parse(source, file, alloc)
            }
            None => Err(Error::new("missing-parser", "the processor has no parser")),
        }
    }

    /// Run every transform over a tree, in order, finishing the jobs asynchronous ones
    /// hand back.
    pub fn run<'a>(
        &self,
        mut tree: Tree<'a>,
        file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        for transform in &self.transforms {
            tree = match transform.start(tree, file, alloc)? {
                Step::Done(tree) => tree,
                Step::Job(job) => job(file)?,
            };
        }
        Ok(tree)
    }

    /// Run every transform over a tree, failing with `async-in-sync` when one hands back a
    /// job.
    pub fn run_sync<'a>(
        &self,
        mut tree: Tree<'a>,
        file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        for transform in &self.transforms {
            tree = match transform.start(tree, file, alloc)? {
                Step::Done(tree) => tree,
                Step::Job(_) => {
                    return Err(Error::new("async-in-sync", "a stage finished async"));
                }
            };
        }
        Ok(tree)
    }

    pub fn stringify(&self, tree: &Tree<'_>, file: &mut File) -> Result<String, Error> {
        match &self.compiler {
            Some(compiler) => compiler.compile(tree, file),
            None => Err(Error::new(
                "missing-compiler",
                "the processor has no compiler",
            )),
        }
    }

    /// The part of `input` the parser reads, which positions count from.
    pub fn source<'a>(&self, input: &'a str) -> &'a str {
        match &self.parser {
            Some(parser) => parser.source(input),
            None => input,
        }
    }

    pub fn has_parser(&self) -> bool {
        self.parser.is_some()
    }

    pub fn has_compiler(&self) -> bool {
        self.compiler.is_some()
    }

    /// The kind of text the compiler writes (`html`, `md`), when there is one.
    pub fn extension(&self) -> Option<&'static str> {
        self.compiler.as_ref().map(|compiler| compiler.extension())
    }

    /// The kind of tree the compiler takes, when there is one.
    pub fn compiler_kind(&self) -> Option<TreeKind> {
        self.compiler.as_ref().map(|compiler| compiler.input())
    }

    /// The kind of tree the first stage after parsing takes.
    pub fn input_kind(&self) -> Option<TreeKind> {
        self.transforms
            .first()
            .map(|transform| transform.input())
            .or_else(|| self.compiler.as_ref().map(|compiler| compiler.input()))
    }
}

// Bridges into nlcst live in md and html, which cannot depend on nl, so the parser they
// share reaches them through a slot
fn seed_slots(context: &mut Attach) {
    #[cfg(feature = "nl")]
    {
        let parser = textlane_nl::LanguageParser::new(textlane_nl::Language::Latin);
        context
            .slots_mut()
            .get_or_default::<textlane_core::NaturalLanguageParserSlot>()
            .parser = Some(std::sync::Arc::new(parser));
    }
    #[cfg(feature = "mdx")]
    {
        context
            .slots_mut()
            .get_or_default::<textlane_core::ScriptParserSlot>()
            .parser = Some(std::sync::Arc::new(textlane_mdx::OxcScriptParser));
    }
    #[cfg(not(any(feature = "nl", feature = "mdx")))]
    let _ = context;
}

fn check_kind(actual: Option<TreeKind>, expected: TreeKind) -> Result<(), Error> {
    match actual {
        Some(actual) if actual != expected => Err(Error::new(
            "invalid-stage",
            format!("a stage takes {expected:?} but gets {actual:?}"),
        )),
        _ => Ok(()),
    }
}

#[cfg(all(test, feature = "md", feature = "html"))]
#[path = "processor_unit.rs"]
mod unit;
