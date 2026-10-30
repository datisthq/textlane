---
title: Processor
description: Build a pipeline with use, share data, freeze it and run each stage on its own.
path: /python/processor/
icon: workflow
order: 4
---

# Processor

`textlane()` returns a new `Processor`, which follows unified's: plugins added with `use`,
shared `data`, `freeze`, and `parse`, `run`, `stringify` and `process` for the stages.

## use

`use` takes a plugin, a plugin and its options as two arguments, a list, or a preset. In a
list, a plugin with options is a `[plugin, options]` pair; a top-level list is always a
list, as in unified. The functions in `textlane.md` and the other modules build a
descriptor, `{"name": ..., "options": ...}`; a plugin's name works as well. A Python callable
is an attacher run between the native stages, as the
[Overview](/python/overview/#python-plugins) describes. Function names
are snake_case (`md.to_html`), while options keep the camelCase keys the plugin takes.

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use("md.gfm")
    .use(
        [
            [md.to_html(), {"allowDangerousHtml": True}],
            [html.stringify(), {"allowDangerousHtml": True}],
        ]
    )
    .process("Hi <kbd>Ctrl</kbd>")
)
file["value"]
# <p>Hi <kbd>Ctrl</kbd></p>
```

A plugin used again is reconfigured: its new options merge into the earlier ones. A preset
bundles plugins, `{"plugins": [...]}`, for reuse:

```python
from textlane import html, md, textlane

markdown_to_html = {"plugins": [md.parse(), md.gfm(), md.to_html(), html.stringify()]}
file = textlane().use(markdown_to_html).process("**bold**")
file["value"]
# <p><strong>bold</strong></p>
```

Each stage must take the tree the one before returns: Markdown into mdast, then mdast to
hast, then hast to HTML. A pipeline out of order fails with `invalid-stage` when it runs.

## data

`data` gets and sets values shared across the pipeline, as unified's does: `data()` returns
all of it, `data(key)` one value, `data(key, value)` and `data(dict)` set it.

```python
from textlane import textlane

processor = textlane().data("site", "textlane.dev")
processor.data("site")
# textlane.dev
```

## freeze

`freeze` attaches every plugin once. Later `use` and `data` setters fail with
`frozen-processor`, so a frozen processor can be shared and reused for many documents.
`process` freezes the processor itself when it first runs.

## Stages

`process` runs all three stages; each also runs on its own:

- `parse(text)` returns the tree the parser reads, as a dict
- `run(tree, file=None)` runs the transforms over a tree and returns the new tree;
  `await arun(tree, file=None)` is its coroutine. Pass the text the tree came from as `file` to keep
  positions
- `stringify(tree)` serializes a tree with no transform
- `process(text)` returns the file; `await aprocess(text)` is its coroutine

```python
from textlane import md, textlane

processor = textlane().use(md.parse()).use(md.stringify({"bullet": "-"}))
tree = processor.parse("* one\n* two")
tree["children"][0]["type"]
# list
processor.stringify(tree)
# - one
# - two
```

The coroutines suit asyncio code:

```python
import asyncio

from textlane import html, md, textlane

processor = textlane().use(md.parse()).use(md.to_html()).use(html.stringify())
file = asyncio.run(processor.aprocess("*hi*"))
file["value"]
# <p><em>hi</em></p>
```

`process` and `run` fail with `async-in-sync` when a stage has asynchronous work to finish;
use `aprocess` and `arun` then.

## The file

`process` returns a dict shaped as a vfile:

- `value` — the output text
- `messages` — what plugins reported: each has a `reason`, a `ruleId`, the `source` plugin
  and a `place` with the `start` and `end` points
- `data` — what plugins stored on the file

## Module functions

- `textlane.version()` — the version of the native crate
- `textlane.plugin_names()` — every plugin's name, as a pipeline names it
- `textlane.retained_memory()` and `textlane.set_retained_memory(size)` — see
  [Memory](/python/memory/)
