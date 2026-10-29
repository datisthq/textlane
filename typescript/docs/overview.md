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

Textlane is pre-release. The pipeline API is not public in Node.js yet; the package exposes
`version()` so far.
