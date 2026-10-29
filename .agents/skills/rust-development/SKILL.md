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
  contract every other crate shares: mdast, hast and nlcst), and one directory per domain: `md/`, `html/`, `mdx/`,
  `prose/` (`textlane-<domain>`). A domain crate depends on `textlane-core`, never on another
  domain or on `textlane`, which depends on them all. `mdx` compiles core's hast to JavaScript
  and parses `md.mdx`'s JavaScript, both with oxc, so oxc is its own dependency and never
  core's or md's
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
- Constant factors in `md`'s tokenizer are memory traffic: each document pass reads and
  rewrites every event, so an `Event` stays at 32 bytes (`Name` a `u16` index into `NAMES`,
  `Point` fields `u32`, which caps input at 4 GiB with `input-too-long`, `Link` indices
  `NonZeroU32` held plus one), `EditMap::consume` moves kept runs
  in place (left movers front to back, right movers back to front) and writes added
  events into the gaps, a child list split into chunks goes into the map's `pool` once,
  strings are tokenized inside each child rather than in a pass of their own, and data,
  paragraph and chunk lines take runs of plain bytes at once (`Tokenizer::consume_while`).
  Per-event side tables pay the same: keep them sparse, as the compiler's trailing runs are
- Page faults cost as much as the passes: glibc hands large freed buffers back and faults
  them in again, so document-sized event lists go back to `utils/tokenizer/recycle.rs`
  (a few per thread, within a size band) instead of being dropped, a pass never
  allocates a second list, and `process` takes the thread's last arena, reset, from
  `with_allocator` (`textlane/src/actions/allocator/reuse.rs`, up to 64 MB kept). An arena never frees a
  vector it outgrew, so trees are built at their exact size: the compiler collects open
  nodes' children in one scratch `Vec` and moves them into the arena when the node closes
  (`pop_node`), and `md.toHtml` gives wrapped parents room for their line endings up front
  (`wrap` works in place); an arena past 64 MB is dropped and faulted in again per
  document. Scan bytes with `memchr`, never `slice.contains` per byte. Count faults with `/usr/bin/time -f %R` beside the timing; the
  pprof harness must not blocklist `libc`, or malloc, free and memcpy vanish from it.
  Measure a change A/B against the previous build, interleaved, best of many runs: a
  loaded machine drifts by more than most wins
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
  containers after a line keeps the child's `interrupt`. A new container always closes the
  flow (micromark's `closeFlow`), even with no container to exit, or a pending GFM table head
  sees its line. A setext underline needs a paragraph in the content above, checked once the
  underline matched; after definitions only it fails and its line starts new content
  (`ParseState::content_breaks`). micromark drops a leading byte order mark before it
  tokenizes: `Parser::source` hands the parser the input without it, and positions count from
  there. Text right after a container prefix (`>*a*`) has no character before it Sizes are columns, not bytes: a
  tab after a list marker widens the prefix to the next tab stop. Differences show best
  by fuzzing random fragments through `md.parse` and the pinned from-markdown; each one
  fixed becomes a case in `conformance/sources/differential.ts`
- `md`'s engine is a port of markdown-rs (micromark's design in Rust): a state is a plain
  `fn(&mut Tokenizer) -> State`, a token name a `Name::new("…")` constant, its text listed in `NAMES` (`models/token.rs`) or
  it fails to compile. A construct not
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
  `html.highlight` follows @shikijs/rehype 4.5.0 on giallo 0.5.2 (Shiki's TextMate grammars
  and themes on Oniguruma, `onig-regset`): giallo's tokens unmerged, then textlane joins
  neighbours every theme styles alike, merges whitespace forward and writes Shiki's hast and
  `--shiki-<mode>` styles itself. `helpers/theme_table.rs` holds what giallo's parsed themes
  lose (each theme's `fg`/`bg` as written, colors spelled `#fff` or `#…ff`);
  `helpers/registry.rs` aliases `injectTo` scopes to languages, since giallo links injections
  by language name. giallo exposes no scopes, so Shiki's token-type splits (same color, one
  scope naming `string`) cannot be followed; those cases are pending one by one
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
- `html.raw` ports hast-util-raw 9.1.0 on the same parse5 port, never a second parser: one
  `Parser` (`for_document("")` when the tree starts with a doctype or `<html>`, else the
  template fragment parser) fed by `utils/builder/feed.rs`. Elements, text, comments and
  doctypes become tokens handed straight to the tree builder (`Parser::process`, parse5's
  `_processToken`: a text node is always a character token, never whitespace); each raw
  node is a chunk that is not the last (`Tokenizer::write`, `next_chunk_token`). A read
  past a chunk's end sets the sticky `Preprocessor::chunk_end` and the tokenizer stops
  with its state and half-built token kept, as parse5 hibernates; the explicit
  `_ensureHibernation` sites skip their fallback on it, and the next chunk drops what that
  lookahead left (`<!-` then `- x -->` is the comment `- x --`). `Tokenizer::reset` is
  `resetTokenizer`: pending characters reach the builder only if parse5 would hold a
  `currentLocation` for them (`located`), so a raw `</textarea>` after a text node in a
  `<title>` vanishes, as upstream. No EOF is ever sent. Passed-through nodes enter as empty
  comments whose DOM ids `convert_dom` swaps back (`replacements`); their children are
  separate parse jobs in a queue, converted last first, so nesting never recurses. The
  output has no positions yet: chunks share no input offsets (rehype-raw maps them through
  the raw node's start point)
- `html.format` ports hast-util-format 1.1.0 (`actions/hast/format.rs`): `minify_hast` with
  `newlines`, then a preorder walk whose frames hold each children vector taken out of its
  parent and put back once walked. Output grows with depth times indent by nature, so deep
  stress inputs use `indent: ""`. `html.parse`'s `verbose` builds `data.position` in
  `convert_dom` from the start tag's span and the attribute spans inside it; attributes the
  tree builder adopted from a later tag have none, as in parse5
- `mdx.compile` is a `Compiler` on hast with extension `js`, porting what @mdx-js/mdx 3.1.1
  runs after remark-rehype: hast-util-to-estree (`actions/hast/convert.rs`), recma-document,
  recma-jsx-rewrite, estree-util-build-jsx with recma-build-jsx-transform
  (`actions/program/{wrap,rewrite,build}.rs`) and astring 1.9.0 with estree-util-to-js's JSX
  handlers (`utils/printer/`), compared byte for byte. oxc only parses: each ESM block or
  expression goes through oxc's own ESTree serializer, which writes acorn's shapes (`raw`,
  one `MemberExpression`, `Property`), into `models/estree.rs`, a flat store of ESTree
  objects (type plus ordered fields) the ports walk exactly as upstream walks JSON-like
  trees; field order is walk order. JSX strings are decoded as acorn-jsx does
  (`helpers/entity.rs`); each snippet gets disjoint virtual offsets so comment attachment
  compares within it and `duplicate-layout` maps back to lines (`helpers/place.rs`). Walks
  (`helpers/walk.rs`, estree-walker's enter/leave/skip/replace) and patterns keep explicit
  stacks; conversion, printing and comment attachment recurse under `stacker`. Calls
  indent each nested object, so the printed size of deep JSX grows with depth squared.
  `helpers/{entity,property}_table.rs` come from `pnpm -C conformance generate:mdx-tables`.
  ESTree kinds and keys are `Kind`/`Key` (`models/name.rs`): `u32` indexes into generated
  tables, as md's `Name` is, so a check compares integers and a name missing from the table
  fails to compile; names oxc writes that the tables lack are interned per `Store`, and
  `kind_text` serves astring's first-letter checks. A node holds where its fields lie in
  one shared list and gains a field by moving its fields to the end: a `Vec` per node cost
  an allocation and a pointer chase per lookup. The JSON reader borrows keys and `type`.
  `with_stack` grows with `stacker::maybe_grow`, never `grow`: a fresh mapped stack per
  expression made expression-heavy MDX 4–6× slower
  With `md.mdx` in the pipeline, `md.toHtml` runs remark-mark-and-unravel first (an
  `UnravelSlot` flag either plugin may attach first): a paragraph of only MDX text nodes
  and white space becomes its MDX as flow nodes, which hast cannot recover once wrapped
- `md.gfm` only puts `GfmOptions` in the `GfmSlot` slot; `md.parse` and `md.stringify` read it
  when their factories run at freeze, so GFM's constructs, handlers and unsafe patterns are
  on only with it. A GFM table is tried after each first byte's own flow constructs, as
  micromark tries constructs keyed on `null`, and a row's indent sits before the row token.
  At `[`, text tries the footnote call, then the task list check, then a link
- `md.frontmatter` appends its matters to `FrontmatterSlot` the same way. Each flow line tries
  them before any built-in construct, as micromark puts extensions first; an unclosed matter
  scans to the end with the lines `concrete` and then rewinds, so `---\n- a` is a break and a
  paragraph, as upstream. Events carry no matter, so the compiler takes the first whose fences
  and place match; a matter that found no closing fence remembers where, so `anywhere` stays
  linear. Custom types are `Literal` nodes, as is any unknown JSON node with only a string
  `value`; any other unknown type is an `Unknown` node (mdast and hast alike) keeping its
  fields as `Data` and its children, so `md.toHtml`'s `passThrough` can copy it into hast
- `md.math`, `md.directive` and `md.definitionList` use slots the same way, and each adds a
  flag to `tokenize_input` and `parse_markdown_with`, in that order after `frontmatter`.
  Every mdast node carries unist `data` (`None` unless a tree from JSON or a plugin set it);
  `md.toHtml` applies its `hName`, `hProperties` and `hChildren` after each handler that
  upstream calls `applyData` in (`utils/converter/unknown.rs`), never to definition lists,
  passed-through nodes or a row's cells twice. Math and directives have no handler upstream:
  the unknown handler makes math's `data` its hast and a directive a `div`. A container directive is a concrete flow
  construct whose lines are a nested document (`ContentType::Document`), not a document
  container. A definition description is a document container (`Container::DefList`): its
  check closes the flow even when no list opens, and `Resolver::DefList` splits the
  paragraph above into terms, as micromark-extension-definition-list does
- `md.mdx` ports micromark-extension-mdxjs (JSX, expressions, ESM; indented code,
  autolinks and raw HTML off, so flow constructs take any indent: `ParseState::indent_max`)
  and mdast-util-mdx. Core holds the MDX nodes for mdast and hast alike (`models/mdx.rs`);
  `md.toHtml` passes them into hast unchanged, as @mdx-js/mdx's remark-rehype does. md
  asks acorn's questions through core's `ScriptParser` in `ScriptParserSlot`, which the
  mdx crate implements on oxc (`OxcScriptParser`, a module checked by oxc_semantic too,
  for acorn's early errors) and `Processor::freeze_with` seeds under `mdx`. A syntax
  error is `State::Error`: `Tokenizer::crash` keeps the first one in `ParseState::error`
  and every tokenizer stops; the compiler's `error` does the same for
  mdast-util-mdx-jsx's tag mismatches, found on its own token stack (`stack_tokens`,
  which buffers are not on, as from-markdown's). An expression tries each `}` with the
  parser, as micromark does; a failure before the end of the value
  (`ScriptError::end`) is one no later text fixes, so no `}` after it is tried
  (`MdxState::hard_crash`), and the value grows per event, not per try. oxc parses
  shallow sources on the caller's stack and deeper ones on a grown one, past 1024 open
  brackets not at all. A JSX text element's tags and children go into its parent's
  phrasing items (`phrasing_into`): copying each nested element's text is quadratic
- `prose`'s parser ports parse-latin 7.0.0 and parse-english 7.0.0 (`language: "english"`
  adds parse-english's two plugins in front of parse-latin's). Core's nlcst
  (`models/nlcst.rs`) is one `Node` enum over two shapes, `Parent` (root, paragraph,
  sentence, word) and `Literal` (text, symbol, punctuation, white space, source), with
  positions as byte spans. parse-latin tests each UTF-16 unit, so an astral character is
  always a symbol, and runs of one mark continue only through the same character.
  `utils/parser/` works on `Store`, ids into a flat node list with `VecDeque` children,
  never on arena vectors: upstream's plugins splice next to the visited child, which
  `cursor.rs` makes free by keeping its children split at a gap (`modify_children` reads a
  visitor's return value as unist-util-modify-children does, and an absolute index after
  inserts or removals). `mergeRemainingFullStops` walks backwards with its own gap; a
  sentence break drains the front of the children, not the tail. A position whose end
  parse-latin left unset is dropped on export
- `prose`'s checks (`prose.equality`, `prose.readability`, `prose.passive`,
  `prose.repeatedWords`, `prose.sentenceSpacing`) are nlcst transforms that only add
  messages. A message's place is unist's (lines, UTF-16 columns), but nlcst holds byte spans:
  `FrozenProcessor::parse` keeps the source as `File::value` (vfile's `value`). A check
  returns `Report`s holding positions; `actions/message/place.rs` (`place_messages`) places
  every byte span in one sorted sweep over the source, since `Lines` counts UTF-16 units from
  the start per point (quadratic over many messages on non-ASCII text), then appends them
  sorted by start offset and rule, the order the conformance cases compare. `actions/nlcst/search.rs`
  is nlcst-search: phrases indexed by first word, each parent's words normalized once and its
  first and last word known, so nlcst-is-literal never rescans siblings. The pattern and
  word-list tables (`helpers/*_table.rs`) are generated from the pinned packages by
  `pnpm -C conformance generate:prose-tables`, never edited; syllable 5.0.1's two expressions
  with a back reference or lookahead are matched by hand in `actions/syllable/count.rs`, and
  pluralize's singular rules run on `regex` from their JavaScript source
- Bridges into nlcst (`md.toProse`, `html.toProse`) live in md and html, so they reach the
  prose parser through core: `ProseParser` (`tokenize`, `tokenize_paragraph`,
  `tokenize_root`, the three steps mdast-util-to-nlcst and hast-util-to-nlcst call on
  parse-latin) in `ProseParserSlot`, which `Processor::freeze_with` seeds with the Latin
  `LanguageParser` before any plugin attaches. A bridge's `attach` reads it from the slots
  and fails with `missing-prose-parser` when textlane is built without `prose`. They port
  mdast-util-to-nlcst 7.0.1 and hast-util-to-nlcst 4.0.0, which place tokens by UTF-16
  offsets into the document (`patch`), so they read the text the parser saw from
  `File::value` and place through core's `Location` (vfile-location: `to_point`,
  `to_offset`, `place_nlcst`), byte spans where an offset starts a character. A value
  that differs from its source (an escape, a lazy line) drifts exactly as upstream's does.
  Both walk with explicit stacks: md one frame per parent, remembering the previous
  child's end for the line endings between; html a task stack of `find` and `add`, with
  `implicit` grouping phrasing runs up front and `flattenAll` and `toString` iterative
- Prose checks (`prose.quotes`, `prose.contractions`, `prose.indefiniteArticle`) collect
  `Report`s with span positions and hand them to `place_messages`, which turns them into
  unist places in one sorted pass over `File::value` (the text the parser read, set by
  `FrozenProcessor::parse`): a transform never sees the input otherwise, and `Lines`
  counts UTF-16 from the start per point, so `helpers/place.rs` `to_place` (a fresh `Lines`
  per call) is for one-off places only. Word walks go through `helpers/words.rs`, which
  finds each parent's first and last word once, so nlcst-is-literal's and
  retext-indefinite-article's sibling scans stay linear. `prose.smartypants` rebuilds each
  sibling list instead of splicing, keeping upstream's walk quirks (an ellipsis skips as
  many siblings as it removed). Word lists are generated from the pinned packages
  (`helpers/contraction_table.rs`, `helpers/article_table.rs`), never hand-copied
- `md` names entities by syntax construct or mdast node and verbs by layer: `tokenize` and
  `resolve` (micromark), `compile` (from-markdown), `stringify` (to-markdown), `convert`
  (to-hast), so `actions/heading/compile.rs` exports `compile_heading`. No file named after
  its parent module (`utils/tokenizer/tokenizer.rs`): clippy's `module_inception` rejects it
- `textlane/src/bindings/node.rs` and `python.rs` only map crate functions to their
  language: `version()`, `plugin_names()` and a `Pipeline` class over `models/pipeline.rs`
  whose methods are its calls (`run`/`runSync`, `process`/`processSync` pick `Api::Async` or
  `Api::Sync`), plus `run_case` under `testing`; behaviour never lives there. Node crosses
  values as `serde_json::Value`, Python as JSON text; both turn a crate `Error` into a thrown
  error whose `code` is the crate's (napi's status, an attribute on `TextlaneError`).
  `Pipeline::use_value` reads the bindings' descriptors (`{name, options?}`) as plugins, alone
  or first in a tuple, and `write_file` (`actions/file/write.rs`) shapes `process`'s vfile
- `actions/plugin/find.rs` holds `PLUGINS`, the one table of public plugins: name, builder
  and, under the `schema` feature, its options' `JsonSchema`. `list_plugins` and
  `describe_plugins` (what `examples/schema.rs` prints for `pnpm generate:bindings`) read it,
  so a plugin missing from the table exists for no binding
- Unit tests sit beside their module as `<module>_unit.rs`, wired in with
  `#[cfg(test)] #[path = "<module>_unit.rs"] mod unit;`; one that can fail returns a `Result`
  and uses `?` instead of `unwrap`, which clippy denies

## Features

- The default features need neither Node nor Python: `cargo add textlane` must build without
  napi or PyO3. Every binding dependency is optional and enabled only by its feature.
- `schema` (never default) derives `schemars::JsonSchema` on every options struct, through
  each domain crate's own `schema` feature, and adds `describe_plugins`. Release builds and
  `cargo add textlane` never compile schemars. `pnpm -C rust check:lint` lints it with `node`.
- `testing` adds `run_case` and the `test.*` plugins the conformance suite drives, and exposes
  them through both bindings; it is never default and never published in an addon or wheel.
  `rust/textlane/tests/conformance.rs` requires it (**`conformance-suite`**).
- `md`, `html`, `mdx` and `prose` (all default) gate the domain re-exports. `node` and
  `python` enable every domain, since each binding is one addon that must carry them all.
- `highlight` (default, and on in `node` and `python`) is `textlane-html/highlight`: giallo,
  its 1 MB grammar dump and Oniguruma's C build (`onig_sys`, plus `zstd-sys`), about 2.5 MB of
  addon. The workspace pins `textlane-html` with `default-features = false`, so `html` alone
  builds without a C compiler
- `node` pulls in napi, and `build.rs` runs napi-build only under it; `python` pulls in PyO3
  with `abi3-py39`. Each binding module is compiled only under its feature.
- `pnpm -C rust check:lint` runs clippy three times: default features, `--features node`
  (with `testing` and `schema`) and `--features python` (with `testing`), so CI on every OS
  compiles both bindings before a release can tag.
- `unsafe_code` is `deny` in `[workspace.lints]`. The one exception is `bindings::node`, allowed
  in `lib.rs` because napi's macros expand to unsafe FFI glue; never widen it.
