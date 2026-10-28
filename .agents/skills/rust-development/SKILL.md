---
name: rust-development
description: Use when working in `rust/` — the textlane crate and its napi and PyO3 bindings. Covers its layout, the cargo commands, the `node` and `python` features, and the lint policy.
---

# Rust development

`rust/` is the whole product: a cargo workspace whose crate `textlane` (`rust/textlane/`) is the
library every channel ships. `textlane/src/lib.rs` is the façade; the npm addon and the Python
extension are the same crate built with the `node` or `python` feature.

## Commands

Run from the repo root; each wraps cargo in `rust/package.json`:

```sh
pnpm -C rust setup          # cargo fetch
pnpm -C rust check:lint     # cargo fmt --check + clippy -D warnings, default/node/python
pnpm -C rust check:types    # cargo check --workspace --all-targets
pnpm -C rust test:unit      # cargo test --workspace
pnpm -C rust build          # cargo build --release into rust/target/release/
```

A single test: `cargo test name` from `rust/`. `rust/rust-toolchain.toml` pins stable with
clippy and rustfmt, so rustup installs the right toolchain on first use, locally and in CI.

## Layout

- `rust/Cargo.toml` is a virtual manifest; each crate is a sibling directory named by its role,
  as in fairspec: `textlane/`, `core/` (`textlane-core`: the trees, the file and the plugin
  contract every other crate shares), and one directory per domain: `md/`, `html/`, `mdx/`,
  `prose/` (`textlane-<domain>`). A domain crate depends on `textlane-core`, never on another
  domain or on `textlane`, which depends on them all. `mdx` compiles core's hast to a JS tree
  on oxc, so oxc is its own dependency and never core's
- A domain is named short (`md`, `html`, `mdx`, `prose`); its functions spell formats out in
  full: `md.toHtml`, `html.toMarkdown`, `md.toProse`. Rust `to_markdown` becomes napi's
  `toMarkdown` and Python's `to_markdown` with no `js_name` override
- `textlane` re-exports each domain as a module of its name and, once `textlane-core` has
  items, those at its root; never as `textlane::core`, which would make every `core::` path in
  the crate ambiguous with Rust's built-in `core`. A glob re-export of a crate with no items
  fails clippy as an unused import
- Shared metadata, the member version pins, the release profile and the lint policy live once
  in `rust/Cargo.toml` (`[workspace.package]`, `[workspace.dependencies]`, `[workspace.lints]`); a
  member inherits each with `.workspace = true`. A new member joins `[workspace.dependencies]`
  and the release-please `extra-files` (**`package-distribution`**)
- In each crate, `src/actions/<entity>/<verb>.rs` exports `verb_noun`
  (`actions/package/version.rs` exports `read_version`); the module tree is declared inline in `lib.rs`, so there is no `mod.rs`
- `textlane/src/bindings/node.rs` and `python.rs` only map crate functions to their
  language (`version()` in both); behaviour never lives there
- Unit tests sit beside their module as `<module>_unit.rs`, wired in with
  `#[cfg(test)] #[path = "<module>_unit.rs"] mod unit;`; one that can fail returns a `Result`
  and uses `?` instead of `unwrap`, which clippy denies

## Features

- The default features need neither Node nor Python: `cargo add textlane` must build without
  napi or PyO3. Every binding dependency is optional and enabled only by its feature.
- `md`, `html`, `mdx` and `prose` (all default) gate the domain re-exports. `node` and
  `python` enable every domain, since each binding is one addon that must carry them all.
- `node` pulls in napi, and `build.rs` runs napi-build only under it; `python` pulls in PyO3
  with `abi3-py39`. Each binding module is compiled only under its feature.
- `pnpm -C rust check:lint` runs clippy three times: default features, `--features node` and
  `--features python`, so CI on every OS compiles both bindings before a release can tag.
- `unsafe_code` is `deny` in `[workspace.lints]`. The one exception is `bindings::node`, allowed
  in `lib.rs` because napi's macros expand to unsafe FFI glue; never widen it.
