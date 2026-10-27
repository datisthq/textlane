---
title: Overview
description: What textlane is for and how it is built.
icon: book-open
order: 2
---

# Overview

Textlane is a Rust-native replacement for [unified](https://unifiedjs.com): a pipeline that
parses text into a syntax tree, runs plugins over it and serializes the result. The pipeline
and its most expensive plugins run in Rust; existing JavaScript plugins can still take part,
called only at the stage where they run.

## One crate, three languages

All behaviour lives in one Rust crate. The npm package loads it as a prebuilt native addon and
the PyPI package as a prebuilt extension, so each language gets the same pipeline at the same
version.

## Conformance

Compatibility is defined by a language-neutral conformance suite: JSON cases, each naming a
pipeline, an input and the expected output, ported from the upstream unified, remark, rehype
and MDX test suites. Every implementation runs the same cases.

## Status

Textlane is pre-release. The packages are published to reserve the names and exercise the
release pipeline; the pipeline API is not public yet.
