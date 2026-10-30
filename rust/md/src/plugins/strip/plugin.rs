use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::models::options::Strip;
use crate::plugins::strip::actions::list::strip::{Handler, read_handlers, strip_list};

impl Plugin for Strip {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        // strip-markdown fails when it is built, before any document
        read_handlers(self)?;
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for Strip {
    fn input(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn output(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn run<'a>(
        &self,
        tree: Tree<'a>,
        _file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        let Tree::Mdast(mut root) = tree else {
            return Err(Error::new("invalid-stage", "`md.strip` takes mdast"));
        };
        let handlers: Vec<(String, Handler)> = read_handlers(self)?;
        let mut stack = root.children_mut().into_iter().collect::<Vec<_>>();
        while let Some(children) = stack.pop() {
            strip_list(children, &handlers, alloc);
            for child in children.iter_mut() {
                if let Some(children) = child.children_mut() {
                    stack.push(children);
                }
            }
        }
        Ok(Tree::Mdast(root))
    }
}
