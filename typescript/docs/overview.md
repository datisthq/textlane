---
title: Overview
description: What textlane is for and how the TypeScript package is built.
path: /typescript/overview/
icon: book-open
order: 1
---

# Overview

Textlane is a Rust-native replacement for [unified](https://unifiedjs.com): a pipeline that
parses text into a syntax tree, runs plugins over it and serializes the result. The pipeline
and its most expensive plugins run in Rust; existing JavaScript plugins can still take part,
called only at the stage where they run.

## Installation

```bash
npm install textlane
```

It needs Node.js 20 or later; a prebuilt addon for Linux, macOS and Windows, x64 and arm64, comes along as an optional dependency, so no Rust toolchain is needed.

## The TypeScript package

`npm install textlane` installs a small loader and one prebuilt napi addon for your platform,
pulled in as an optional dependency, so no Rust toolchain is needed. Never install with
`--omit=optional`. The addon is the textlane Rust crate itself: the npm package only maps its
types, so Node.js gets the same pipeline as Rust and Python, at the same version.

## Conformance

Compatibility is defined by a language-neutral conformance suite: JSON cases, each naming a
pipeline, an input and the expected output, ported from the upstream unified, remark, rehype,
retext and MDX test suites. Every implementation, TypeScript included, runs the same cases.

## Status

Textlane is pre-release. `textlane()` builds a processor over the native pipeline, and
`textlane/md`, `textlane/html`, `textlane/mdx` and `textlane/nl` export one function per
plugin, typed from the crate's option structs.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = await textlane().use(md.parse()).use(md.toHtml()).use(html.stringify()).process("# hi")
file.value // "<h1>hi</h1>"
```

## JavaScript plugins

A function passed to `use` is a unified attacher: called with the processor as `this` and its
options when the processor freezes, it returns a transformer. A run pauses before each run of
consecutive JavaScript plugins and hands them the tree as JSON, mdast or hast as the stage
before left it; they change it in place or return a new one, directly, through a promise or
through `next`, and the run goes on natively. `processSync` fails with `async-in-sync` on a
transformer that finishes later, and one returning an `Error` fails the run. The file is
vfile's shape: `message`, `info` and `fail` make vfile-messages that land in the result's
`messages`, `file.data` becomes its `data`, and `process({ value, path })` gives plugins
`path`, `dirname`, `stem` and `extname` and puts `path` on the result. Messages from native
stages reach only the result, not `file.messages`.

```js
const strong = () => tree => {
  tree.children[0].children[0].type = "strong"
}

await textlane().use(md.parse()).use(strong).use(md.toHtml()).use(html.stringify()).process("*a*")
```

Each pause serializes the tree, so a plugin costs one round trip per batch, not per node. A
JavaScript plugin only transforms: syntax extensions such as remark-gfm need the native
plugin.

## Next steps

- [Getting started](/typescript/getting-started/) — a first pipeline, syntax extensions and the result
- [Processor](/typescript/processor/) — every method of the pipeline
- [Recipes](/typescript/recipes/) — pipelines for common jobs
- [Markdown plugins](/typescript/markdown-plugins/), [HTML plugins](/typescript/html-plugins/),
  [MDX plugins](/typescript/mdx-plugins/) and
  [natural language plugins](/typescript/natural-language-plugins/) — each plugin and its options
