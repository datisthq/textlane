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
  tests/<area>/<construct>.json  one area per domain (core, md, html, mdx, prose), one file per construct
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
- A test has a `description`, an `input` string or a `tree`, and exactly one `expected`
  outcome. Serialized output is keyed by its file extension: `html`, `md`, `js`, `txt`. A
  `tree` compares as JSON, positions in UTF-16 code units; an `error` matches on its `code`
  and on `message` only as a substring.
- `status: "deviation"` marks a deliberate difference from upstream, explained in `comment`.

`test.*` plugins exist only in builds with the crate's `testing` feature; they let a suite
case exercise the pipeline itself.

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

| source            | reads                                                                       | expectations                                                                                                                                                |
| ----------------- | --------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `commonmark`      | the 26 spec sections                                                        | the spec's 652 examples, each as an HTML case and a tree case, rendered by unified, pinned                                                                  |
| `micromark`       | `test/io` at `6577c20`                                                      | 1,098 cases rendered by unified; micromark's own HTML kept in `comment` where it differs                                                                    |
| `from-markdown`   | `mdast-util-from-markdown` at `afe9516`                                     | 55 fixture and inline inputs as tree cases, positions included                                                                                              |
| `to-markdown`     | `mdast-util-to-markdown` at `a31fe06`                                       | 399 tree → Markdown cases with serializer options and 23 round trips, upstream's expected Markdown verbatim                                                 |
| `gfm`             | the GFM spec and micromark's five GFM extensions                            | 190 cases rendered by unified with remark-gfm; micromark's or GitHub's HTML kept in `comment` where it differs                                              |
| `gfm-mdast`       | mdast-util-gfm-*, mdast-util-gfm, remark-gfm                                | 114 tree, stringify and round-trip cases; upstream's Markdown verbatim, the rest rendered by the reference                                                  |
| `directive`       | micromark-extension-directive, mdast-util-directive, remark-directive       | 325 cases: micromark's inputs as tree cases, upstream's serializer output verbatim, remark-directive's fixtures                                             |
| `math`            | micromark-extension-math, mdast-util-math, remark-math                      | 111 cases: micromark's inputs as tree cases (its HTML embeds KaTeX), mdast-util-math's serializer output, remark-math's KaTeX-free HTML, trees and Markdown |
| `frontmatter`     | micromark-extension-frontmatter, mdast-util-frontmatter, remark-frontmatter | 91 cases: HTML (frontmatter renders to nothing), serializer output, fixtures, and 7 invalid-option errors                                                   |
| `gemoji`          | remark-gemoji with gemoji 8.1.0                                             | 1,915 cases: every name in gemoji's table, as remark-gemoji's own test loops, plus its fixtures                                                             |
| `definition-list` | micromark-extension-definition-list (TypeScript, types stripped)            | 97 cases in the plain, dangerous-HTML and GFM variants upstream runs, rendered by remark-definition-list's hast handlers                                    |
| `to-hast`         | mdast-util-to-hast                                                          | 97 `md.toHtml` cases, mdast in and hast out, the hast rendered by the pinned mdast-util-to-hast rather than by running upstream's `h()` expectations        |

A file holds one construct (`md/heading-atx`), named as micromark names its test files, with a
group per source: the spec section, then each upstream test file. An importer replaces only its
own source's groups, so sources merge into the same files in a stable order.
