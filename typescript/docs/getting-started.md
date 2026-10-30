---
title: Getting started
description: Install textlane and turn Markdown into HTML from Node.js.
path: /typescript/getting-started/
icon: play
order: 3
---

# Getting started

## Install

```bash
npm install textlane
```

The package brings a prebuilt addon for your platform as an optional dependency, so never
install with `--omit=optional`. Node.js 20 or later loads it as an ES module.

## A first pipeline

A pipeline parses text into a tree, runs plugins over it and serializes the result. Each
plugin is a function from its domain's subpath: `textlane/md`, `textlane/html`,
`textlane/mdx` and `textlane/nl`.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = await textlane()
  .use(md.parse())
  .use(md.toHtml())
  .use(html.stringify())
  .process("# Hello *world*")
file.value
// <h1>Hello <em>world</em></h1>
```

`md.parse` reads Markdown into mdast, `md.toHtml` turns it into hast and `html.stringify`
writes HTML. `process` returns a promise of the file; `processSync` returns the file itself.

## Adding syntax

Syntax extensions sit beside the parser. Add `md.gfm` for tables, task lists, strikethrough
and autolinks, and `md.frontmatter` to read a YAML block at the top:

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.gfm())
  .use(md.frontmatter())
  .use(md.toHtml())
  .use(html.stringify())
  .processSync("---\ntitle: Notes\n---\n\n~~draft~~ done")
file.value
// <p><del>draft</del> done</p>
```

## Reading the result

The file holds the output as `value`, the `messages` plugins reported and the `data` they
set. Checks such as `nl.repeatedWords` leave the text alone and report a message:

```js
import { textlane } from "textlane"
import * as md from "textlane/md"
import * as nl from "textlane/nl"

const file = textlane()
  .use(md.parse())
  .use(md.toNaturalLanguage())
  .use(nl.repeatedWords())
  .use(nl.stringify())
  .processSync("It is the the best.")
file.messages[0].reason
// Unexpected repeated `the`, remove one occurrence
```

Next, the [processor](/typescript/processor/) covers every method, and the plugin pages list
each plugin with its options.
