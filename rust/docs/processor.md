---
title: Processor
description: Build a typed pipeline, freeze it and run each stage on its own, or drive one by name.
path: /rust/processor/
icon: workflow
order: 4
---

# Processor

`Processor::new()` starts a pipeline; `with` adds a plugin and `freeze` returns a
`FrozenProcessor`, which runs documents.

## with

`with` takes any plugin struct. Options are its fields; `..Default::default()` leaves the
rest as the plugin's defaults, which are the upstream plugin's.

```rust
use textlane::{Processor, html, md};

let processor = Processor::new()
    .with(md::Parse::default())
    .with(md::ToHtml { allow_dangerous_html: true, ..Default::default() })
    .with(html::Stringify { allow_dangerous_html: true, ..Default::default() })
    .freeze()?;
let output = processor.process("Hi <kbd>Ctrl</kbd>")?;
output.value
// <p>Hi <kbd>Ctrl</kbd></p>
```

Parsers and compilers apply wherever they are added, transforms in the order they are
added.

## freeze

`freeze` attaches every plugin once and checks that each stage takes the tree the one before
returns: Markdown into mdast, then mdast to hast, then hast to HTML. A pipeline out of order
fails here with `invalid-stage`, before any document runs:

```rust
use textlane::{Processor, html, md};

let error = Processor::new()
    .with(md::Parse::default())
    .with(html::Stringify::default())
    .freeze()
    .err()
    .ok_or("no error")?;
error.code
// invalid-stage
```

A `FrozenProcessor` is immutable, so one can serve many documents and threads.

## Stages

`process` runs all three stages and returns an `Output`: the text as `value`, the output's
file `extension` and the `file`, a `textlane::File` holding the `messages`. Each stage also
runs on its own; trees live in an `Allocator` arena that outlives them:

- `parse(input, &mut file, &alloc)` returns a `Tree`
- `run(tree, &mut file, &alloc)` runs the transforms, finishing asynchronous work;
  `run_sync` fails with `async-in-sync` on it instead
- `stringify(&tree, &mut file)` serializes a tree with no transform

```rust
use textlane::{Allocator, File, Processor, md};

let processor = Processor::new()
    .with(md::Parse::default())
    .with(md::Stringify { bullet: Some("-".into()), ..Default::default() })
    .freeze()?;
let alloc = Allocator::new();
let mut file = File::new();
let tree = processor.parse("* one\n* two", &mut file, &alloc)?;
let tree = processor.run(tree, &mut file, &alloc)?;
processor.stringify(&tree, &mut file)?
// - one
// - two
```

## Pipeline

`textlane::Pipeline` is the processor the Node.js and Python packages drive: plugins by name
with JSON options, and unified's `use`, `data` and `freeze`. Use it when a pipeline comes
from configuration rather than code; its options are `serde_json` values.

```rust
use serde_json::json;
use textlane::{Api, File, Pipeline};

let mut pipeline = Pipeline::new();
pipeline.use_value(&json!("md.parse"), None)?;
pipeline.use_value(&json!("md.gfm"), None)?;
pipeline.use_value(&json!({ "name": "md.toHtml" }), None)?;
pipeline.use_value(&json!("html.stringify"), Some(json!({ "quote": "'" })))?;
pipeline.set_data("site", json!("textlane.dev"))?;
pipeline.process("[a](b)", &mut File::new(), Api::Sync)?
// <p><a href='b'>a</a></p>
```

## Crate functions

- `textlane::read_version()` — the crate's version
- `textlane::list_plugins()` — every plugin's name, as a pipeline names it
- `textlane::read_retained_memory()` and `textlane::set_retained_memory(bytes)` — see
  [Memory](/rust/memory/)
