use textlane_core::mdast::{Node, Text};
use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::models::options::PackageCommand;
use crate::plugins::package_command::actions::code::convert::convert_code;

impl Plugin for PackageCommand {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for PackageCommand {
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
            return Err(Error::new(
                "invalid-stage",
                "`md.packageCommand` takes mdast",
            ));
        };
        let mut stack = root.children_mut().into_iter().collect::<Vec<_>>();
        while let Some(children) = stack.pop() {
            for child in children.iter_mut() {
                *child = match std::mem::replace(child, placeholder()) {
                    Node::Code(code) => convert_code(self, code, alloc),
                    node => node,
                };
                if let Some(children) = child.children_mut() {
                    stack.push(children);
                }
            }
        }
        Ok(Tree::Mdast(root))
    }
}

fn placeholder<'a>() -> Node<'a> {
    Node::Text(Text {
        value: "",
        position: None,
        data: None,
    })
}
