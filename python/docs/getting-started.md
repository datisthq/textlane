---
title: Getting started
description: Install textlane and turn Markdown into HTML from Python.
path: /python/getting-started/
icon: play
order: 3
---

# Getting started

## Install

```bash
pip install textlane
```

The wheel carries a prebuilt extension for your platform, so no Rust toolchain is needed. It
supports Python 3.9 or later.

## A first pipeline

A pipeline parses text into a tree, runs plugins over it and serializes the result. Each
plugin is a function in its domain's module: `textlane.md`, `textlane.html`, `textlane.mdx`
and `textlane.nl`.

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.to_html())
    .use(html.stringify())
    .process("# Hello *world*")
)
file["value"]
# <h1>Hello <em>world</em></h1>
```

`md.parse` reads Markdown into mdast, `md.to_html` turns it into hast and `html.stringify`
writes HTML. `process` returns the file; `await processor.aprocess(text)` is its
coroutine form.

## Adding syntax

Syntax extensions sit beside the parser. Add `md.gfm` for tables, task lists, strikethrough
and autolinks, and `md.frontmatter` to read a YAML block at the top:

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.gfm())
    .use(md.frontmatter())
    .use(md.to_html())
    .use(html.stringify())
    .process("---\ntitle: Notes\n---\n\n~~draft~~ done")
)
file["value"]
# <p><del>draft</del> done</p>
```

## Reading the result

The file is a dict holding the output as `value`, the `messages` plugins reported and the
`data` they set. Checks such as `nl.repeated_words` leave the text alone and report a
message:

```python
from textlane import md, nl, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.to_natural_language())
    .use(nl.repeated_words())
    .use(nl.stringify())
    .process("It is the the best.")
)
file["messages"][0]["reason"]
# Unexpected repeated `the`, remove one occurrence
```

Next, the [processor](/python/processor/) covers every method, and the plugin pages list
each plugin with its options.
