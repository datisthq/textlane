use textlane_core::mdast::Node;
use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::models::options::{PathRewrite, UrlNode};
use crate::plugins::path_rewrite::actions::url::rewrite::rewrite_url;

impl Plugin for PathRewrite {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for PathRewrite {
    fn input(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn output(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn run<'a>(
        &self,
        tree: Tree<'a>,
        file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        let Tree::Mdast(mut root) = tree else {
            return Err(Error::new("invalid-stage", "`md.pathRewrite` takes mdast"));
        };
        let nodes = match &self.nodes {
            Some(nodes) => nodes.as_slice(),
            None => &[UrlNode::Image, UrlNode::Link, UrlNode::Definition],
        };
        let path = file.path.as_deref();
        let mut stack = root.children_mut().into_iter().collect::<Vec<_>>();
        while let Some(children) = stack.pop() {
            for child in children.iter_mut() {
                let (url, kind) = match child {
                    Node::Image(image) => (&mut image.url, UrlNode::Image),
                    Node::Link(link) => (&mut link.url, UrlNode::Link),
                    Node::Definition(definition) => {
                        (&mut definition.url, UrlNode::Definition)
                    }
                    _ => {
                        if let Some(children) = child.children_mut() {
                            stack.push(children);
                        }
                        continue;
                    }
                };
                // a definition serves links and images alike, so it counts as a link
                let link = kind != UrlNode::Image;
                if nodes.contains(&kind)
                    && let Some(rewritten) = rewrite_url(url, path, self, link)
                {
                    *url = alloc.alloc_str(&rewritten);
                }
                if let Some(children) = child.children_mut() {
                    stack.push(children);
                }
            }
        }
        Ok(Tree::Mdast(root))
    }
}
