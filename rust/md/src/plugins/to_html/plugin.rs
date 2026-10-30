use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use textlane_core::{
    Allocator, Attach, Error, File, HtmlTarget, HtmlWriter, Plugin, Transform, Tree,
    TreeKind,
};

use crate::actions::mdast::convert::{convert_mdast, convert_mdast_with};
use crate::actions::mdast::write::write_html;
use crate::models::options::{ToHtml, UnravelSlot};

impl Plugin for ToHtml {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        let unravel = context
            .slots_mut()
            .get_or_default::<UnravelSlot>()
            .0
            .clone();
        context.add_transform(Box::new(Converting {
            options: self.clone(),
            unravel,
        }));
        Ok(())
    }
}

impl Transform for ToHtml {
    fn input(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn output(&self) -> TreeKind {
        TreeKind::Hast
    }

    fn run<'a>(
        &self,
        tree: Tree<'a>,
        _file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        match tree {
            Tree::Mdast(node) => Ok(Tree::Hast(convert_mdast(&node, alloc, self))),
            _ => Err(Error::new("invalid-stage", "`md.toHtml` takes mdast")),
        }
    }
}

// The stage a pipeline runs: with `md::Mdx` in it, @mdx-js/mdx's remark-mark-and-unravel
// first lifts MDX out of paragraphs that hold nothing else
struct Converting {
    options: ToHtml,
    unravel: Arc<AtomicBool>,
}

impl Transform for Converting {
    fn input(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn output(&self) -> TreeKind {
        TreeKind::Hast
    }

    fn run<'a>(
        &self,
        tree: Tree<'a>,
        _file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        let unravel = self.unravel.load(Ordering::Relaxed);
        match tree {
            Tree::Mdast(node) => Ok(Tree::Hast(convert_mdast_with(
                &node,
                alloc,
                &self.options,
                unravel,
            ))),
            _ => Err(Error::new("invalid-stage", "`md.toHtml` takes mdast")),
        }
    }

    fn html_writer(&self, target: HtmlTarget) -> Option<Box<dyn HtmlWriter>> {
        Some(Box::new(Writing {
            options: self.options.clone(),
            target,
        }))
    }
}

// What `html_writer` hands out: this stage and an HTML compiler after it, in one pass
struct Writing {
    options: ToHtml,
    target: HtmlTarget,
}

impl HtmlWriter for Writing {
    fn write(&self, tree: &Tree<'_>) -> Option<String> {
        match tree {
            Tree::Mdast(node) => {
                write_html(node, &self.options, self.target.allow_dangerous_html)
            }
            _ => None,
        }
    }
}
