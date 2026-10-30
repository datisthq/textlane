use regex::Regex;
use textlane_core::mdast::Node;
use textlane_core::{
    Allocator, ArenaVec, Attach, Error, File, Plugin, Transform, Tree, TreeKind,
};

use crate::models::options::{ParentCheck, ParentTest, Toc};
use crate::plugins::toc::actions::contents::build::{ContentsSettings, build_contents};
use crate::plugins::toc::actions::expression::compile::compile_expression;
use crate::plugins::toc::actions::heading::search::{SearchSettings, search_headings};
use crate::plugins::toc::actions::node::copy::{CopyMode, copy_nodes};

const DEFAULT_HEADING: &str = "(table[ -]of[ -])?contents?|toc";

impl Plugin for Toc {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        let heading = match self.heading.as_deref() {
            Some(heading) if !heading.is_empty() => heading,
            _ => DEFAULT_HEADING,
        };
        let skip = match self.skip.as_deref() {
            Some(skip) if !skip.is_empty() => Some(compile_expression(skip)?),
            _ => None,
        };
        // unified-style falsy: an empty string is no test at all
        let parents = match &self.parents {
            Some(ParentTest::One(ParentCheck::Kind(kind))) if kind.is_empty() => None,
            parents => parents.clone(),
        };
        context.add_transform(Box::new(TocTransform {
            heading: compile_expression(heading)?,
            skip,
            parents,
            min_depth: self.min_depth.unwrap_or(0),
            max_depth: self.max_depth.unwrap_or(0),
            tight: self.tight.unwrap_or(true),
            ordered: self.ordered.unwrap_or(false),
            prefix: self.prefix.clone().unwrap_or_default(),
        }));
        Ok(())
    }
}

struct TocTransform {
    heading: Regex,
    skip: Option<Regex>,
    parents: Option<ParentTest>,
    min_depth: u8,
    max_depth: u8,
    tight: bool,
    ordered: bool,
    prefix: String,
}

impl Transform for TocTransform {
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
            return Err(Error::new("invalid-stage", "`md.toc` takes mdast"));
        };
        let settings = SearchSettings {
            heading: &self.heading,
            skip: self.skip.as_ref(),
            parents: self.parents.as_ref(),
            min_depth: self.min_depth,
            max_depth: self.max_depth,
        };
        let search = search_headings(&root, &settings, alloc);
        let Some(index) = search.index else {
            return Ok(Tree::Mdast(root));
        };
        if search.map.is_empty() {
            return Ok(Tree::Mdast(root));
        }
        let contents = ContentsSettings {
            ordered: self.ordered,
            tight: self.tight,
            prefix: &self.prefix,
        };
        let table = build_contents(search.map, &contents, alloc);
        if let Some(children) = root.children_mut() {
            splice_table(children, index, search.end_index, table, alloc);
        }
        Ok(Tree::Mdast(root))
    }
}

// remark-toc's `[...slice(0, index), map, ...slice(endIndex)]`; with `parents`, the closing
// heading can come before the opening's index, and the children between appear twice
fn splice_table<'a>(
    children: &mut ArenaVec<'a, Node<'a>>,
    index: usize,
    end_index: usize,
    table: Node<'a>,
    alloc: &'a Allocator,
) {
    let length = children.len();
    let index = index.min(length);
    let end_index = end_index.min(length);
    let repeated = if end_index < index {
        copy_nodes(&children[end_index..index], CopyMode::Whole, alloc)
    } else {
        ArenaVec::new_in(alloc)
    };
    let old = std::mem::replace(children, ArenaVec::new_in(alloc));
    let mut spliced = ArenaVec::with_capacity_in(
        index + 1 + repeated.len() + length - end_index,
        alloc,
    );
    let mut old = old.into_iter();
    spliced.extend(old.by_ref().take(index));
    spliced.push(table);
    spliced.extend(repeated);
    spliced.extend(old.skip(end_index.saturating_sub(index)));
    *children = spliced;
}

#[cfg(test)]
#[path = "plugin_unit.rs"]
mod unit;
