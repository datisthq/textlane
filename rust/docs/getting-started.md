---
title: Getting started
description: Add textlane to a Rust project and turn Markdown into HTML.
path: /rust/getting-started/
icon: play
order: 3
---

# Getting started

## Install

```bash
cargo add textlane
```

The crate needs Rust 1.88 or later. Each domain is a feature, all on by default: `md`,
`html`, `highlight` (`html.highlight` and its grammars), `mdx` and `nl`. Leave out what you
do not use to build faster:

```bash
cargo add textlane --no-default-features --features md,html
```

## A first pipeline

A pipeline parses text into a tree, runs plugins over it and serializes the result. Each
plugin is a struct in its domain's module, `textlane::md`, `textlane::html`,
`textlane::mdx` and `textlane::nl`, and its options are the struct's fields.

```rust
use textlane::{Processor, html, md};

let processor = Processor::new()
    .with(md::Parse::default())
    .with(md::ToHtml::default())
    .with(html::Stringify::default())
    .freeze()?;
let output = processor.process("# Hello *world*")?;
output.value
// <h1>Hello <em>world</em></h1>
```

`md::Parse` reads Markdown into mdast, `md::ToHtml` turns it into hast and
`html::Stringify` writes HTML. `freeze` checks that each stage takes the tree the one before
returns, and the frozen processor then handles any number of documents.

## Adding syntax

Syntax extensions sit beside the parser. Add `md::Gfm` for tables, task lists, strikethrough
and autolinks, and `md::Frontmatter` to read a YAML block at the top:

```rust
use textlane::{Processor, html, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Gfm::default())
    .with(md::Frontmatter::default())
    .with(md::ToHtml::default())
    .with(html::Stringify::default())
    .freeze()?
    .process("---\ntitle: Notes\n---\n\n~~draft~~ done")?;
output.value
// <p><del>draft</del> done</p>
```

## Reading the result

`process` returns an `Output`: the text as `value` and the `file`, which holds the
`messages` plugins reported. Checks such as `nl::RepeatedWords` leave the text alone and
report a message:

```rust
use textlane::{Processor, md, nl};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::ToNaturalLanguage::default())
    .with(nl::RepeatedWords)
    .with(nl::Stringify)
    .freeze()?
    .process("It is the the best.")?;
output.file.messages[0].reason
// Unexpected repeated `the`, remove one occurrence
```

Next, the [processor](/rust/processor/) covers every method, and the plugin pages list each
plugin with its options.
