---
title: Overview
description: What textlane is for and how the Rust crate is built.
path: /rust/overview/
icon: book-open
order: 2
---

# Overview

Textlane is a Rust-native replacement for [unified](https://unifiedjs.com): a pipeline that
parses text into a syntax tree, runs plugins over it and serializes the result. The pipeline
and its most expensive plugins run in Rust; existing JavaScript plugins can still take part,
called only at the stage where they run.

## The Rust crate

`cargo add textlane` adds the crate that holds all of textlane's behaviour. It re-exports one
crate per domain as a module of its name: `textlane::md`, `textlane::html`,
`textlane::mdx` and `textlane::nl`. The npm package loads the same crate as a prebuilt native addon and the PyPI
package as a prebuilt extension, so each language gets the same pipeline at the same version.

## Conformance

Compatibility is defined by a language-neutral conformance suite: JSON cases, each naming a
pipeline, an input and the expected output, ported from the upstream unified, remark, rehype,
retext and MDX test suites. Every implementation runs the same cases.

## Status

Textlane is pre-release. In Rust, `Processor::new().with(plugin)` builds a typed pipeline;
the Node.js and Python packages drive `textlane::Pipeline`, which takes plugins by name with
JSON options and follows unified's `use`, `data` and `freeze`. JavaScript and Python plugins
cannot join a pipeline yet.

## Memory

Each thread keeps the working memory a document's trees were built in, emptied, for the next
document, up to 256 MB; a document that needs more has its memory freed once it finishes.
Documents of several megabytes need more than that, so raise the cap for them, or lower it
when idle memory matters more than speed. It applies to every processor in the process, and
`textlane::read_retained_memory()` reads it back.

```rust
textlane::set_retained_memory(512 << 20);
```
