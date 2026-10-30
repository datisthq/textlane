---
title: Processor
description: Build a pipeline with use, share data, freeze it and run each stage on its own.
path: /typescript/processor/
icon: workflow
order: 4
---

# Processor

`textlane()` returns a new processor, which follows unified's: plugins added with `use`,
shared `data`, `freeze`, and `parse`, `run`, `stringify` and `process` for the stages.

## use

`use` takes a plugin, a plugin and its options as two arguments, a list, or a preset. In a
list, a plugin with options is a `[plugin, options]` pair; a top-level array is always a
list, as in unified. The functions in `textlane/md` and the other subpaths build a
descriptor, `{ name, options }`; a plugin's name works as well. A JavaScript function is a
unified attacher run between the native stages, as the
[Overview](/typescript/overview/#javascript-plugins) describes.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use("md.gfm")
  .use([
    [md.toHtml(), { allowDangerousHtml: true }],
    [html.stringify(), { allowDangerousHtml: true }],
  ])
  .processSync("Hi <kbd>Ctrl</kbd>")
file.value
// <p>Hi <kbd>Ctrl</kbd></p>
```

A plugin used again is reconfigured: its new options merge into the earlier ones. A preset
bundles plugins, `{ plugins: [...] }`, for reuse:

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const markdownToHtml = { plugins: [md.parse(), md.gfm(), md.toHtml(), html.stringify()] }
const file = textlane().use(markdownToHtml).processSync("**bold**")
file.value
// <p><strong>bold</strong></p>
```

Each stage must take the tree the one before returns: Markdown into mdast, then mdast to
hast, then hast to HTML. A pipeline out of order fails with `invalid-stage` when it runs.

## data

`data` gets and sets values shared across the pipeline, as unified's does: `data()` returns
all of it, `data(key)` one value, `data(key, value)` and `data(object)` set it.

```js
import { textlane } from "textlane"

const processor = textlane().data("site", "textlane.dev")
processor.data("site")
// textlane.dev
```

## freeze

`freeze` attaches every plugin once. Later `use` and `data` setters fail with
`frozen-processor`, so a frozen processor can be shared and reused for many documents.
`process` freezes the processor itself when it first runs.

## Stages

`process` runs all three stages; each also runs on its own:

- `parse(text)` returns the tree the parser reads, as plain JSON
- `run(tree, file?)` runs the transforms over a tree and resolves to the new tree;
  `runSync` returns it. Pass the text the tree came from as `file` to keep positions
- `stringify(tree)` serializes a tree with no transform
- `process(text)` resolves to the file; `processSync(text)` returns it

```js
import { textlane } from "textlane"
import * as md from "textlane/md"

const processor = textlane()
  .use(md.parse())
  .use(md.stringify({ bullet: "-" }))
const tree = processor.parse("* one\n* two")
tree.children[0].type
// list
processor.stringify(tree)
// - one
// - two
```

The sync forms fail with `async-in-sync` when a stage has asynchronous work to finish; use
`process` and `run` then.

## The file

`process` gives back a file shaped as a vfile:

- `value` — the output text
- `messages` — what plugins reported: each has a `reason`, a `ruleId`, the `source` plugin
  and a `place` with the `start` and `end` points
- `data` — what plugins stored on the file

## Package exports

- `version()` — the version of the native crate
- `pluginNames()` — every plugin's name, as a pipeline names it
- `retainedMemory()` and `setRetainedMemory(bytes)` — see [Memory](/typescript/memory/)
