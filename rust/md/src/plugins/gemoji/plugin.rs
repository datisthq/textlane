use textlane_core::mdast::Node;
use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::plugins::gemoji::actions::text::replace::replace_shortcodes;

/// `md.gemoji`: gemoji shortcodes, `:smile:`, into emoji.
#[derive(Clone, Debug, Default)]
pub struct Gemoji;

/// The `md.gemoji` plugin.
pub fn gemoji() -> Gemoji {
    Gemoji
}

impl Plugin for Gemoji {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(Gemoji));
        Ok(())
    }
}

impl Transform for Gemoji {
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
            return Err(Error::new("invalid-stage", "`md.gemoji` takes mdast"));
        };
        // Like remark-gemoji, text that is the whole tree has no parent to be replaced in
        let mut stack = root.children_mut().into_iter().collect::<Vec<_>>();
        while let Some(children) = stack.pop() {
            for child in children.iter_mut() {
                if let Node::Text(text) = child {
                    if let Some(value) = replace_shortcodes(text.value) {
                        text.value = alloc.alloc_str(&value);
                        // upstream replaces the node with a new one, which has no position
                        text.position = None;
                    }
                } else if let Some(children) = child.children_mut() {
                    stack.push(children);
                }
            }
        }
        Ok(Tree::Mdast(root))
    }
}
