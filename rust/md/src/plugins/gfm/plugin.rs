use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::models::options::{Alerts, AlertsOptions, Gfm, GfmSlot};
use crate::plugins::gfm::actions::blockquote::alert::mark_alerts;

impl Plugin for Gfm {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.slots_mut().get_or_default::<GfmSlot>().0 = Some(self.clone());
        match &self.alerts {
            Some(Alerts::Enabled(true)) => {
                context.add_transform(Box::new(Alerting(AlertsOptions::default())));
            }
            Some(Alerts::Options(options)) => {
                context.add_transform(Box::new(Alerting(options.clone())));
            }
            Some(Alerts::Enabled(false)) | None => {}
        }
        Ok(())
    }
}

// remark-github-blockquote-alert, run where `md.gfm` stands in the pipeline
struct Alerting(AlertsOptions);

impl Transform for Alerting {
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
        match tree {
            Tree::Mdast(mut node) => {
                mark_alerts(&mut node, &self.0, alloc);
                Ok(Tree::Mdast(node))
            }
            _ => Err(Error::new("invalid-stage", "`md.gfm` takes mdast")),
        }
    }
}
