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
pnpm -C rust test:unit      # cargo test --workspace --features testing
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
- Beside `actions/`, as in fairspec: `models/` for types, `helpers/` for small free functions,
  `plugins/<name>/` for one `.use()` stage each (`plugin.rs` plus its own `actions/`), and
  `utils/<name>/` for a bounded internal system whose files are mutually recursive (`md`'s
  `tokenizer`, `compiler`, `serializer`, `converter`); only it is exempt from one action per file
- Trees live in one `bumpalo` arena per document (`textlane_core::Allocator`, `ArenaVec`);
  strings borrow the input or the arena. Input decides how deep a tree nests, so nothing
  walks one by recursion: `ArenaVec` never drops its items, conversion and `html.stringify`
  keep an explicit stack, and JSON stops at 1000 levels with `tree-too-deep`. Watch for
  quadratic passes too: `processor_unit.rs` runs 100,000-deep and 50,000-run inputs,
  and unclosed code spans of growing length. A construct that scans ahead and can fail
  needs a memo of what the failed scan saw, as `code_text_runs` is for code spans and
  `thematic_break_failed` for nested list markers. Nested containers all close at one
  place, so a walk back over their exits must be precomputed (`helpers/spread.rs`);
  blank lines skip list items that continue for free (`document_blank_ends`). Text joins
  pieces through a growable arena buffer, a label is matched against definitions in the
  length of the longest identifier (`ParseState::is_defined`), and a label's text is read
  only when it names a reference. Inline HTML remembers which kinds scanned to the end
  unclosed (`html_text_unclosed`). Time
  growth in release mode, best of several runs: single debug timings are noisy.
- Container ends follow micromark, not markdown-rs: a list item ends at the first line
  ending of its trailing blank lines (`prepareList`), stepping over the exits of lists
  inside but not of block quotes; a list does the same without stepping over block quote
  prefixes. Trailing blank lines never make an item loose A `Point` is a byte offset whose top two bits hold
  a tab's virtual spaces; `Lines` turns it into unist's UTF-16 line, column and offset
- `md`'s engine is a port of markdown-rs (micromark's design in Rust): a state is a plain
  `fn(&mut Tokenizer) -> State`, a token name an open `Name("…")` constant. A construct not
  ported yet is marked where it joins, in `utils/tokenizer/{document,flow,text}.rs`. Where
  markdown-rs differs from micromark or unified, the corpus wins (flow nodes start after their
  indent: `tail_push_after_prefix`; `*` and `_` never count as punctuation next to attention;
  an escape's text starts at its backslash, as from-markdown opens it on `characterEscape`;
  string content is never trimmed; a token's slice leaves out container prefixes of its later
  lines and reads NUL as U+FFFD, as micromark's chunks do; each line of a label or title is
  a string of its own, a title's indent after a line ending belongs to neither; a point
  inside a tab is one column and one offset past the tab's start).
  `helpers/entities.rs` and `helpers/unicode.rs` are tables generated from markdown-rs's,
  sorted for binary search
- `md` names entities by syntax construct or mdast node and verbs by layer: `tokenize` and
  `resolve` (micromark), `compile` (from-markdown), `stringify` (to-markdown), `convert`
  (to-hast), so `actions/heading/compile.rs` exports `compile_heading`. No file named after
  its parent module (`utils/tokenizer/tokenizer.rs`): clippy's `module_inception` rejects it
- `textlane/src/bindings/node.rs` and `python.rs` only map crate functions to their
  language (`version()` in both); behaviour never lives there
- Unit tests sit beside their module as `<module>_unit.rs`, wired in with
  `#[cfg(test)] #[path = "<module>_unit.rs"] mod unit;`; one that can fail returns a `Result`
  and uses `?` instead of `unwrap`, which clippy denies

## Features

- The default features need neither Node nor Python: `cargo add textlane` must build without
  napi or PyO3. Every binding dependency is optional and enabled only by its feature.
- `testing` adds `run_case` and the `test.*` plugins the conformance suite drives, and exposes
  them through both bindings; it is never default and never published in an addon or wheel.
  `rust/textlane/tests/conformance.rs` requires it (**`conformance-suite`**).
- `md`, `html`, `mdx` and `prose` (all default) gate the domain re-exports. `node` and
  `python` enable every domain, since each binding is one addon that must carry them all.
- `node` pulls in napi, and `build.rs` runs napi-build only under it; `python` pulls in PyO3
  with `abi3-py39`. Each binding module is compiled only under its feature.
- `pnpm -C rust check:lint` runs clippy three times: default features, `--features node` and
  `--features python`, so CI on every OS compiles both bindings before a release can tag.
- `unsafe_code` is `deny` in `[workspace.lints]`. The one exception is `bindings::node`, allowed
  in `lib.rs` because napi's macros expand to unsafe FFI glue; never widen it.
