use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

/// A stage a binding runs: a JavaScript or Python plugin, known to the crate by its id.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Host {
    pub(crate) id: u32,
}

impl Plugin for Host {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(*self));
        Ok(())
    }
}

// `input` and `output` are never checked: a host stage takes whatever tree reaches it
impl Transform for Host {
    fn input(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn output(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn run<'a>(
        &self,
        _tree: Tree<'a>,
        _file: &mut File,
        _alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        Err(missing_host(self.id))
    }

    fn host(&self) -> Option<u32> {
        Some(self.id)
    }
}

/// The error of a run that reaches a host stage with no binding to hand it to.
pub(crate) fn missing_host(id: u32) -> Error {
    Error::new(
        "missing-host",
        format!("Cannot run host plugin `{id}` without a binding to run it"),
    )
}
