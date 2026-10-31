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
  `nl/` (`textlane-<domain>`). A domain crate depends on `textlane-core`, never on another
  domain or on `textlane`, which depends on them all. `mdx` compiles core's hast to JavaScript
  and parses `md.mdx`'s JavaScript, both with oxc, so oxc is its own dependency and never
  core's or md's
- A domain is named short (`md`, `html`, `mdx`, `nl`); its functions spell formats out in
  full: `md.toHtml`, `html.toMarkdown`, `md.toNaturalLanguage`. Rust `to_markdown` becomes napi's
  `toMarkdown` and Python's `to_markdown` with no `js_name` override
- `textlane` re-exports each domain as a module of its name and, once `textlane-core` has
  items, those at its root; never as `textlane::core`, which would make every `core::` path in
  the crate ambiguous with Rust's built-in `core`. A glob re-export of a crate with no items
  fails clippy as an unused import
- Shared metadata, the member version pins, the release profile and the lint policy live once
  in `rust/Cargo.toml` (`[workspace.package]`, `[workspace.dependencies]`, `[workspace.lints]`); a
  member inherits each with `.workspace = true`. A new member joins `[workspace.dependencies]`
  and the release-please `extra-files` (**`package-distribution`**)
- `rust/vendor/` holds dependencies textlane patches, wired in through `[patch.crates-io]`
  and excluded from the workspace so its lints and rustfmt leave them alone; only the npm
  and PyPI addons build with them, a crates.io build takes the published crate. Mark each
  change `// textlane:` and drop the copy once upstream ships it. `onig-regset` (giallo's
  Oniguruma binding) takes its global lock for the first `onig_new` only, which initializes
  the encoding: with the lock on every compile, cold highlighting on several threads ran
  slower than on one, and the Node `process` (a napi `AsyncTask`) gained nothing
- In each crate, `src/actions/<entity>/<verb>.rs` exports `verb_noun`
  (`actions/package/version.rs` exports `read_version`); the module tree is declared inline in `lib.rs`, so there is no `mod.rs`
- Beside `actions/`, as in fairspec: `models/` for types, `helpers/` for small free functions,
  `plugins/<name>/` for one `.use()` stage each (`plugin.rs` plus its own `actions/`), and
  `utils/<name>/` for a bounded internal system whose files are mutually recursive (`md`'s
  `onepass`, `serializer`, `converter`); only it is exempt from one action per file
- A plugin is one struct named after it and holding its options as fields, declared in
  `models/` and implementing `Plugin` in its `plugin.rs`; no constructor function beside
  it. Callers write `md::Gfm::default()` or `md::Gfm { single_tilde: false,
..Default::default() }`; a plugin without options is a unit struct (`md::Gemoji`),
  declared in its `plugin.rs`
- Trees live in one `bumpalo` arena per document (`textlane_core::Allocator`, `ArenaVec`);
  strings borrow the input or the arena. Input decides how deep a tree nests, so nothing
  walks one by recursion on the plain stack: `ArenaVec` never drops its items, conversion
  and `html.stringify` keep an explicit stack, `md.stringify`'s handlers recurse inside
  `stacker::maybe_grow`, and JSON stops at 1000 levels with `tree-too-deep`. Data built
  while walking must not nest either (`md.stringify` keeps emphasis in one flat store),
  or dropping it recurses. Watch for
  quadratic passes too: `processor_unit.rs` runs 100,000-deep and 50,000-run inputs,
  and unclosed code spans of growing length. A construct that scans ahead and can fail
  needs a memo of what the failed scan saw: in `md`'s parser, unclosed code and math runs,
  inline HTML (`HtmlMemo`), GFM literal autolinks' trails and domains (`LiteralMemo`) and
  MDX's tries at a `}` and ESM (a parse budget). Per line, work stays O(1) in nesting depth:
  `touch` stops at an ancestor already ending there, blank lines reuse the last container
  match (`blank_match`), an unclosed link bracket's deactivation stops at one already off,
  and runs pair through buckets. A label is matched against definitions only up to the
  longest identifier. The find-and-replace pass for GFM literals after parsing shares one
  domain run and one atext run among the starts inside it, and skips a text with no `@`
  (emails) or no `www.`/`http:` (URLs) found by memchr. The serializer
  never re-indents a container's text: `utils/serializer/sink.rs` records each container as a layer and writes every prefix
  once, deciding a blank line from the innermost prefix out. Nested emphasis renders into
  one shared token buffer, and a delimiter run's token list is shared, not copied per
  token. Time
  growth in release mode, best of several runs: single debug timings are noisy.
- `process` takes the thread's last arena, reset, from `with_allocator`
  (`textlane/src/actions/allocator/reuse.rs`), as glibc hands large freed buffers back and
  faults them in again. It keeps what `reset` leaves (the largest chunk) up to
  `set_retained_memory`, 256 MB by default; trees take 15–75 bytes per input byte, so a
  64 MB cap dropped the arena of a 4 MB document every time and cost it 22–45%. An arena never frees a vector it outgrew, so trees
  are built at their exact size where it is known, and `md.toHtml` gives wrapped parents
  room for their line endings up front (`wrap` works in place); an arena past the cap is
  dropped and faulted in again per document. Scan bytes with `memchr`, never
  `slice.contains` per byte. Count faults with `/usr/bin/time -f %R` beside the timing;
  the pprof harness must not blocklist `libc`, or malloc, free and memcpy vanish from it.
  Measure a change A/B against the previous build, interleaved, best of many runs: a
  loaded machine drifts by more than most wins
- `md.toHtml`'s `breaks` is remark-breaks as its own mdast stage ahead of the conversion
  (`plugins/to_html/actions/text/split.rs`), real `break` nodes, so the direct writer and
  mdast-util-to-hast's trimming after a break see them as upstream's tree has them
- `process` skips hast when the last stage is `md.toHtml` and the compiler `html.stringify`
  with no option but `allowDangerousHtml` set: core's `Transform::html_writer` and
  `Compiler::html_target` let `FrozenProcessor::finish` hand the mdast to `md`'s
  `utils/writer`, which writes those two stages' bytes directly (19–30% off `process`) and
  gives up, to the hast path, on any node it does not know, any `data`, an undefined
  reference or footnotes. A change to a `convert` handler changes what the writer must
  write: `textlane/tests/html_writer.rs` compares both over every Markdown input of the
  conformance suite and fails on any byte. A stage timing must reuse one reset arena, as
  `process` does: a fresh one per run measures page faults, which tripled `md.toHtml`
- The hast path is memory-bound, not instruction-bound: on a 1 MB README it runs about 1.7
  instructions a cycle against the direct writer's 2.5, so trimming instructions
  (`apply_data` inlined, `parse_inline`'s items pre-sized) saved 1% of instructions and no
  cycles. Its lever is node size (112 bytes, moved about three times). Under load, A/B with
  `perf stat -e instructions:u,cycles:u`, which load does not skew as wall time does. Keep
  short-lived working lists on the heap: moving `parse_inline`'s into the arena saved 5% of
  instructions but cost 16% of cycles and 55% more cache misses, as the heap hands back the
  block just freed, still hot, and the arena only ever hands out cold memory. Converting
  hast children straight into their parents' vectors saved 2% of instructions and no
  cycles, as the misses come from the tree's size, not that copy
- Container ends follow micromark, not markdown-rs: a list item ends at the first line
  ending of its trailing blank lines (`prepareList`), stepping over the exits of lists
  inside but not of block quotes; a list does the same without stepping over block quote
  prefixes. Trailing blank lines never make an item loose, nor does the blank line that
  closes empty containers opened on its own line (micromark puts it inside them). An item
  whose last construct swallowed its line ending (an unclosed fence) ends after the next
  item's prefix. A `Point` is a byte offset whose top two bits hold a tab's virtual
  spaces; `Lines` turns it into unist's UTF-16 line, column and offset
- `md.parse` is `utils/onepass/`, a cmark-style parser that builds mdast with no event
  list: `block.rs` takes each line through a stack of open blocks, leaves keeping their
  lines, with `FlowText` reading ahead through container prefixes for MDX's multi-line
  constructs; `inline.rs` runs once per leaf: items, then pairing runs, then the tree
  (`skip_plain` jumps over plain text through `INLINE_SPECIAL`, so a byte given an arm in
  `scan` must join that table, or its construct is skipped over; under GFM it also stops at
  `w`/`h` and walks byte by byte only through the atext run before the next `@`, where an
  email may start);
  `build.rs` turns blocks into nodes, groups definition lists and assembles JSX elements.
  It reproduces micromark's and from-markdown's trees and positions, quirks included:
  `interrupt` is decided once a line, before any container opens on it; unclosed fences
  and HTML take their last line ending unless the next line is lazy; a setext underline
  after definitions only starts new content; a description line never continues a
  paragraph lazily; GFM's `~` and emphasis pair in the order their kind first appears,
  the second inside the first's nodes. micromark drops a leading byte order mark:
  `Parser::source` hands the parser the input without it. Text right after a container
  prefix (`>*a*`) has no character before it. Sizes are columns, not bytes. Where unified
  differs from CommonMark's prose, the corpus wins (an escape's text starts at its
  backslash; a token's slice leaves out container prefixes of its later lines and reads
  NUL as U+FFFD; a label keeps its continuation indent, a title and inline HTML do not;
  an email literal tokenizes in text, then mdast-util-gfm-autolink-literal's
  find-and-replace links what micromark left; a task check goes in an item's first
  content). Neighbours of attention and strikethrough classify as micromark classifies a
  UTF-16 code unit (`classify_character_unit`). Differences show best by fuzzing random
  fragments through `md.parse` and the pinned from-markdown; each one becomes a case in
  `conformance/sources/differential.ts`. `helpers/entities.rs` and `helpers/unicode.rs`
  are tables generated from markdown-rs's, sorted for binary search
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
  `html.sanitize` and `html.slug` port hast-util-sanitize and rehype-slug (its `autolink`,
  rehype-autolink-headings 7.1.0, a second pass after slugging in `actions/hast/autolink.rs`,
  skipping what `before`/`after` spliced in as upstream's visitor does); the sanitizer
  walks with its own stack and counts open ancestors instead of searching them. The slugger
  is github-slugger in core (`textlane_core::Slugger`, `core/src/helpers/slug.rs`), shared by
  `html.slug` and `md.toc`, since md cannot depend on html; `helpers/slug_table.rs` beside it
  is github-slugger's removal regex as code-point ranges, hand-ported with no generator
  `html.highlight` follows @shikijs/rehype 4.5.0 on giallo 0.5.2 (Shiki's TextMate grammars
  and themes on Oniguruma, `onig-regset`): giallo's tokens unmerged, then textlane joins
  neighbours every theme styles alike, merges whitespace forward and writes Shiki's hast and
  `--shiki-<mode>` styles itself. `helpers/theme_table.rs` holds what giallo's parsed themes
  lose (each theme's `fg`/`bg` as written, colors spelled `#fff` or `#…ff`, the
  `terminal.ansi*` colors), generated by `pnpm -C conformance generate:highlight-tables`;
  `helpers/registry.rs` aliases `injectTo` scopes to languages, since giallo links injections
  by language name. giallo exposes no scopes, so Shiki's token-type splits (same color, one
  scope naming `string`) cannot be followed; those cases are pending one by one. `ansi`
  bypasses giallo: `actions/ansi/tokenize.rs` is Shiki's `tokenizeAnsiWithTheme` over
  `utils/ansi/`, a port of ansi-sequence-parser 1.1.3 (one state across lines, a
  sequence's resets before its sets, codes read with JavaScript's `parseInt`/`Number`) and
  its palette (`terminal.ansi*`, else Shiki's `defaultAnsiColors`; `dimColor`). Its tokens
  keep apart where styles agree, a dim color past the 256-color table fails as Shiki does. The
  transformers port @shikijs/transformers over `models/highlight.rs`'s `CodeBlock`, never
  over hast: line hooks (`metaHighlight`, `renderLineNumber`) per line, then the notation
  `code` hooks sharing one `parseComments` (`utils/notation/`, matching `v3`). A notation
  marks tokens `None` and lines `removed` instead of splicing, so the indexes parsed comments
  hold stay valid; Shiki's line ending after a removed line goes with it. giallo's time grows with the square of a
  line's length and cannot be interrupted, so `tokenizeMaxLineLength` (2,000 by default,
  Shiki's 0) drops long lines from the code giallo sees, which keeps the grammar state passing
  them by as Shiki's does. `metaWordHighlight`
  ports Shiki's decorations engine (`actions/code/decorate.rs`): tokens split at UTF-16
  offsets before the hooks, decorations applied to the finished hast last, as Shiki's own
  transformer runs after every caller's. Cases come from the
  pinned packages through `import:highlight` (`highlight-transformers.json`, `highlight-ansi.json`)
  `html.katex` follows rehype-katex 7.0.1 on katex-rs 0.3.0 (KaTeX 0.18.5): KaTeX's markup
  goes back through the HTML parser, as `fromHtmlIsomorphic` does. katex-rs keeps attributes
  in a randomly seeded hash map, so every element's are sorted (`class`, `style`, the rest by
  name) or output would change per run; the conformance importer sorts rehype-katex's the
  same way. katex-rs recurses once per group or style switch with no depth check, and a
  stack overflow aborts the host: `measure_levels` bounds the depth from the source (a group
  or `[` four levels, every control word one until its group closes, since `\displaystyle`
  nests without braces; macro bodies × `maxExpand` once any macro exists, `\gdef`s of earlier
  formulas included) and renders on a stack grown to fit, reporting `rangeerror` past 4096
  levels as rehype-katex reports KaTeX.js's `RangeError`. The bound fails closed: it skips
  nothing (KaTeX reads `%` in `\url` literally), no `]` closes, and a closer in a comment or
  `\verb` closes nothing, since a closer the parser never sees would lower it. Its
  whole-formula error span is rebuilt as KaTeX.js words it. Rendering bugs of katex-rs are
  pending one by one
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
  stress inputs use `indent: ""`. Its `minify` is rehype-minify-whitespace: `minify_hast`
  alone, no walk, its cases hast-util-minify-whitespace's trees. `html.parse`'s `verbose` builds `data.position` in
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
  trees; field order is walk order. An MDX node carrying `data.estree` is read from it
  instead (`actions/estree/load.rs`, its offsets moved into a range of their own), as
  hast-util-to-estree reads nothing else: JavaScript plugins (remark-mdx-frontmatter) build
  ESM that way with `value` empty; native `md.mdx` stores none, so `value` is parsed
  otherwise. JSX strings are decoded as acorn-jsx does
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
- `md.gfm` only puts its options (the `md::Gfm` struct itself) in the `GfmSlot` slot; `md.parse` and `md.stringify` read it
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
  With `data` or `strip` it also adds a transform over the first root child when it is a
  `yaml` node (vfile-matter's regex reads the file, so `--- ` with trailing spaces differs):
  `utils/yaml/` composes yaml-rust2's pull events into a flat node store with an explicit
  stack (yaml-rust2's own `load` recurses), then ports the `yaml` package's `toJS`: core
  schema scalars, numbers through JavaScript's `String(number)` so JSON reads them back as
  `JSON.stringify` wrote them, array-index keys first, `maxAliasCount` 100 with
  `getAliasCount`, duplicate keys by resolved value. Depth stops at 512 (Python 3.9's `json`
  fails near 1000); every failure is `invalid-frontmatter`
- `File::data` is vfile's `data`, a JSON map native plugins store results in
  (`html.extractToc` puts `toc`); `write_file` hands it back, and the loaders lay what their
  host plugins stored over it. `md.headingId`, `md.strip`, `html.unwrapImages`,
  `html.extractToc` and `html.externalLinks` port remark-heading-id, strip-markdown, rehype-unwrap-images,
  @stefanprobst/rehype-extract-toc and rehype-external-links (JSON options only; `content` read
  as hast JSON, so html depends on core's `json`) with explicit stacks (`import:site-plugins` renders their
  cases); `md.strip` handles each sibling list once, a handler's children going back on that
  list's pending stack, never recursion. `md.headingId`'s `attributes` reuses
  `md.directive`'s `scan_attributes`, trying ` {` from the right so a hostile run of braces
  fails fast; `markers` strip literal suffixes repeatedly, longest first, while the
  attribute list (or `{#id}`) is read once. `html.extractToc`'s `skip` compares attributes
  through property-information, so `data-step` and `dataStep` match
- `md.toc` ports remark-toc 9.0.0 over mdast-util-toc 7.1.0 (`import:toc`, which also runs
  their fixtures): every heading is slugged in document order, ids `md.headingId` stored in
  `data.hProperties.id` win over the text, and the list is built in a flat store of lists
  and items, `spread` set bottom-up along each insert's path. `heading` and `skip` are
  JavaScript regex source: `actions/expression/compile.rs` translates them for `regex`
  (ASCII `\d`/`\w`/`\b`, JavaScript's `\s` and `.`, Annex B escapes, `[]`/`[^]`, a lone
  `{`) and refuses lookaround and back references; case folding stays Unicode's, and `.`
  takes a code point, not a UTF-16 unit. `parents` is unist-util-is as JSON (`ParentTest`),
  fields compared as `===`; a closing heading before the opening's index repeats the
  children between, as upstream's slices do, by `copy_nodes` (`CopyMode::Whole`)
- A checkbox is written twice over: by `actions/list_item/convert.rs` into hast and by
  `utils/writer/state.rs` straight into HTML; an option that changes one (`taskListLabels`)
  changes both, and `conformance/tests/md/task-list-labels.json` runs both paths
- `md.pathRewrite` is textlane's own (`conformance/tests/md/path-rewrite.json` is its
  specification): pure path arithmetic over `File::path`, which a test's `path`, the loaders'
  `process({ value, path })` and `FrozenProcessor::process_file` set; a URL it cannot place
  (no path, outside `root`) stays as written
- `md.component` is textlane's own plugin, with no upstream: `conformance/tests/md/component.json`
  is its specification. It walks with a stack of sibling lists, converting each list's items
  before pushing their children, so a list groups (`actions/children/group.rs`) once its own
  items are elements and nesting never recurses. A rule that fails `where` or `required`
  hands the directive back (`DirectiveParts`) for the next rule, so read the label and build
  attributes before dropping anything. Attribute order is part of the output: `nameAs`, then
  `labelAs` unless listed, then the list, the order livemark's components were written against.
  Its code rules (`codes`) mark matched blocks with `Mark::Code`; a code group's wrapper reads
  and strips meta pairs (`actions/meta/read.rs`), its blocks staying code. A rule's `treeAs` reads a container's lists into a JSON file tree
  (`actions/tree/read.rs`, recursing under `stacker`) and drops them from the content
- `md.packageCommand` ports npm-to-yarn 3.2.0's `convert` per line of an `npm` block
  (`plugins/package_command/actions/<manager>/convert.rs`, tables in
  `helpers/command_table.rs`), quirks included: `npm` matches inside `pnpm`, a bare `npm`
  becomes `npm undefined`, bun replaces the first `=`. Only npm itself deviates: a line no
  executor leads stays, since upstream reads it as yarn's. Prototype names (`npm toString`)
  are not ported. `pip` and `cargo` are textlane's own tables (`PipEntry`, one column per
  manager, `None` keeping the line); `import:package-command` renders npm cases through the
  pinned package and spells the others out
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
  autolinks and raw HTML off, so flow constructs take any indent) and mdast-util-mdx. Core
  holds the MDX nodes for mdast and hast alike (`models/mdx.rs`); `md.toHtml` passes them
  into hast unchanged, as @mdx-js/mdx's remark-rehype does. md asks acorn's questions
  through core's `ScriptParser` in `ScriptParserSlot`, which the mdx crate implements on
  oxc (`OxcScriptParser`, a module checked by oxc_semantic too, for acorn's early errors)
  and `Processor::freeze_with` seeds under `mdx`. A syntax error micromark throws while
  tokenizing beats any mdast-util-mdx-jsx raises while compiling (`MdxErrors`): the block
  pass stops at the first, the inline passes record theirs and the tree's tag mismatches
  after. An expression tries each `}` with the parser, as micromark does; a failure before
  the end of the value (`ScriptError::end`) is one no later text fixes, so no `}` after it
  is tried. oxc parses
  shallow sources on the caller's stack and deeper ones on a grown one, past 1024 open
  brackets not at all. A JSX text element's tags and children go into its parent's
  phrasing items (`phrasing_into`): copying each nested element's text is quadratic
- `nl`'s parser ports parse-latin 7.0.0 and parse-english 7.0.0 (`language: "english"`
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
- `nl`'s checks (`nl.equality`, `nl.readability`, `nl.passive`,
  `nl.repeatedWords`, `nl.sentenceSpacing`) are nlcst transforms that only add
  messages. A message's place is unist's (lines, UTF-16 columns), but nlcst holds byte spans:
  `FrozenProcessor::parse` keeps the source as `File::value` (vfile's `value`). A check
  returns `Report`s holding positions; `actions/message/place.rs` (`place_messages`) places
  every byte span in one sorted sweep over the source, since `Lines` counts UTF-16 units from
  the start per point (quadratic over many messages on non-ASCII text), then appends them
  sorted by start offset and rule, the order the conformance cases compare. `actions/nlcst/search.rs`
  is nlcst-search: phrases indexed by first word, each parent's words normalized once and its
  first and last word known, so nlcst-is-literal never rescans siblings. The pattern and
  word-list tables (`helpers/*_table.rs`) are generated from the pinned packages by
  `pnpm -C conformance generate:nl-tables`, never edited; syllable 5.0.1's two expressions
  with a back reference or lookahead are matched by hand in `actions/syllable/count.rs`, and
  pluralize's singular rules run on `regex` from their JavaScript source
- Bridges into nlcst (`md.toNaturalLanguage`, `html.toNaturalLanguage`) live in md and html, so they reach the
  natural-language parser through core: `NaturalLanguageParser` (`tokenize`, `tokenize_paragraph`,
  `tokenize_root`, the three steps mdast-util-to-nlcst and hast-util-to-nlcst call on
  parse-latin) in `NaturalLanguageParserSlot`, which `Processor::freeze_with` seeds with the Latin
  `LanguageParser` before any plugin attaches. A bridge's `attach` reads it from the slots
  and fails with `missing-natural-language-parser` when textlane is built without `nl`. They port
  mdast-util-to-nlcst 7.0.1 and hast-util-to-nlcst 4.0.0, which place tokens by UTF-16
  offsets into the document (`patch`), so they read the text the parser saw from
  `File::value` and place through core's `Location` (vfile-location: `to_point`,
  `to_offset`, `place_nlcst`), byte spans where an offset starts a character. A value
  that differs from its source (an escape, a lazy line) drifts exactly as upstream's does.
  Both walk with explicit stacks: md one frame per parent, remembering the previous
  child's end for the line endings between; html a task stack of `find` and `add`, with
  `implicit` grouping phrasing runs up front and `flattenAll` and `toString` iterative
- Natural-language checks (`nl.quotes`, `nl.contractions`, `nl.indefiniteArticle`) collect
  `Report`s with span positions and hand them to `place_messages`, which turns them into
  unist places in one sorted pass over `File::value` (the text the parser read, set by
  `FrozenProcessor::parse`): a transform never sees the input otherwise, and `Lines`
  counts UTF-16 from the start per point, so `helpers/place.rs` `to_place` (a fresh `Lines`
  per call) is for one-off places only. Word walks go through `helpers/words.rs`, which
  finds each parent's first and last word once, so nlcst-is-literal's and
  retext-indefinite-article's sibling scans stay linear. `nl.smartypants` rebuilds each
  sibling list instead of splicing, keeping upstream's walk quirks (an ellipsis skips as
  many siblings as it removed). Word lists are generated from the pinned packages
  (`helpers/contraction_table.rs`, `helpers/article_table.rs`), never hand-copied
- `md` names entities by syntax construct or mdast node and verbs by layer: `stringify`
  (to-markdown), `convert` (to-hast), so `actions/heading/convert.rs` exports
  `convert_heading`. No file named after its parent module (`utils/onepass/onepass.rs`):
  clippy's `module_inception` rejects it
- `textlane/src/bindings/node.rs` and `python.rs` only map crate functions to their
  language: `version()`, `plugin_names()` and a `Pipeline` class over `models/pipeline.rs`
  whose methods are its calls (`run`/`runSync`, `process`/`processSync` pick `Api::Async` or
  `Api::Sync`), plus `run_case` under `testing`; behaviour never lives there. Node crosses
  options and data as `serde_json::Value` and trees as JSON text, since napi refuses a
  `Value` past 1M nodes or 128 levels (`MAX_JSON_NODES`, private, no opt-out); Python crosses
  all as JSON text. Both read it with `read_json` (`actions/json/read.rs`), which lifts
  serde_json's 128-level limit (~60 nested nodes) through `serde_stacker` but refuses past
  2048 levels with `tree-too-deep`: core's 1000-node cap, two levels a node, since a deeper
  `Value` overflows the stack on drop. A debug addon already overflows ~700 nodes deep in
  `serialize_mdast`; check depth in a release build. Both turn a crate `Error` into a thrown
  error whose `code` is the crate's (napi's status, an attribute on `TextlaneError`).
  `Pipeline::use_value` reads the bindings' descriptors (`{name, options?}`) as plugins, alone
  or first in a tuple, and `write_file` (`actions/file/write.rs`) shapes `process`'s vfile
- `process` leaves the caller's thread: `Pipeline::prepare_process` hands back a
  `ProcessJob` (the frozen processor's `Arc`), which Node runs as a napi `AsyncTask` and
  Python inside `py.detach`, the GIL released. Both take `&self`: napi keeps a `&mut` borrow
  until the task settles and PyO3 one while detached, so a second call would fail as already
  borrowed. The loaders therefore freeze the native pipeline once, under a lock in Python,
  before the first `process`; the Python `Pipeline` class is sendable for it. Pipelines with
  host plugins stay on the calling thread, through `start_process`
- Host plugins (a JavaScript or Python function) are `{host: id}` descriptors; their options
  never cross, since they may hold functions, so the loader keeps and merges them as unified's
  `addPlugin` does and the crate only orders and switches them (`Pipeline::hosts`). Each becomes
  a `models/host.rs` stage whose `Transform::host()` is `Some(id)`: freeze skips it in the kind
  check, and `FrozenProcessor::advance` stops before a batch of them. `start_process` /
  `start_run` hand back `Progress`; a `Pause` (`models/run.rs`) owns the tree as JSON, the file
  and an `Arc` of the frozen processor, never an arena, so a binding can await between
  `resume`s. The napi and PyO3 `Run` classes wrap it; the loader's `finishRun`
  (`typescript/actions/run/finish.ts`) and `python/textlane/_host.py` drive the loop. The plain
  `run`/`process` fail with `missing-host` on a host stage
- `actions/plugin/find.rs` holds `PLUGINS`, the one table of public plugins: name, builder
  and, under the `schema` feature, its options' `JsonSchema` and its unified upstream.
  `list_plugins` and `describe_plugins` (what `examples/schema.rs` prints for
  `pnpm generate:bindings`) read it, so a plugin missing from the table exists for no binding
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
- `md`, `html`, `mdx` and `nl` (all default) gate the domain re-exports. `node` and
  `python` enable every domain, since each binding is one addon that must carry them all.
- `highlight` (default, and on in `node` and `python`) is `textlane-html/highlight`: giallo,
  its 1 MB grammar dump and Oniguruma's C build (`onig_sys`, plus `zstd-sys`), about 2.5 MB of
  addon. The workspace pins `textlane-html` with `default-features = false`, so `html` alone
  builds without a C compiler
- `katex` (default, and on in `node` and `python`) is `textlane-html/katex`: katex-rs, pinned
  exactly, pure Rust, about 2.5 MB of binary with its font metrics and symbol tables
- `node` pulls in napi, and `build.rs` runs napi-build only under it; `python` pulls in PyO3
  with `abi3-py39`. Each binding module is compiled only under its feature.
- `pnpm -C rust check:lint` runs clippy three times: default features, `--features node`
  (with `testing` and `schema`) and `--features python` (with `testing`), so CI on every OS
  compiles both bindings before a release can tag.
- `unsafe_code` is `deny` in `[workspace.lints]`. The one exception is `bindings::node`, allowed
  in `lib.rs` because napi's macros expand to unsafe FFI glue; never widen it.
