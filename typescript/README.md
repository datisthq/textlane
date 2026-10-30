# Textlane

Textlane is a Rust-native text processing pipeline: a replacement for unified whose plugins
run natively, while existing JavaScript plugins can still join the same pipeline. One Rust
crate ships to crates.io, npm and PyPI at one version.

## Installation

```bash
npm install textlane
```

The package carries a prebuilt addon for Linux, macOS and Windows (x64 and arm64)
through a platform-specific optional dependency; no Rust toolchain is needed.

## Usage

A processor chains plugins with `use`, as unified does, then parses, transforms and serializes
text with `process`. Each language reaches the same crate; plugins are grouped by domain (`md`,
`html`, `mdx`, `nl`) and named as the pipeline names them.

Rust:

```rust
use textlane::{Processor, html, md};

let processor = Processor::new()
    .with(md::Parse::default())
    .with(md::Gfm::default())
    .with(md::ToHtml::default())
    .with(html::Stringify::default())
    .freeze()?;
assert_eq!(processor.process("# hi")?.value, "<h1>hi</h1>");
```

Node.js:

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = await textlane()
  .use(md.parse())
  .use(md.gfm())
  .use(md.toHtml({ allowDangerousHtml: true }))
  .use(html.stringify())
  .process("# hi")
file.value // "<h1>hi</h1>"
```

Python:

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.gfm())
    .use(md.to_html({"allowDangerousHtml": True}))
    .use(html.stringify())
    .process_sync("# hi")
)
file["value"]  # "<h1>hi</h1>"
```

Node.js and Python also offer `parse`, `run`, `stringify`, `data` and `freeze`, with
`process`/`run` returning a Promise or coroutine and `processSync`/`process_sync` the result.
Errors carry the crate's `code`, such as `unknown-plugin`. Only native plugins can join a
pipeline so far; JavaScript and Python plugins are planned.

## Documentation

Read the docs at [textlane.dev](https://textlane.dev).

## License

MIT.
