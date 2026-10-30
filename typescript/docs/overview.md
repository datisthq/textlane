---
title: Overview
description: What textlane is for and how the TypeScript package is built.
path: /typescript/overview/
icon: book-open
order: 2
---

# Overview

Textlane is a Rust-native replacement for [unified](https://unifiedjs.com): a pipeline that
parses text into a syntax tree, runs plugins over it and serializes the result. The pipeline
and its most expensive plugins run in Rust; existing JavaScript plugins can still take part,
called only at the stage where they run.

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

## Memory

Each thread keeps the working memory a document's trees were built in, emptied, for the next
document, up to 256 MB; a document that needs more has its memory freed once it finishes.
Documents of several megabytes need more than that, so raise the cap for them, or lower it
when idle memory matters more than speed. It applies to every processor in the process, and
`retainedMemory()` reads it back.

```js
import { setRetainedMemory } from "textlane"

setRetainedMemory(512 * 1024 ** 2)
```
