---
title: Syntax trees
description: The mdast, hast and nlcst trees a pipeline passes between its stages.
path: /rust/syntax-trees/
icon: network
order: 10
---

# Syntax trees

A pipeline passes one of three trees between its stages, each a [unist](https://github.com/syntax-tree/unist)
tree as the unified ecosystem defines it:

- [mdast](https://github.com/syntax-tree/mdast) — Markdown, from `md::Parse`, in
  `textlane::mdast`
- [hast](https://github.com/syntax-tree/hast) — HTML, from `html::Parse` and `md::ToHtml`, in
  `textlane::hast`
- [nlcst](https://github.com/syntax-tree/nlcst) — natural language, from `nl::Parse`,
  `md::ToNaturalLanguage` and `html::ToNaturalLanguage`, in `textlane::nlcst`

`Tree` is one of the three. Each module's `Node` is an enum with a variant per node type,
whose struct holds the fields unist names: `children`, `value`, `depth` and the rest. Nodes
live in an `Allocator` arena, so a tree borrows from it and from the text it was parsed from.

```rust
use textlane::{Allocator, File, Processor, Tree, md, mdast};

let processor = Processor::new().with(md::Parse::default()).freeze()?;
let alloc = Allocator::new();
let mut file = File::new();
let tree = processor.parse("# Hi *there*", &mut file, &alloc)?;
let Tree::Mdast(mdast::Node::Root(root)) = &tree else { return Err("no root".into()) };
let Some(mdast::Node::Heading(heading)) = root.children.first() else {
    return Err("no heading".into());
};
heading.depth
// 1
```

The full set of nodes and their fields is on [docs.rs](https://docs.rs/textlane).

## Positions

A node's `position` points into the source by byte offset. `Lines` turns it into unist's
1-indexed lines and columns:

```rust
use textlane::{Allocator, File, Lines, Processor, Tree, md, mdast};

let processor = Processor::new().with(md::Parse::default()).freeze()?;
let alloc = Allocator::new();
let mut file = File::new();
let input = "# Hi *there*";
let tree = processor.parse(input, &mut file, &alloc)?;
let Tree::Mdast(mdast::Node::Root(root)) = &tree else { return Err("no root".into()) };
let position = root.position.as_ref().ok_or("no position")?;
let end = Lines::new(processor.source(input)).to_unist_position(position).end;
format!("{}:{}", end.line, end.column)
// 1:13
```

## As JSON

`serialize_mdast`, `serialize_hast` and `serialize_nlcst` write a tree as the JSON unified's
ecosystem uses, and `deserialize_*` read it back:

```rust
use textlane::{Allocator, File, Processor, Tree, md, mdast};

let processor = Processor::new().with(md::Parse::default()).freeze()?;
let alloc = Allocator::new();
let mut file = File::new();
let tree = processor.parse("*hi*", &mut file, &alloc)?;
let Tree::Mdast(root) = &tree else { return Err("not mdast".into()) };
let json = mdast::serialize_mdast(root, None)?;
json["children"][0]["children"][0]["type"]
// "emphasis"
```
