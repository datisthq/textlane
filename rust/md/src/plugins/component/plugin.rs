use textlane_core::mdast::{Code, Node, Text};
use textlane_core::{
    Allocator, ArenaVec, Attach, Error, File, Plugin, Transform, Tree, TreeKind,
};

use crate::models::options::{Component, UnknownDirectives};
use crate::plugins::component::actions::alert::convert::convert_alert;
use crate::plugins::component::actions::children::group::{
    Mark, group_children, read_group,
};
use crate::plugins::component::actions::code::convert::{convert_code, matches_code};
use crate::plugins::component::actions::directive::convert::{
    DirectiveParts, convert_directive,
};
use crate::plugins::component::actions::directive::restore::restore_directive;
use crate::plugins::component::actions::node::convert::convert_node;

impl Plugin for Component {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for Component {
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
            return Err(Error::new("invalid-stage", "`md.component` takes mdast"));
        };
        let source = file.value.as_deref();
        // Parents convert before their children, so a sibling list groups once all of its
        // own items are converted, and nesting never recurses
        let mut stack = root.children_mut().into_iter().collect::<Vec<_>>();
        while let Some(children) = stack.pop() {
            self.convert_children(children, source, alloc);
            for child in children.iter_mut() {
                if let Some(children) = child.children_mut() {
                    stack.push(children);
                }
            }
        }
        Ok(Tree::Mdast(root))
    }
}

impl Component {
    fn convert_children<'a>(
        &self,
        children: &mut ArenaVec<'a, Node<'a>>,
        source: Option<&str>,
        alloc: &'a Allocator,
    ) {
        let mut marks = Vec::with_capacity(children.len());
        for child in children.iter_mut() {
            let node = std::mem::replace(child, placeholder());
            let (node, mark) = self.convert_child(node, source, alloc);
            *child = node;
            marks.push(mark);
        }
        if marks
            .iter()
            .flatten()
            .any(|&mark| read_group(self, mark).is_some())
        {
            group_children(self, children, &marks, alloc);
        }
    }

    // A converted directive or a matched code block reports its rule, which grouping reads
    fn convert_child<'a>(
        &self,
        node: Node<'a>,
        source: Option<&str>,
        alloc: &'a Allocator,
    ) -> (Node<'a>, Option<Mark>) {
        let node = match node {
            Node::Code(code) => return self.convert_code(code, alloc),
            node => node,
        };
        let mut parts = match DirectiveParts::take(node) {
            Ok(parts) => parts,
            Err(node) => return (self.convert_other(node, alloc), None),
        };
        for (index, rule) in self.directives.iter().enumerate() {
            match convert_directive(rule, parts, alloc) {
                Ok(node) => return (node, Some(Mark::Directive(index))),
                Err(given) => parts = given,
            }
        }
        match self.unknown_directives {
            UnknownDirectives::Keep => (parts.into_node(), None),
            UnknownDirectives::Source => (restore_directive(parts, source, alloc), None),
        }
    }

    fn convert_code<'a>(
        &self,
        code: Code<'a>,
        alloc: &'a Allocator,
    ) -> (Node<'a>, Option<Mark>) {
        let Some(index) = self.codes.iter().position(|rule| matches_code(rule, &code))
        else {
            return (Node::Code(code), None);
        };
        let rule = &self.codes[index];
        let node = match &rule.component {
            Some(component) => convert_code(rule, component, code, alloc),
            None => Node::Code(code),
        };
        (node, Some(Mark::Code(index)))
    }

    fn convert_other<'a>(&self, node: Node<'a>, alloc: &'a Allocator) -> Node<'a> {
        let node = match (&self.alerts, node) {
            (Some(rule), Node::Blockquote(quote)) => convert_alert(rule, quote, alloc),
            (_, node) => node,
        };
        convert_node(&self.nodes, node, alloc)
    }
}

fn placeholder<'a>() -> Node<'a> {
    Node::Text(Text {
        value: "",
        position: None,
        data: None,
    })
}
