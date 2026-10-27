---
name: rust-development
description: Use when working in `rust/` — the textlane crate and its napi and PyO3 bindings. Covers its layout, the cargo commands, the `node` and `python` features, and the lint policy.
---

# Rust development

`rust/` is the whole product: the crates.io package `textlane` and the library every channel
ships. `src/lib.rs` is the façade; the npm addon and the Python extension are the same crate
built with the `node` or `python` feature.

## Commands

Run from the repo root; each wraps cargo in `rust/package.json`:

```sh
pnpm -C rust setup          # cargo fetch
pnpm -C rust check:lint     # cargo fmt --check + clippy -D warnings, default/node/python
pnpm -C rust check:types    # cargo check --all-targets
pnpm -C rust test:unit      # cargo test
pnpm -C rust build          # cargo build --release into rust/target/release/
```

A single test: `cargo test name` from `rust/`. `rust/rust-toolchain.toml` pins stable with
clippy and rustfmt, so rustup installs the right toolchain on first use, locally and in CI.

## Layout

- `src/actions/<entity>/<verb>.rs` exports `verb_noun` (`actions/package/version.rs` exports
  `read_version`); the module tree is declared inline in `lib.rs`, so there is no `mod.rs`
- `src/bindings/node.rs` and `src/bindings/python.rs` only map crate functions to their
  language (`version()` in both); behaviour never lives there
- Unit tests sit beside their module as `<module>_unit.rs`, wired in with
  `#[cfg(test)] #[path = "<module>_unit.rs"] mod unit;`; one that can fail returns a `Result`
  and uses `?` instead of `unwrap`, which clippy denies

## Features

- The default features need neither Node nor Python: `cargo add textlane` must build without
  napi or PyO3. Every binding dependency is optional and enabled only by its feature.
- `node` pulls in napi, and `build.rs` runs napi-build only under it; `python` pulls in PyO3
  with `abi3-py39`. Each binding module is compiled only under its feature.
- `pnpm -C rust check:lint` runs clippy three times: default features, `--features node` and
  `--features python`, so CI on every OS compiles both bindings before a release can tag.
- `unsafe_code` is `deny` in `rust/Cargo.toml`. The one exception is `bindings::node`, allowed
  in `lib.rs` because napi's macros expand to unsafe FFI glue; never widen it.
