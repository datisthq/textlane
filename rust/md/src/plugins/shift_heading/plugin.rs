use textlane_core::mdast::Node;
use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::models::options::ShiftHeading;

impl Plugin for ShiftHeading {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for ShiftHeading {
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
        _alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        let Tree::Mdast(mut root) = tree else {
            return Err(Error::new("invalid-stage", "`md.shiftHeading` takes mdast"));
        };
        if self.drop_title
            && let Some(children) = root.children_mut()
        {
            drop_title(children);
        }
        if self.shift != 0 {
            let mut stack = root.children_mut().into_iter().collect::<Vec<_>>();
            while let Some(children) = stack.pop() {
                for child in children.iter_mut() {
                    if let Node::Heading(heading) = child {
                        heading.depth = shift_depth(heading.depth, self.shift);
                    }
                    if let Some(children) = child.children_mut() {
                        stack.push(children);
                    }
                }
            }
        }
        Ok(Tree::Mdast(root))
    }
}

// The title is dropped before the shift, so a document opening on an h2 keeps it
fn drop_title(children: &mut textlane_core::ArenaVec<'_, Node<'_>>) {
    let first = children.iter().position(|child| {
        !matches!(
            child,
            Node::Yaml(_) | Node::Toml(_) | Node::Literal(_) | Node::MdxjsEsm(_)
        )
    });
    if let Some(index) = first
        && let Node::Heading(heading) = &children[index]
        && heading.depth == 1
    {
        children.remove(index);
    }
}

fn shift_depth(depth: u8, shift: i32) -> u8 {
    (i32::from(depth).saturating_add(shift)).clamp(1, 6) as u8
}
