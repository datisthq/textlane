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
