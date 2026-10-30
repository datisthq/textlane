---
name: textlane-usage
description: Use when installing textlane or calling it from Rust, Node.js or Python — which package each language uses, how to build a pipeline of native plugins, the processor API and what it does not offer yet.
---

# Textlane Usage

Textlane is a Rust-native text processing pipeline: a replacement for unified whose plugins run
natively, with Node.js and Python bindings over the same crate. It is pre-release.

## When to use this skill

Invoke when the user asks to:

- Install textlane in a Rust, Node.js or Python project.
- Turn Markdown into HTML (or HTML into Markdown, MDX into JavaScript, natural language into checks) with
  textlane, or port a unified, remark, rehype or retext pipeline to it.
- Check which textlane version is installed.

## Install

| Language | Package                | Install                |
| -------- | ---------------------- | ---------------------- |
| Rust     | `textlane` (crates.io) | `cargo add textlane`   |
| Node.js  | `textlane` (npm)       | `npm install textlane` |
| Python   | `textlane` (PyPI)      | `pip install textlane` |

Every channel ships the same crate at the same version. npm installs one prebuilt addon for the
current platform as an optional dependency; never install with `--omit=optional`.

## Plugins

Plugins are grouped by domain and named `<domain>.<name>`, the name unified's ecosystem uses
spelled out: `md.parse`, `md.gfm`, `md.toHtml`, `html.stringify`, `html.toMarkdown`,
`mdx.compile`, `nl.parse`, `nl.equality`. Options take the upstream plugin's camelCase
names (`allowDangerousHtml`, as remark-rehype). A pipeline has one parser and one compiler, and
bridges such as `md.toHtml` (remark-rehype) change the tree between them.

| Domain | Node.js         | Python          | Rust             |
| ------ | --------------- | --------------- | ---------------- |
| `md`   | `textlane/md`   | `textlane.md`   | `textlane::md`   |
| `html` | `textlane/html` | `textlane.html` | `textlane::html` |
| `mdx`  | `textlane/mdx`  | `textlane.mdx`  | `textlane::mdx`  |
| `nl`   | `textlane/nl`   | `textlane.nl`   | `textlane::nl`   |

`pluginNames()` (Node.js) and `textlane.plugin_names()` (Python) list every plugin.

## Node.js

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = await textlane()
  .use(md.parse())
  .use(md.gfm())
  .use(md.toHtml({ allowDangerousHtml: true }))
  .use(html.stringify({ allowDangerousHtml: true }))
  .process("# hi")
file.value // "<h1>hi</h1>"
file.messages // [{ reason, ruleId, source, place }]
```

A domain function returns a descriptor, `{ name: "md.toHtml", options }`, typed from the
crate. The processor follows unified: `use(plugin | [plugin, options] | preset | list)`,
`data(key?, value?)`, `freeze()`, `parse(text)`, `run(tree, file?)` (Promise),
`runSync(tree, file?)`, `stringify(tree)`, `process(text)` (Promise) and `processSync(text)`.
Trees are plain unist JSON objects.

## Python

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.gfm())
    .use(md.to_html({"allowDangerousHtml": True}))
    .use(html.stringify())
    .process("# hi")
)
file["value"]  # "<h1>hi</h1>"
```

Functions are snake_case (`md.to_html`, `nl.indefinite_article`); their options stay the
camelCase keys the plugin takes, typed as `TypedDict`s. `process(text)` and `run(tree)`
return directly; `await processor.aprocess(text)` and `await processor.arun(tree)` are their
coroutines. A
file and a tree are dicts.

## Rust

```rust
use textlane::{Processor, html, md};

let processor = Processor::new()
    .with(md::Parse::default())
    .with(md::ToHtml::default())
    .with(html::Stringify::default())
    .freeze()?;
let output = processor.process("# hi")?; // output.value, output.file.messages
```

## Memory

A document's trees are built in working memory that each thread keeps afterwards, emptied, so
the next document reuses it instead of asking the system again. At most 256 MB is kept per
thread; a larger document's memory is freed once it finishes. Change the cap, in bytes, with
`setRetainedMemory(bytes)` (Node.js), `textlane.set_retained_memory(bytes)` (Python) or
`textlane::set_retained_memory(bytes)` (Rust), and read it back with `retainedMemory()`,
`textlane.retained_memory()` or `textlane::read_retained_memory()`. It applies to every
processor in the process. Raise it for documents of several megabytes, which need more
than 256 MB of working memory; lower it, or set `0`, when idle memory matters more than speed.

## Errors

Failures raise an `Error` (Node.js) or `textlane.TextlaneError` (Python) whose `code` names
the case: `unknown-plugin`, `invalid-options`, `missing-parser`, `missing-compiler`,
`frozen-processor`, `async-in-sync`, `invalid-message`. Handle by code, not by message.

## Not yet

- A JavaScript or Python plugin joins only as a transformer: `use(fn, options)` calls it at
  freeze (`this` is the processor in JavaScript, the first argument in Python) and runs what it
  returns over the JSON tree; its file is vfile-shaped, its path from `process({value, path})`.
  The tree crosses as JSON, so non-JSON values in nodes or `data` do not survive a native
  stage. Syntax plugins (remark-gfm, remark-directive, remark-math) need the native ones; do
  not port a pipeline that needs one textlane lacks, say so. A remark plugin that only turns
  directives, `> [!NOTE]` alerts, definition lists or fenced code into MDX JSX elements is a
  rule of `md.component` instead. remark-heading-id, strip-markdown, rehype-unwrap-images and
  @stefanprobst/rehype-extract-toc are `md.headingId`, `md.strip`, `html.unwrapImages` and
  `html.extractToc`; the table of contents lands in the result's `data.toc`.
- remark-stringify reads `settings` from processor data; textlane's `md.stringify` takes its
  options directly.
