---
title: Overview
description: What textlane is for and how the Python package is built.
path: /python/overview/
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
pip install textlane
```

It needs Python 3.9 or later; the wheel carries a prebuilt extension for your platform, so no Rust toolchain is needed.

## The Python package

`pip install textlane` installs a wheel with a prebuilt PyO3 extension for your platform,
so no Rust toolchain is needed. The extension is the textlane Rust crate itself: the Python
package only maps its types, so Python gets the same pipeline as Rust and Node.js, at the same
version.

## Conformance

Compatibility is defined by a language-neutral conformance suite: JSON cases, each naming a
pipeline, an input and the expected output, ported from the upstream unified, remark, rehype,
retext and MDX test suites. Every implementation, Python included, runs the same cases.

## Status

Textlane is pre-release. `textlane()` builds a processor over the native pipeline, and
`textlane.md`, `textlane.html`, `textlane.mdx` and `textlane.nl` hold one function per
plugin, its options typed as a `TypedDict` generated from the crate's option structs.

```python
from textlane import html, md, textlane

file = textlane().use(md.parse()).use(md.to_html()).use(html.stringify()).process("# hi")
file["value"]  # "<h1>hi</h1>"
```

## Python plugins

A callable passed to `use` is an attacher: called with the processor and its options when the
processor freezes, it returns a transformer taking the tree, as a dict, and the file. A run
pauses before each run of consecutive Python plugins and hands them the tree, mdast or hast as
the stage before left it; they change it in place or return a new one, and `aprocess` awaits
one returning an awaitable, which `process` refuses with `async-in-sync`; one returning
an exception fails the run. The file is vfile's shape: `message`, `info` and `fail` make
`HostMessage`s that land in the result's `messages`, `file.data` becomes its `data`, and
`process({"value": …, "path": …})` gives plugins `path`, `dirname`, `stem` and `extname` and
puts `path` on the result.

```python
def strong(processor, options=None):
    def transformer(tree, file):
        tree["children"][0]["children"][0]["type"] = "strong"

    return transformer

textlane().use(md.parse()).use(strong).use(md.to_html()).use(html.stringify()).process("*a*")
```

A Python plugin only transforms: syntax, parsers and compilers need native plugins.

## Next steps

- [Getting started](/python/getting-started/) — a first pipeline, syntax extensions and the result
- [Processor](/python/processor/) — every method of the pipeline
- [Recipes](/python/recipes/) — pipelines for common jobs
- [Markdown plugins](/python/markdown-plugins/), [HTML plugins](/python/html-plugins/),
  [MDX plugins](/python/mdx-plugins/) and
  [natural language plugins](/python/natural-language-plugins/) — each plugin and its options
