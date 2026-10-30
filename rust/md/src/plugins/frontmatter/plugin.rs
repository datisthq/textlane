use textlane_core::mdast::Node;
use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::models::options::{Frontmatter, FrontmatterSlot};
use crate::plugins::frontmatter::actions::matter::read::read_matter;

impl Plugin for Frontmatter {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        let slot = context.slots_mut().get_or_default::<FrontmatterSlot>();
        slot.0.extend(self.matters.iter().cloned());
        if self.data || self.strip {
            context.add_transform(Box::new(self.clone()));
        }
        Ok(())
    }
}

// Reads the yaml node a document starts with, the one vfile-matter's pattern finds
impl Transform for Frontmatter {
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
        _alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        let Tree::Mdast(mut root) = tree else {
            return Err(Error::new("invalid-stage", "`md.frontmatter` takes mdast"));
        };
        let Some(children) = root.children_mut() else {
            return Ok(Tree::Mdast(root));
        };
        let yaml = match children.first() {
            Some(Node::Yaml(yaml)) => Some(yaml.value),
            _ => None,
        };
        if self.data {
            file.data.insert("matter".into(), read_matter(yaml)?);
        }
        if self.strip && yaml.is_some() {
            children.remove(0);
        }
        Ok(Tree::Mdast(root))
    }
}
