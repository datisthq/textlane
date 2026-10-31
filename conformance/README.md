# Conformance

The language-neutral conformance suite every textlane implementation runs: the same pipeline,
the same input, the same expected result. The Rust crate and the Node and Python bindings all
run it, so a difference between them is a failing test rather than a surprise for a user.

The format follows [JSON-Schema-Test-Suite](https://github.com/json-schema-org/JSON-Schema-Test-Suite),
and the layout fairspec's suite.

## Layout

```
conformance/
  test-schema.json             JSON Schema of a test file
  tests/<area>/<construct>.json  one area per domain (core, md, html, mdx, nl), one file per construct
  pending.json                 what the crate does not pass yet, shared by every runner
  sources/<source>.ts          importers that turn an upstream suite into test files
  .cache/                      pinned upstream checkouts the importers read; git-ignored
```

## Test files

A file is an array of groups. A group fixes the pipeline; its tests vary only the input:

```json
[
  {
    "description": "a parser and a compiler round-trip text",
    "pipeline": [["test.parseText"], ["test.compileValue"]],
    "tests": [{ "description": "a word", "input": "alpha", "expected": { "txt": "alpha" } }]
  }
]
```

- A group has a `description`, unique within its file, a `pipeline` of `[name, options?]`
  entries naming public plugins, and `tests`. It may carry a `comment`, `specification` links,
  and a `source` (repository, a commit or release `version`, path, test name, licence),
  required for anything ported from upstream. Ported groups follow upstream's grouping: one
  per spec section or test block.
- A test has a `description`, an `input` string or a `tree` (and a `path` when a plugin
  resolves URLs against where the document lives), and exactly one `expected`
  outcome. Serialized output is keyed by its file extension: `html`, `md`, `js`, `txt`. A
  `tree` compares as JSON, positions in UTF-16 code units (dropped on both sides when the group
  says `positions: false`); `messages` compare on `ruleId`, `source` and `place`, in place order; an `error`
  matches on its `code` and on `message` only as a substring.
- `status: "deviation"` marks a deliberate difference from upstream, explained in `comment`.
- Core lifecycle cases carry `operations` instead of an input: steps run in order on a processor
  holding the group's pipeline (`["use", plugin, options?]`, `["freeze"]`, `["data", key?, value?]`,
  `["parse", text]`, `["run", tree, {api}]`, `["stringify", tree]`, `["process", text, {api}]`,
  `api` being `sync` or `async`). `expected` names the outcome of the last step: `data` (the
  processor's data), `fileData` (what plugins stored in the file's `data`), `trace`,
  `messages`, `tree`, `txt` or `error`. A run may report several; the
  named one is compared, and an unexpected error always fails.

`test.*` plugins exist only in builds with the crate's `testing` feature; they let a suite case
exercise the pipeline itself: `test.parseText` and `test.compileValue` (text to a text node and
back), `test.recordOptions` (records its options in `data.recorded`), `test.setData`,
`test.replaceTree`, `test.appendNode`, `test.fail`, `test.message`, `test.noop`, and
`test.trace:<label>` (appends its label to the trace; one plugin per label, since a plugin used
twice is one plugin reconfigured). Transformers take `mode: "sync" | "async"`.

`host:<behaviour>[:<label>]` entries stand for a binding's own plugin (`core/host.json`): each
runner implements them in its language, the Rust one through `start_process` and `resume`, the
Node and Python ones as JavaScript and Python functions through the public API, processing
asynchronously. `appendNode` (`node`, `mode`), `replaceTree` (`tree`), `message` (`reason`,
`ruleId`, `source`, `place`), `countMessages` and `readValue` (a paragraph of the message count
or the document), `fail` and `returnError` (an error coded `plugin-failed`) and
`recordOptions` (its attacher appends its options to the data's `recorded`).

## Runners

| implementation | runner                                 | run with                       |
| -------------- | -------------------------------------- | ------------------------------ |
| Rust           | `rust/textlane/tests/conformance.rs`   | `pnpm -C rust test:unit`       |
| Node           | `typescript/-test/conformance.unit.ts` | `pnpm -C typescript test:unit` |
| Python         | `python/-test/conformance_unit.py`     | `pnpm -C python test:unit`     |

## Pending

The suite is filled before the implementation: `pending.json` maps a file (`md/heading-atx`),
a group (`md/tab#Tabs`) or a whole area (`md/*`) to why it does not pass yet. Every runner still runs
pending cases and tolerates their failures; an entry whose cases all pass fails the run until it
is removed, so the list only shrinks as the crate grows. A runner's own `KNOWN_DIVERGENCES`, an
entry mapped to an open issue, is for a difference in that runner alone.

`.scripts/-test/conformance.unit.ts` validates every test file against `test-schema.json`, and
every `pending.json` entry against the files and groups that exist.

## Importers

`pnpm -C conformance import:<source>` rewrites that source's files from a pinned upstream
release; importers are run deliberately and their output committed, never run in CI.

| source             | reads                                                                                                                                  | expectations                                                                                                                                                                                                                         |
| ------------------ | -------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `commonmark`       | the 26 spec sections                                                                                                                   | the spec's 652 examples, each as an HTML case, a tree case and a parse → stringify round trip, rendered by unified, pinned                                                                                                           |
| `micromark`        | `test/io` at `6577c20`                                                                                                                 | 1,098 cases rendered by unified; micromark's own HTML kept in `comment` where it differs                                                                                                                                             |
| `from-markdown`    | `mdast-util-from-markdown` at `afe9516`                                                                                                | 55 fixture and inline inputs as tree cases, positions included                                                                                                                                                                       |
| `to-markdown`      | `mdast-util-to-markdown` at `a31fe06`                                                                                                  | 399 tree → Markdown cases with serializer options and 23 round trips, upstream's expected Markdown verbatim; 17 tree round trips stringified by the reference                                                                        |
| `gfm`              | the GFM spec and micromark's five GFM extensions                                                                                       | 190 cases rendered by unified with remark-gfm; micromark's or GitHub's HTML kept in `comment` where it differs                                                                                                                       |
| `gfm-mdast`        | mdast-util-gfm-*, mdast-util-gfm, remark-gfm                                                                                           | 114 tree, stringify and round-trip cases; upstream's Markdown verbatim, the rest rendered by the reference                                                                                                                           |
| `cjk`              | markdown-cjk-friendly's shared test cases                                                                                              | 420 `md.cjkFriendly` cases: each paragraph of its test files and CommonMark's emphasis examples as HTML and Markdown, rendered by remark-cjk-friendly; the Markdown ones pending                                                     |
| `alerts`           | remark-github-blockquote-alert's tests                                                                                                 | 24 `md.gfm({ alerts })` cases rendered with remark-gfm and the pinned plugin, upstream's HTML agreeing                                                                                                                               |
| `directive`        | micromark-extension-directive, mdast-util-directive, remark-directive                                                                  | 325 cases: micromark's inputs as tree cases, upstream's serializer output verbatim, remark-directive's fixtures                                                                                                                      |
| `math`             | micromark-extension-math, mdast-util-math, remark-math                                                                                 | 111 cases: micromark's inputs as tree cases (its HTML embeds KaTeX), mdast-util-math's serializer output, remark-math's KaTeX-free HTML, trees and Markdown                                                                          |
| `frontmatter`      | micromark-extension-frontmatter, mdast-util-frontmatter, remark-frontmatter                                                            | 91 cases: HTML (frontmatter renders to nothing), serializer output, fixtures, and 7 invalid-option errors                                                                                                                            |
| `frontmatter-data` | vfile-matter, yaml                                                                                                                     | 44 `md.frontmatter` cases with `data` and `strip`, each input run through the pinned vfile-matter: `fileData.matter`, `invalid-frontmatter` errors and stripped output                                                               |
| `gemoji`           | remark-gemoji with gemoji 8.1.0                                                                                                        | 1,915 cases: every name in gemoji's table, as remark-gemoji's own test loops, plus its fixtures                                                                                                                                      |
| `emoji`            | remark-emoji's test tables, node-emoji 2.2.0, emoticon 4.1.0                                                                           | 63 `md.gemoji` cases with remark-emoji's options, as Markdown and HTML; cases whose shortcodes node-emoji and gemoji name differently left out                                                                                       |
| `definition-list`  | micromark-extension-definition-list (TypeScript, types stripped)                                                                       | 97 cases in the plain, dangerous-HTML and GFM variants upstream runs, rendered by remark-definition-list's hast handlers                                                                                                             |
| `to-hast`          | mdast-util-to-hast                                                                                                                     | 97 `md.toHtml` cases, mdast in and hast out, the hast rendered by the pinned mdast-util-to-hast rather than by running upstream's `h()` expectations                                                                                 |
| `breaks`           | remark-breaks, mdast-util-newline-to-break                                                                                             | 22 `md.toHtml({ breaks })` cases: upstream's fixtures as HTML, verbatim, and as hast trees                                                                                                                                           |
| `smartypants`      | remark-smartypants' tests                                                                                                              | 50 `md.smartypants` cases as Markdown and HTML, with inputs of textlane's own for the UTF-16 splice                                                                                                                                  |
| `to-html`          | hast-util-to-html                                                                                                                      | 279 `html.stringify` cases, hast in and HTML out, upstream's HTML verbatim (all agreeing with the pinned reference); input trees built by calling the pinned hastscript                                                              |
| `rehype`           | rehype's fixtures and parse errors, hast-util-from-html                                                                                | 153 `html.parse` tree and output cases and 59 parse-error cases (`expected.messages`)                                                                                                                                                |
| `site-plugins`     | remark-heading-id, strip-markdown, rehype-unwrap-images, @stefanprobst/rehype-extract-toc                                              | `md.headingId`, `md.strip`, `html.unwrapImages` and `html.extractToc` cases on inputs written for them, each rendered through its pinned package; the table of contents compared as `fileData`                                       |
| `html-transforms`  | hast-util-raw, hast-util-sanitize, rehype-sanitize, rehype-slug, github-slugger, hast-util-minify-whitespace, rehype-autolink-headings | 285 `html.raw`, `html.sanitize` (each URL vector a literal case), `html.slug` (with `autolink`) and `html.format({ minify })` cases                                                                                                  |
| `html5lib`         | html5lib-tests serializer                                                                                                              | 205 `html.stringify` cases, each token stream rebuilt as a hast tree; html5lib's own HTML kept in `comment` where it differs beyond the stream's edges                                                                               |
| `wpt`              | web-platform-tests tree construction, sparse checkout                                                                                  | 1,738 `html.parse` cases, trees without positions (`positions: false`); fragment and scripting-off cases skipped                                                                                                                     |
| `unified`          | unified's test files, as operations                                                                                                    | 82 core lifecycle cases authored as data, each citing the unified test it ports; expectations from running them through the pinned unified with JavaScript test plugins                                                              |
| `html-convert`     | hast-util-to-mdast, rehype-format                                                                                                      | 241 `html.toMarkdown` Markdown and tree cases (stringified with GFM and fenced code, as upstream's harness does) and 16 `html.format` cases                                                                                          |
| `nl-parse`         | parse-latin, parse-english, mdast-util-to-nlcst, hast-util-to-nlcst                                                                    | 360 `nl.parse` cases (each input is its fixture's text, which nlcst keeps whole) and 25 `md.toNaturalLanguage` / `html.toNaturalLanguage` cases                                                                                      |
| `nl-plugins`       | nine retext plugins                                                                                                                    | 283 `nl.*` cases: smartypants output and the others' messages, sorted by place; retext-spell deferred (it needs a hunspell dictionary)                                                                                               |
| `mdx`              | micromark-extension-mdx-_, mdast-util-mdx-_, @mdx-js/mdx                                                                               | 275 `md.mdx` cases (every micromark input as a tree or a `source:ruleId` error through remark-mdx, ESTree dropped; serializer output) and 101 `mdx.compile` cases whose `expected.js` is the pinned compiler's exact output          |
| `estree`           | hast-util-to-estree's reading of `data.estree`                                                                                         | 6 `mdx.compile` cases whose MDX nodes carry only `data.estree`, compiled by the pinned @mdx-js/mdx                                                                                                                                   |
| `highlight`        | textmate-grammars-themes samples (giallo's pinned commit), @shikijs/rehype 4.5.0                                                       | 285 `html.highlight` cases: every bundled grammar's sample (first 30 lines) under livemark's default themes, a theme matrix, and language fallbacks; expected HTML is Shiki's, so giallo must match it with Shiki's grammar versions |
| `katex`            | KaTeX 0.18.5's screenshot formulas, rehype-katex 7.0.1                                                                                 | 141 `html.katex` cases: KaTeX's screenshot formulas, inline and display, and rehype-katex's element and message behaviour; expected HTML is rehype-katex's over KaTeX 0.18.5 with attributes sorted as `html.katex` sorts katex-rs's |
| `chains`           | remark-mdx, retext, rehype-remark, remark-rehype                                                                                       | 24 cases read from the `.parse` / `.process` / `.stringify` chains in their tests, replayed through the pinned plugins                                                                                                               |

A file holds one construct (`md/heading-atx`), named as micromark names its test files, with a
group per source: the spec section, then each upstream test file. An importer replaces only its
own source's groups, so sources merge into the same files in a stable order.
