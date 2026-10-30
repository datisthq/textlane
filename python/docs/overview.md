---
title: Overview
description: What textlane is for and how the Python package is built.
path: /python/overview/
icon: book-open
order: 2
---

# Overview

Textlane is a Rust-native replacement for [unified](https://unifiedjs.com): a pipeline that
parses text into a syntax tree, runs plugins over it and serializes the result. The pipeline
and its most expensive plugins run in Rust; existing JavaScript plugins can still take part,
called only at the stage where they run.

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

file = textlane().use(md.parse()).use(md.to_html()).use(html.stringify()).process_sync("# hi")
file["value"]  # "<h1>hi</h1>"
```

## Python plugins

A callable passed to `use` is an attacher: called with the processor and its options when the
processor freezes, it returns a transformer taking the tree, as a dict, and the file. A run
pauses before each run of consecutive Python plugins and hands them the tree, mdast or hast as
the stage before left it; they change it in place or return a new one, and `process` awaits
one returning an awaitable, which `process_sync` refuses with `async-in-sync`; one returning
an exception fails the run. The file is vfile's shape: `message`, `info` and `fail` make
`HostMessage`s that land in the result's `messages`, `file.data` becomes its `data`, and
`process({"value": …, "path": …})` gives plugins `path`, `dirname`, `stem` and `extname` and
puts `path` on the result.

```python
def strong(processor, options=None):
    def transformer(tree, file):
        tree["children"][0]["children"][0]["type"] = "strong"

    return transformer

textlane().use(md.parse()).use(strong).use(md.to_html()).use(html.stringify()).process_sync("*a*")
```

A Python plugin only transforms: syntax, parsers and compilers need native plugins.
