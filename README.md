# Textlane

A Rust-native text processing pipeline: a replacement for unified whose parsers, transforms and
serializers run natively, while existing JavaScript plugins can still join the same pipeline.

| Channel   | Package                                              | Install                |
| --------- | ---------------------------------------------------- | ---------------------- |
| npm       | [`textlane`](https://www.npmjs.com/package/textlane) | `npm install textlane` |
| PyPI      | [`textlane`](https://pypi.org/project/textlane/)     | `pip install textlane` |
| crates.io | [`textlane`](https://crates.io/crates/textlane)      | `cargo add textlane`   |

Every channel ships the same Rust crate at the same version. Textlane is pre-release: native
plugins run from Rust, Node.js and Python; JavaScript and Python plugins cannot join a pipeline
yet. The docs live at [textlane.dev](https://textlane.dev), built
with livemark from `python/docs/`,
`typescript/docs/` and `rust/docs/`. This repository holds the crate in `rust/` and the npm and PyPI
bindings in `typescript/` and `python/`; see [Contributing](CONTRIBUTING.md) for the layout and
the commands.
