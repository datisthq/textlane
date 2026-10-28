# Textlane packages and conformance suite

Design for the package layout of textlane and the language-neutral conformance suite every
implementation is built against. Plugin behaviour (parsers, transforms) is out of scope here:
this fixes the structure everything else lands in.

## Intent

Textlane replaces unified: the pipeline and its most important plugins run natively in Rust,
existing JavaScript plugins still join a pipeline, and the same crate ships to crates.io, npm
and PyPI. The first consumer is livemark, which today runs unified through `compileMDX` with
remark-gfm, gemoji, definition lists, math, directives, ~22 local `remark-*` plugins and
rehype-slug, katex and shiki, plus three small unified pipelines
(`helpers/render-inline-markdown.ts`, `helpers/toc.ts`, `actions/changelog/build.ts`).

The conformance suite is the contract: a pipeline, an input and the expected result, in the
style of JSON-Schema-Test-Suite, run by the Rust, Node and Python implementations alike. Each
implementation keeps its own unit tests for internals.

## Goals

- One cargo workspace of a few large crates, one per content domain, with a shared core
- One package per registry (`textlane` on crates.io, npm and PyPI), with a subpath per domain
- A conformance suite whose format, layout and runners exist before the first plugin does,
  seeded from the upstream tests catalogued below
- Textlane's own parser: markdown-rs is a behavioural reference and a test source, never a
  dependency

## Non-goals

- MDX, KaTeX and Shiki output parity in the first phases
- Porting upstream tests that assert JavaScript semantics: object identity, `this`, argument
  counts, trough's callback/promise internals, TypeScript type tests
- Rebuilding the whole unified ecosystem: the target is livemark's plugin set, then breadth

## Anti-goals

- A binding that reimplements behaviour, or one npm/PyPI package per domain: separate addons
  cannot share a tree without serializing it at every boundary
- Corpus expectations copied per implementation, or tests that assert Rust internals
- Renaming `remark-*`/`rehype-*` JavaScript plugins out of reach: they must still `.use()`

## API summary

### Crates (`rust/`, flat, fairspec-style)

`rust/Cargo.toml` is a virtual manifest; every crate is a sibling directory named by its role.
Shared metadata, member version pins (`[workspace.dependencies]`), lints and the release
profile live there once. All crates are published, lock-stepped at one version.

| crate            | dir         | depends on | holds                                                                                                                                                                                                                                                                                                       |
| ---------------- | ----------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `textlane`       | `textlane/` | all below  | the processor (`.use`, presets, freeze, `parse`/`run`/`stringify`/`process`, sync and async), the host-language plugin bridge, the `node`/`python` bindings, the conformance test plugins behind a `testing` feature; re-exports `core`'s items at its root and the domains as `md`, `html`, `mdx`, `prose` |
| `textlane-core`  | `core/`     | —          | unist, mdast (with GFM, math, directive, frontmatter, definition list and MDX nodes), hast and nlcst trees; the file (vfile) and its messages; the plugin contract; visit/is/position utilities                                                                                                             |
| `textlane-md`    | `md/`       | core       | `parse`, `stringify`, `gfm`, `frontmatter`, `directive`, `math`, `definitionList`, `gemoji`, `toc`, `mdx` (syntax only: JSX, expressions and ESM found by balanced scanning, kept as source strings), `toHtml` (mdast → hast), `toProse` (mdast → nlcst)                                                    |
| `textlane-html`  | `html/`     | core       | `parse` (HTML5), `stringify`, `raw`, `sanitize`, `slug`, `format`, `toMarkdown` (hast → mdast), `toProse` (hast → nlcst); later `katex` and `highlight` behind features                                                                                                                                     |
| `textlane-mdx`   | `mdx/`      | core, oxc  | `compile`: hast into a JS tree on oxc (upstream rehype-recma), the MDX transforms (document, JSX rewrite, build JSX) and code generation; ESTree conversion so host recma plugins can join                                                                                                                  |
| `textlane-prose` | `prose/`    | core       | `parse` (English/Latin → paragraphs, sentences, words), `stringify`, then `smartypants`, `readability`, `spell`, `repeatedWords`, `passive`, `equality`, `contractions`, `indefiniteArticle`, `sentenceSpacing`, `quotes`                                                                                   |

Rules:

- A domain crate depends on `textlane-core` only, never on another domain or on `textlane`.
  Every tree type but the JS tree lives in core, so the bridges (`md.toHtml`,
  `html.toMarkdown`, `*.toProse`) need no domain-to-domain edge. The JS tree is oxc's and
  stays inside `mdx`, the only crate that compiles oxc.
- Domains are named short, like file extensions (`md`, `html`, `mdx`), with `prose` for
  natural language; functions spell formats out in full: `md.toHtml`, `html.toMarkdown`,
  `md.toProse`. Each name derives mechanically from the Rust one: `to_markdown` is
  `toMarkdown` through napi's default conversion and `to_markdown` in Python, with no
  `js_name` overrides.
- MDX is split by job, as upstream splits it between remark-mdx and `@mdx-js/mdx`: its syntax
  lives in `md` (it changes how Markdown tokenizes), its compilation in `mdx`. Livemark's
  `compileMDX` becomes `md.parse` → `md.mdx` → … → `md.toHtml` → `html.*` → `mdx.compile`.
- `textlane` features: `md`, `html`, `mdx`, `prose`, all default; `node` and `python` enable
  every domain, since each binding is one addon; `testing` adds the test plugins.
- `textlane-core` is never re-exported as `textlane::core`, which would make every `core::`
  path ambiguous with Rust's built-in `core`.

### Bindings

| Rust                             | npm                                                | Python                                   |
| -------------------------------- | -------------------------------------------------- | ---------------------------------------- |
| `textlane::{Processor, File, …}` | `import { textlane } from "textlane"`              | `from textlane import textlane`          |
| `textlane::md::parse()`          | `import * as md from "textlane/md"` → `md.parse()` | `from textlane import md` → `md.parse()` |
| `textlane::html::sanitize()`     | `textlane/html`                                    | `textlane.html`                          |
| `textlane::prose::parse()`       | `textlane/prose`                                   | `textlane.prose`                         |
| `textlane::mdx::compile()`       | `textlane/mdx`                                     | `textlane.mdx`                           |

```ts
import { textlane } from "textlane"
import * as md from "textlane/md"
import * as html from "textlane/html"

const processor = textlane()
  .use(md.parse())
  .use(md.gfm())
  .use(md.toHtml())
  .use(html.sanitize())
  .use(html.stringify())
```

Plugin names (`md.gfm`), their options and the node shapes are public API: the corpus
names them, so renaming one is a breaking change everywhere at once.

## Conformance suite

### Layout

Mirrors fairspec's `conformance/`:

```
conformance/
  README.md
  test-schema.json               JSON Schema of a test file
  tests/<area>/<construct>.json  e.g. core/use.json, md/heading-atx.json, html/sanitize.json
  sources/                       importers that turn upstream tests into test files
```

`<area>` is `core`, `md`, `html`, `mdx`, `prose`; a file holds one construct with a group per source. The suite belongs to no package and is never
shipped.

### Test files

An array of groups; a group fixes the pipeline, its tests vary the input:

```json
[
  {
    "description": "atx headings",
    "source": {
      "repository": "https://github.com/micromark/micromark",
      "commit": "6577c20",
      "path": "test/io/flow/heading-atx.js",
      "license": "MIT"
    },
    "pipeline": [["md.parse"], ["md.toHtml"], ["html.stringify"]],
    "tests": [
      { "description": "rank 1", "input": "# a", "expected": { "html": "<h1>a</h1>" } },
      {
        "description": "no rank 7",
        "input": "####### foo",
        "expected": { "html": "<p>####### foo</p>" }
      }
    ]
  }
]
```

- **Group**: `description`, `tests`, `pipeline` (a list of `[name, options?]`); optional
  `comment`, `specification` (links), `source` (repository, commit, path, test name, licence —
  required for anything imported).
- **Test**: `description`, an input and exactly one expectation kind.
  - Input: `input` (a string) or `tree` (a JSON tree, for stringify-only and transform cases).
  - `expected.html`, `expected.md`, `expected.js`, `expected.txt`: the serialized output,
    keyed by its file extension, never by the domain that produced it.
  - `expected.tree`: the resulting tree; `positions: false` on the group or test strips
    positions before comparing. `offset` and `column` count UTF-16 code units, as unified's
    do, so upstream trees import verbatim and host plugins see unified-identical positions;
    the crate tracks bytes internally and converts at the edge (decided).
  - `expected.messages`: compared on `ruleId`, `source` and `place` only; reason text is not
    contract.
  - `expected.error`: a stable code plus an optional substring, never a JS regex.
- **Status**: a test may carry `status: "deviation"` with a `comment` when textlane
  deliberately differs from upstream, so an intentional difference is never a failure.

### Pending

The corpus is filled before the implementation. `conformance/pending.json` maps a file, a group
or an area wildcard (`md/*`) to why it does not pass yet. Every runner runs pending cases and
tolerates their failures; an entry whose cases all pass fails the run until it is removed (a
wildcard only once its whole area passes), so the list only shrinks. Ported groups follow
upstream's grouping, one per spec section or test block.

### Comparison

Exact, always: strings byte for byte, trees as JSON. There is no HTML-equivalence mode.
micromark's tests assert micromark's direct HTML compiler, while textlane only has the
`toHtml` → `html.stringify` path, so the importers regenerate those expectations by running
the reference unified pipeline (`remark-parse` → `remark-rehype` → `rehype-stringify`) once
(decided; see Importers).

### Test plugins (core only)

The unified core tests are about lifecycle. They become data once the suite defines built-in
test plugins, written once in Rust in `textlane` behind its `testing` feature (my call): every
runner reaches the same code, the Node and Python runners through an addon built with
`--features node,testing` (`TEXTLANE_ADDON`) or `python,testing`, and nothing ships them. The
host-plugin bridge is tested separately, by host-language cases added with the bridge:
`test.parseText`, `test.compileValue`, `test.recordOptions`, `test.replaceTree`,
`test.fail`, `test.noop`, `test.appendNode`, `test.message`, `test.setData`, `test.trace`,
each with a `mode` (sync, `next`, deferred, async, promise) where it applies. Core cases add an
`operations` list (`use`, `freeze`, `data`, `parse`, `run`, `process`, with `api: sync |
async`) in place of a single pipeline. The processor's async model (runtime, awaiting a JS
promise through napi, asyncio) is the processor phase's design; until then a runner skips the
modes it cannot express (my call).

### Runners

One thin adapter per implementation, all reading the same files; each keeps its own skip list
of cases it does not support yet.

- Rust: `rust/textlane/tests/conformance.rs` (`cargo test`)
- Node: `typescript/-test/conformance.unit.ts` (vitest), through the published loader
- Python: `python/-test/conformance_unit.py` (pytest)
- Optional reference runner on real unified, only to verify expectations at import time

The runners and the schema guard come first; the conformance plan covers phases 1 and 2 in
full, and phases 3–7 stay a roadmap planned one phase at a time (my call).

A repo guard validates every test file against `test-schema.json`.

## Upstream test catalogue

Every repo below is MIT unless noted. "Portable" means expressible as data (pipeline + input
→ expected) without host-language code.

### Core (`tests/core/`)

| source                                       | style                        | ~cases           | portable                          | notes                                                                                |
| -------------------------------------------- | ---------------------------- | ---------------- | --------------------------------- | ------------------------------------------------------------------------------------ |
| unifiedjs/unified                            | inline `node:test`, 11 files | ~195             | ~25% bare, ~65% with test plugins | `use.js` (87: presets, reconfigure, option merge) is the richest; stable error texts |
| vfile/vfile                                  | inline                       | 26 (192 asserts) | ~70%                              | path setters, history, messages; POSIX paths only                                    |
| vfile/vfile-message                          | inline                       | 1 (32 asserts)   | ~65%                              | message fields                                                                       |
| syntax-tree/unist-util-visit, -visit-parents | inline                       | 26 + 47          | ~55–60%                           | needs a scripted-visitor test plugin                                                 |
| syntax-tree/unist-util-is, -position         | inline                       | 27 + 26          | ~70% / ~95%                       |                                                                                      |
| wooorm/trough                                | inline                       | 32               | ~15%                              | JS middleware model; mostly a Rust unit concern                                      |

### Markdown (`tests/md/`)

| source                                                   | style                          | ~cases              | portable | seeds                                                                            |
| -------------------------------------------------------- | ------------------------------ | ------------------- | -------- | -------------------------------------------------------------------------------- |
| commonmark/commonmark-spec 0.31.2                        | `spec.txt` examples            | 652                 | 100%     | parse → html; spec text CC-BY-SA 4.0                                             |
| micromark/micromark `test/io/**`                         | inline, one file per construct | ~1,100 after dedupe | ~95%     | parse → html; options as data (`allowDangerousHtml`, a `constructs` disable map) |
| syntax-tree/mdast-util-from-markdown                     | 27 fixture pairs + ~45 inline  | ~72                 | ~70%     | parse → tree with positions                                                      |
| syntax-tree/mdast-util-to-markdown                       | inline, one 6.7k-line file     | ~470                | ~93%     | tree → markdown, serializer options, 46 round-trips                              |
| syntax-tree/mdast-util-to-string                         | inline                         | 13                  | 100%     |                                                                                  |
| wooorm/markdown-rs                                       | inline `cargo test`            | 2,480               | ~85%     | ~97% duplicates micromark; use for GFM/math/frontmatter and ~80 `to_mdast` trees |
| GFM: github/cmark-gfm spec + micromark-extension-gfm-*   | spec + inline + fixtures       | ~210 html           | ~95%     | `md.gfm`; expectations regenerated through the reference pipeline                |
| GFM: mdast-util-gfm-*, remark-gfm                        | inline + fixture dirs          | ~140                | ~85%     | trees and round-trips                                                            |
| math: mdast-util-math, remark-math                       | inline                         | 31 + 52             | ~85–90%  | micromark-math's HTML embeds KaTeX output: reuse inputs only                     |
| directive: micromark-extension-directive                 | inline                         | 257                 | ~77%     | 196 default-option cases; 58 JS-handler cases become tree cases                  |
| directive: mdast-util-directive, remark-directive        | inline + fixtures              | 78 + 3              | ~95%     |                                                                                  |
| frontmatter: micromark-, mdast-util-, remark-frontmatter | inline + fixtures              | 31 + 28 + 29        | ~100%    | exact option errors, good `expected.error` seeds                                 |
| remark-gemoji                                            | generated + 2 fixtures         | ~1.9k generated     | 100%     | generate from the gemoji dataset, don't store                                    |
| definition lists                                         | 2 plugin tests                 | 2                   | 50%      | mine micromark/mdast-util-definition-list (not yet inventoried)                  |

### HTML (`tests/html/`)

| source                                           | style                                   | ~cases      | portable | seeds                                                                                            |
| ------------------------------------------------ | --------------------------------------- | ----------- | -------- | ------------------------------------------------------------------------------------------------ |
| syntax-tree/mdast-util-to-hast                   | inline, one file per node               | 142         | ~90%     | `md.toHtml`; expectations built with `h()` must be expanded once                                 |
| remarkjs/remark-rehype                           | inline                                  | 10          | ~30%     | ~3 string cases                                                                                  |
| rehypejs/rehype                                  | 46 fixture dirs + 59 parse-error dirs   | 128         | ~95%     | `html.parse`, `html.stringify`, reparse invariant                                                |
| syntax-tree/hast-util-to-html                    | inline, 37 files                        | ~360        | ~97%     | `html.stringify` option matrix                                                                   |
| syntax-tree/hast-util-from-html                  | inline + parse errors                   | 17 + 59     | ~90%     | messages compared on `ruleId` + `place`                                                          |
| syntax-tree/hast-util-raw                        | inline                                  | ~60         | ~90%     | `html.raw`                                                                                       |
| syntax-tree/hast-util-sanitize, rehype-sanitize  | inline + URL-vector loops               | ~85         | ~80%     | ~45 adversarial vectors, one literal case each; GitHub element×attribute grid as one golden case |
| rehype-slug + github-slugger (ISC)               | inline + `fixtures.json`                | 4 + 83      | 100%     | slugger is stateful: one ordered group per document                                              |
| html5lib-tests                                   | tokenizer + serializer JSON             | 6,827 + 230 | 100%     | optional `html.tokenize`; serializer `optionaltags`                                              |
| web-platform-tests `html/syntax/parsing` (BSD-3) | 62 `.dat` files                         | 1,973       | ~97%     | `html.parse` tree construction; read through a `.dat` loader                                     |
| rehype-katex, @shikijs/rehype                    | computed at test time / theme snapshots | 10 + 17     | ~0–30%   | deferred; behaviour-only cases later                                                             |

### MDX (`tests/mdx/`)

Not yet inventoried: `@mdx-js/mdx`'s compile fixtures, `hast-util-to-estree`, `recma-*`.

### Prose (`tests/prose/`)

Not yet inventoried: the retext parsers (`parse-latin`, `parse-english`), `retext-stringify`
and the plugins listed under `textlane-prose`.

### Totals

About 2.3k Markdown cases after dedupe, ~1.1k extension cases plus generated gemoji, ~2.6k
HTML-side cases including WPT, and the core lifecycle cases: the suite starts large.

## Importers

`conformance/sources/<source>.ts` turns one upstream repo into test files and records
provenance. Inputs are extracted from the upstream test files as data (an AST parser such as
acorn for `assert.equal(micromark(…), …)` pairs, spec-text splitting, fixture-dir copying); a
case whose options contain functions is skipped. Expectations come from upstream verbatim
where they already describe textlane's path, and otherwise from running the reference unified
packages once, pinned in the importer's own lockfile (decided): HTML through `remark-parse` →
`remark-rehype` → `rehype-stringify`, hastscript `h()` trees expanded, trees for inputs that
only had HTML expectations. Imported files are committed; importers are rerun deliberately,
never in CI.

## Hardest points

1. **Reference expectations.** micromark-layer HTML is micromark's own compiler output, not
   `toHtml` → `stringify` output; the importers must regenerate ~1k expectations through the
   reference pipeline and record its versions in `source`, or those cases fail on formatting
   alone.
2. **One addon carries every domain.** npm/PyPI subpaths are views over one addon; a tree
   handed between `textlane/md` and `textlane/html` must never cross a serialization
   boundary.
3. **Core holds every tree type but the JS tree.** Bridges stay dependency-free only while
   mdast, hast and nlcst all live in `textlane-core`; the JS tree is oxc's, kept inside `mdx`.
4. **Lifecycle as data.** The core corpus depends on the test-plugin set being identical in
   all three implementations.
5. **Sanitization is security.** `html.sanitize` cases are literal, one vector each, never
   generated in a loop, and run in every target.
6. **Licences.** CommonMark spec text is CC-BY-SA 4.0, WPT BSD-3, github-slugger ISC; each
   imported group keeps its `source.license`.

## Phases

1. **Workspace.** Done: the flat crates `textlane`, `core`, `md`, `html`, `mdx`, `prose`
   (empty domains), their features, release-please `extra-files`, `cargo publish --workspace`;
   docs and skills updated. Bindings gain the `textlane/md`, `textlane/html`, `textlane/mdx`,
   `textlane/prose` subpaths once each has an export.
2. **Suite skeleton.** Done: `conformance/` with `README.md`, `test-schema.json`, the schema
   guard, the three runners wired into `pnpm test`, and a smoke group passing in all three
   languages through stub test plugins (`run_case` behind `testing`).
3. **Core.** Processor and test plugins; import unified, vfile and unist-util cases.
4. **Markdown.** Importers for the CommonMark spec, micromark io, from-markdown and
   to-markdown; then livemark's extensions (GFM, directive, math, frontmatter, gemoji,
   definition lists).
5. **HTML.** to-hast, to-html, rehype fixtures, sanitize, slug, raw; then WPT tree
   construction.
6. **Prose.** Inventory, then import.
7. **MDX.** Inventory `@mdx-js/mdx`; `md.mdx()` syntax, then `mdx.compile()`.

## Left to the builder

- Exact JSON Schema wording, runner internals and how skip lists are stored
- Whether WPT `.dat` files are vendored as-is or converted
- Test-plugin option names beyond those listed
- The order of importers within a phase
