---
name: textlane-usage
description: Use when installing textlane or calling it from Rust, Node.js or Python — which package each language uses, what the API offers today and what it does not yet.
---

# Textlane Usage

Textlane is a Rust-native text processing pipeline: a replacement for unified whose plugins run
natively, with Node.js and Python bindings over the same crate. It is pre-release; the pipeline
API is not published yet.

## When to use this skill

Invoke when the user asks to:

- Install textlane in a Rust, Node.js or Python project.
- Check which textlane version is installed.

## Install

| Language | Package                | Install                |
| -------- | ---------------------- | ---------------------- |
| Rust     | `textlane` (crates.io) | `cargo add textlane`   |
| Node.js  | `textlane` (npm)       | `npm install textlane` |
| Python   | `textlane` (PyPI)      | `pip install textlane` |

Every channel ships the same crate at the same version. npm installs one prebuilt addon for the
current platform as an optional dependency; never install with `--omit=optional`.

## API today

Only the version is exposed:

- Rust: `textlane::read_version()`
- Node.js: `import { version } from "textlane"`, then `version()`
- Python: `import textlane`, then `textlane.version()`

Do not invent pipeline calls (`textlane()`, `.use()`, `process()`): they are planned, not
shipped. Say so instead of guessing.
