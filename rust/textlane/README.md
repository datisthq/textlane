# Textlane

Textlane is a Rust-native text processing pipeline: a replacement for unified whose plugins
run natively, while existing JavaScript plugins can still join the same pipeline. One Rust
crate ships to crates.io, npm and PyPI at one version.

## Installation

```bash
cargo add textlane
```

## Usage

Textlane is pre-release: the pipeline API is not published yet, and each package exposes only
its version so far.

| Language | Call                                                   |
| -------- | ------------------------------------------------------ |
| Rust     | `textlane::read_version()`                             |
| Node.js  | `import { version } from "textlane"`, then `version()` |
| Python   | `import textlane`, then `textlane.version()`           |

## Documentation

Read the docs at [textlane.dev](https://textlane.dev).

## License

MIT.
