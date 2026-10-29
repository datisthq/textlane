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

Textlane is pre-release. The pipeline API is not public in Python yet; the package exposes
`textlane.version()` so far.
