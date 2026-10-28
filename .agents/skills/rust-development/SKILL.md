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
  walks one by recursion on the plain stack: `ArenaVec` never drops its items, conversion
  and `html.stringify` keep an explicit stack, `md.stringify`'s handlers recurse inside
  `stacker::maybe_grow`, and JSON stops at 1000 levels with `tree-too-deep`. Data built
  while walking must not nest either (`md.stringify` keeps emphasis in one flat store),
  or dropping it recurses. Watch for
  quadratic passes too: `processor_unit.rs` runs 100,000-deep and 50,000-run inputs,
  and unclosed code spans of growing length. A construct that scans ahead and can fail
  needs a memo of what the failed scan saw, as `code_text_runs` is for code spans and
  `thematic_break_failed` for nested list markers. Nested containers all close at one
  place, so a walk back over their exits must be precomputed (`helpers/spread.rs`);
  blank lines skip list items that continue for free (`document_blank_ends`). Text joins
  pieces through a growable arena buffer, a label is matched against definitions in the
  length of the longest identifier (`ParseState::is_defined`), and a label's text is read
  only when it names a reference. Inline HTML remembers which kinds scanned to the end
  unclosed (`html_text_unclosed`). A GFM literal autolink remembers where its last trail
  check and failed domain ended (`literal_trail`, `literal_domain_failed`): a check or
  domain starting in between ends the same way; the find-and-replace pass after parsing
  shares one domain run and one atext run among the starts inside it. The serializer
  never re-indents a container's text: `utils/serializer/sink.rs` records each container as a layer and writes every prefix
  once, deciding a blank line from the innermost prefix out. Nested emphasis renders into
  one shared token buffer, and a delimiter run's token list is shared, not copied per
  token. Time
  growth in release mode, best of several runs: single debug timings are noisy.
- Container ends follow micromark, not markdown-rs: a list item ends at the first line
  ending of its trailing blank lines (`prepareList`), stepping over the exits of lists
  inside but not of block quotes; a list does the same without stepping over block quote
  prefixes. Trailing blank lines never make an item loose, nor does the blank line that
  closes empty containers opened on its own line (micromark puts it inside them). An item
  whose last construct swallowed its line ending (an unclosed fence) ends after the next
  item's prefix. A `Point` is a byte offset whose top two bits hold a tab's virtual
  spaces; `Lines` turns it into unist's UTF-16 line, column and offset
- `utils/tokenizer/document.rs` follows micromark's `document.js`, where markdown-rs
  differs: `interrupt` is decided once a line, before any container opens on it
  (`document_opened`); the last line is flushed before laziness is judged; the empty line
  after a final line ending is lazy inside a block quote the line continued; closing
  containers after a line keeps the child's `interrupt`. Sizes are columns, not bytes: a
  tab after a list marker widens the prefix to the next tab stop. Differences show best
  by fuzzing random fragments through `md.parse` and the pinned from-markdown; each one
  fixed becomes a case in `conformance/sources/differential.ts`
- `md`'s engine is a port of markdown-rs (micromark's design in Rust): a state is a plain
  `fn(&mut Tokenizer) -> State`, a token name an open `Name("…")` constant. A construct not
  ported yet is marked where it joins, in `utils/tokenizer/{document,flow,text}.rs`. Where
  markdown-rs differs from micromark or unified, the corpus wins (flow nodes start after their
  indent: `tail_push_after_prefix`; `*` and `_` never count as punctuation next to attention;
  an escape's text starts at its backslash, as from-markdown opens it on `characterEscape`;
  string content is never trimmed; a token's slice leaves out container prefixes of its later
  lines and reads NUL as U+FFFD, as micromark's chunks do; each line of a label or title is
  a string of its own, a title's indent after a line ending belongs to neither; a point
  inside a tab is one column and one offset past the tab's start; GFM follows micromark's
  extensions: an email literal tokenizes in text, without `mailto:` or `xmpp:`, then
  mdast-util-gfm-autolink-literal's find-and-replace links what micromark left, and a task
  check goes in the first content of an item: after one blank line ending, or definitions).
  Attention and strikethrough classify a neighbour as micromark does a UTF-16 code unit
  (`classify_character_unit`); the serializer reads
  characters as upstream reads UTF-16 code units: nothing, or an astral character, is
  neither whitespace nor punctuation (`classify_unit`), and a reference without a
  `referenceType` is collapsed. GFM footnote calls follow micromark, not markdown-rs: a
  construct of their own at `[`, tried before a link start, that needs a definition the
  document pass already found (`ParseState::gfm_footnote_definitions`), plus a `]` after a
  failed label end that turns `![^a]` into `!` and a call (`gfm_footnote_image`).
  `helpers/entities.rs` and `helpers/unicode.rs` are tables generated from markdown-rs's,
  sorted for binary search
- `html`'s parser is a port of parse5 7.3.0, the tree hast-util-from-parse5 and hastscript
  make from it. The tokenizer is pulled (`next_token`) where parse5 calls back: it stops
  after each non-character token so the tree builder can switch its `state` and
  `in_foreign_node` first, and its errors are drained before each token is handled, which
  keeps parse5's error order. The tree builder works on `utils/builder/dom.rs`, an arena with
  sibling links, because it moves nodes. parse5 scans for things that are not there:
  the stack keeps per-tag counts (`has_open_tag`, the scope checks) and the formatting
  list per-marker counts (Noah's Ark, `find_in_scope`), and parse5's recursion through
  `onEof` for nested templates is a loop. `helpers/schema.rs` is property-information's
  `html` and `svg` schemas, generated and sorted for a binary search. `html.stringify` ports
  hast-util-to-html 9.0.5 whole, omission rules in `helpers/omission.rs`; serde_json keeps
  `preserve_order`, since hast writes properties in the order a tree lists them
  `html.sanitize` and `html.slug` port hast-util-sanitize and rehype-slug; the sanitizer
  walks with its own stack and counts open ancestors instead of searching them, and
  `helpers/slug_table.rs` is github-slugger's removal regex as generated code-point ranges
  `html.toMarkdown` ports hast-util-to-mdast 10.1.2 with rehype-minify-whitespace run first
  (`actions/hast/minify.rs`, which works out `collapsableAfter` for every node in one pass).
  Handlers build into `utils/converter/store.rs`, a flat store that keeps per-subtree flags
  (`flow` for `wrapNeeded`, `link`) and `<q>`'s quotes apart from text, then builds mdast and
  merges texts as upstream's last `visit` does. Flow containers hand their already wrapped
  content up as one `Fragment`, since `wrap(a ++ w) == wrap(a) ++ w`; only consumers that look
  at single items expand it (`expand`). List spread and datalist options are memoized by
  element address. Table spans clamp as browsers do (rowspan 65534, colspan 1000), one table adds
  at most `MAX_ADDED_CELLS` empty cells, and each row's splices from rows above are replayed in
  one pass (`replay_inserts`, a Fenwick tree), keeping upstream's index-based splices exact
- `md.gfm` only puts `GfmOptions` in the `GfmSlot` slot; `md.parse` and `md.stringify` read it
  when their factories run at freeze, so GFM's constructs, handlers and unsafe patterns are
  on only with it. A GFM table is tried after each first byte's own flow constructs, as
  micromark tries constructs keyed on `null`, and a row's indent sits before the row token.
  At `[`, text tries the footnote call, then the task list check, then a link
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
