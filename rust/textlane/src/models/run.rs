use std::sync::Arc;

use serde_json::Value;
use textlane_core::{Allocator, Error, File, Lines, Tree, TreeKind};

use crate::actions::allocator::reuse::with_allocator;
use crate::models::pipeline::{Api, read_tree, write_tree};
use crate::models::processor::{Advance, FrozenProcessor};

/// How far a run of a pipeline got: to its end, or to stages a binding must run.
pub enum Progress {
    Finished(Finished),
    Paused(Pause),
}

/// A finished run: what it produced and the file plugins reported against.
pub struct Finished {
    pub output: Outcome,
    pub file: File,
}

/// What a run produced: the tree, from `start_run`, or the text, from `start_process`.
pub enum Outcome {
    Tree(Value),
    Text(String),
}

/// A run stopped before consecutive host stages: the binding runs each of `hosts` over
/// `tree`, in order, and hands the result to `resume`.
pub struct Pause {
    pub(crate) processor: Arc<FrozenProcessor>,
    pub(crate) hosts: Vec<u32>,
    pub(crate) tree: Value,
    pub(crate) kind: TreeKind,
    pub(crate) next: usize,
    pub(crate) file: File,
    pub(crate) api: Api,
    pub(crate) compile: bool,
}

impl Pause {
    /// The ids of the host stages to run, in order.
    pub fn hosts(&self) -> &[u32] {
        &self.hosts
    }

    /// The tree the first host stage takes, as JSON, positioned in the file's text.
    pub fn tree(&self) -> &Value {
        &self.tree
    }

    /// The file so far, its `value` the text the tree's positions point into.
    pub fn file(&self) -> &File {
        &self.file
    }

    /// The file so far; host stages add their messages to it.
    pub fn file_mut(&mut self) -> &mut File {
        &mut self.file
    }

    /// Continue the run with the tree the host stages returned.
    pub fn resume(self, tree: &Value) -> Result<Progress, Error> {
        let Pause {
            processor,
            kind,
            next,
            mut file,
            api,
            compile,
            ..
        } = self;
        with_allocator(|alloc| {
            let tree = read_tree(tree, Some(kind), alloc)?;
            let run = Run {
                processor: &processor,
                api,
                compile,
            };
            let step = run.advance(tree, &mut file, alloc, next)?;
            Ok(run.progress(step, file))
        })
    }
}

// What `advance` hands back before the file it borrowed is moved into the progress
pub(crate) enum Step {
    Finished(Outcome),
    Paused {
        hosts: Vec<u32>,
        tree: Value,
        kind: TreeKind,
        next: usize,
    },
}

/// One run's settings, shared by its start and every resume.
pub(crate) struct Run<'p> {
    pub(crate) processor: &'p Arc<FrozenProcessor>,
    pub(crate) api: Api,
    pub(crate) compile: bool,
}

impl Run<'_> {
    /// Run from the stage at `from` until the end, compiling when the run is a process, or
    /// until host stages.
    pub(crate) fn advance<'a>(
        &self,
        tree: Tree<'a>,
        file: &mut File,
        alloc: &'a Allocator,
        from: usize,
    ) -> Result<Step, Error> {
        let advanced = self
            .processor
            .advance(tree, file, alloc, from, self.api)
            .map_err(|error| self.rename_async(error))?;
        match advanced {
            Advance::Done(tree) if self.compile => self
                .processor
                .stringify(&tree, file)
                .map(|text| Step::Finished(Outcome::Text(text))),
            Advance::Done(tree) => {
                let lines = file.value.as_deref().map(Lines::new);
                write_tree(&tree, lines.as_ref())
                    .map(|tree| Step::Finished(Outcome::Tree(tree)))
            }
            Advance::Host { hosts, next, tree } => {
                let lines = file.value.as_deref().map(Lines::new);
                Ok(Step::Paused {
                    hosts,
                    kind: tree.kind(),
                    tree: write_tree(&tree, lines.as_ref())?,
                    next,
                })
            }
        }
    }

    /// The progress a step makes, given the file back.
    pub(crate) fn progress(&self, step: Step, file: File) -> Progress {
        match step {
            Step::Finished(output) => Progress::Finished(Finished { output, file }),
            Step::Paused {
                hosts,
                tree,
                kind,
                next,
            } => Progress::Paused(Pause {
                processor: Arc::clone(self.processor),
                hosts,
                tree,
                kind,
                next,
                file,
                api: self.api,
                compile: self.compile,
            }),
        }
    }

    fn rename_async(&self, error: Error) -> Error {
        if error.code != "async-in-sync" {
            return error;
        }
        let call = if self.compile { "process" } else { "run" };
        let message = format!("`{call}Sync` finished async. Use `{call}` instead");
        Error::new(error.code, message)
    }
}

#[cfg(all(test, feature = "md", feature = "html"))]
#[path = "run_unit.rs"]
mod unit;
