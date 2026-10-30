# Benchmark

How fast, and in how much memory, textlane turns Markdown into HTML: its Rust crate, Node.js
addon and Python extension side by side with other Markdown libraries, all converting the same
generated document, as CommonMark or, in the `gfm` case, as GFM with tables, task lists,
strikethrough, autolink literals and footnotes switched on in each. Under GFM:

| tool                      | how it runs                                                                            |
| ------------------------- | -------------------------------------------------------------------------------------- |
| textlane (Rust, mdast)    | `md.parse` → `md.gfm` → `md.toHtml` → `html.stringify` through `Pipeline`, release+LTO |
| textlane (Node.js, mdast) | the same pipeline through the addon's processor, frozen once, `processSync`            |
| textlane (Python, mdast)  | the same pipeline through the wheel's processor, frozen once, `process`                |
| … (…, mdast + hast)       | each of the three with hast kept, as any `html.*` plugin before `stringify` keeps it   |
| comrak                    | `markdown_to_html`, `unsafe` on, the five GFM extensions                               |
| pulldown-cmark            | `html::push_html` over `Parser::new_ext` with its GFM options (no autolink literals)   |
| markdown-rs               | `to_html_with_options` with `Options::gfm()`, dangerous HTML allowed                   |
| unified (remark + rehype) | `remark-parse` → `remark-gfm` → `remark-rehype` → `rehype-stringify`                   |
| micromark                 | `micromark` with `micromark-extension-gfm`, dangerous HTML allowed                     |
| markdown-it               | the default preset with `linkify`, plus the footnote and task-list plugins             |
| marked                    | `Marked` with GFM on, plus `marked-footnote`                                           |
| markdown-it-py            | the `gfm-like` preset, plus mdit-py-plugins' footnote and task-list plugins            |
| mistune                   | `create_markdown(escape=False)` with its five GFM plugins                              |
| Python-Markdown           | `Markdown().convert` with tables, footnotes and pymdown-extensions' GFM pieces         |
| cmark (C, via cmarkgfm)   | `github_flavored_markdown_to_html` with `CMARK_OPT_UNSAFE` and footnotes               |

The adapters live in `adapters/`: `javascript/` beside this package's pinned npm dependencies,
`python/` a uv project of its own, `rust/` a cargo crate with its own workspace and lock, so no
rival reaches the published packages. Linux only: memory is the peak
anonymous memory of the process, sampled from `/proc`, and the cap comes from `systemd-run`.

Under CommonMark each runs with no extension: textlane and unified without `md.gfm` and
`remark-gfm`, markdown-it and markdown-it-py with the `commonmark` preset, marked with GFM off,
cmark through `markdown_to_html`, the rest with their default options.

Time leaves out start-up: every adapter imports its library and reads the document, then starts
its clock, converts the document `repeat` times (at least 8MB of input per run) and prints
`{"elapsedMs", "outputBytes"}`. The report shows the time per document.

## Usage

```bash
pnpm -C benchmark setup:tools                    # uv sync, the addon and wheel, the Rust adapters
pnpm -C benchmark measure --case gfm --size 1MB
pnpm -C benchmark show:report                    # .cache/report.html
```

`measure` runs every size of every case unless narrowed with `--case`, `--size` or `--tool`
(each repeatable). `--runs` (5) and `--warmup` (1) set the rounds, each running every tool once in turn so
load that comes and goes lands on all of them alike, `--timeout` its
limit in seconds (180), and `--memory-max` caps each tool, for example `4G`. The Rust adapters
and the Node.js addon are rebuilt before each run (a no-op when fresh); the Python wheel is not,
so rerun `setup:tools` after a crate change before measuring textlane (Python). Results land in
`.cache/results/`, one file per invocation; `show:report` keeps the latest result per case,
size and tool, and `--out` writes the page elsewhere.

`generate:data` writes a document without measuring anything:

```bash
pnpm -C benchmark generate:data --profile conformance --size 4MB
```

## Data

Every document is generated from a seed, so each tool reads identical bytes, and cached in
`.cache/data/` until its profile, version, size or seed changes. A size is a byte target such
as `10KB`. A profile in `profiles/` writes one block of Markdown at a time:

- `readme` — a README in CommonMark: headings, paragraphs with emphasis, code spans and links,
  lists, block quotes, fenced code
- `readme-gfm` — the same README in GFM, adding strikethrough, autolink literals, footnotes,
  task lists and tables
- `conformance` — the plain CommonMark inputs of `conformance/tests/md/`, back to back and split by
  thematic breaks; snippets that would swallow the rest (an unclosed fence or raw HTML block, a
  reference definition) are left out

A case in `cases/` picks a profile, its sizes and its `flavor` (`commonmark` unless it says
`gfm`), which the harness hands each adapter as its third argument; `tools` can narrow a size to
a few tools:

- `cmark` — the `readme` profile as CommonMark
- `gfm` — the `readme-gfm` profile as GFM
- `cmark-full` — the `conformance` profile as CommonMark: every edge case of the spec

`cmark` and `gfm` run 10MB only for the six fastest, to show their memory on a large
document.

## Correctness

There is no gate yet. Every library renders Markdown slightly differently (textlane matches
unified byte for byte), so the report shows each one's HTML size: one far from the others
renders the document differently, and its time is not like for like.
