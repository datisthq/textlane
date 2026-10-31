---
name: benchmark-development
description: Use when working in `benchmark/` — textlane's Markdown-to-HTML speed and memory against other libraries in Rust, Node.js and Python. Covers commands, profiles, the adapter contract and its traps.
---

# Benchmark

`benchmark/` measures textlane converting Markdown to HTML from Rust, Node.js and Python against
eleven other libraries (comrak, pulldown-cmark, markdown-rs, unified, micromark, markdown-it,
marked, markdown-it-py, mistune, Python-Markdown, cmark). It ships nothing and is not a release
component: its commits are `chore(benchmark)`. `benchmark/README.md` is the user reference;
this skill is how to change it without making the numbers lie.

## Commands

- `pnpm -C benchmark test:unit` — the units in `benchmark/-test/`; the root `test:unit` chains
  it, and the root `tsgo --noEmit` types it. Nothing there runs a real library, and the section
  has no CI job: its numbers need a fixed, idle machine
- `pnpm -C benchmark setup:tools` — `uv sync --frozen` in `benchmark/adapters/python/`, the
  release addon into `.cache/addon/`, the release wheel installed into that venv, and the
  Rust adapters. Rerun it after a crate change: the wheel is not rebuilt per run
- `pnpm -C benchmark measure` — narrowed with `--case`, `--size`, `--tool`; `--runs 1 --warmup 0`
  for a smoke run. **By default a rerun measures only textlane:**
  the six textlane tools (`textlane-rust`, `textlane-typescript`, `textlane-python`, each
  also with `-hast`). The rivals'
  results carry over, as `show:report` merges the latest per tool, and re-measuring them
  costs half an hour for numbers that did not change. Measure every tool only when asked
  for a full run, or when a rival's version or the machine changed
- `pnpm -C benchmark show:report` — merges `.cache/results/`, latest per case, size and tool
- Publish a full run in `.humans/benchmark.html`: its tables, chart and cards, with the date,
  commit and machine. A stale result file from before a textlane change still merges into
  `show:report`, so pass the run's own file (`show:report <path>`) when the page is regenerated. After a
  textlane-only rerun, build the page from one file merging the rivals' rows of the last
  full run (it may sit in another worktree's `benchmark/.cache/results/`, and JavaScript
  tools that crashed there may have a rerun file of their own) with textlane's new rows,
  titled by their tool names now: a result file keeps the titles it was measured with
- A full run takes about half an hour; unified and micromark dominate it and grow faster than
  their input, so keep the timeout at 180s. The `cmark` README runs every tool at 10MB too, for
  the landing chart: unified and micromark take 100s a document there and add about 20
  minutes. Other large sizes run only the fastest tools, listed under the case's `tools`
  (`gfm`'s 10MB, for memory)

## Layout

`@*.ts` are the entry points; `generate/` the seeded writer, `profiles/` the document shapes,
`cases/` what `measure` runs, `harness/` running and timing, `tools/` one `Tool` per library,
`report/` the HTML, `adapters/<language>/` the scripts and binaries that convert. Everything
generated lives in its `.cache/` (gitignored repo-wide).

## The adapter contract

An adapter takes `<document> <repeat> <flavor>`, imports its library, reads the document and
converts it once untimed, then converts `repeat` times and prints `{"elapsedMs", "outputBytes"}` last. The harness picks
`repeat` so a run converts at least 8MB, and divides. Every library gets the same job, raw HTML
passed through (`allowDangerousHtml`, `unsafe`, `html: true`), in the case's `flavor`:
`commonmark` with no extension, or `gfm` (the `gfm` case), with tables, task lists,
strikethrough, autolink literals and footnotes, through a plugin where the library has none
built in. A GFM piece a library cannot do goes in its `caveats`

Without that warm-up conversion a fresh process's first one, faulting in memory and code,
weighed on the 80 repeats of a 100 KB run unevenly: textlane's Python adapter came out ahead
of its Rust one, which is 5% faster warm.

## Adding a library

Write the adapter beside the others, add a `Tool` in `tools/<language>.ts`, list it in `TOOLS`,
and run `measure --size 10KB --size 100KB --runs 1 --warmup 0 --tool <name>`: its HTML size
must sit near the others', or it renders a different document. Pin it: npm by exact version
(the workspace's seven-day quarantine applies), Python in `pyproject.toml` under
`exclude-newer`, Rust in `adapters/rust/Cargo.lock`. Whatever it does differently from textlane
(no tree, not CommonMark, a GFM piece missing) goes in its `caveats`, which the report prints.

## The two textlane paths

`process` writes HTML straight from mdast when `md.toHtml` and `html.stringify` end the
pipeline, and builds hast as soon as anything sits between them. Each build is measured both
ways: `textlane-<language>-hast` sets `TEXTLANE_HAST`, and its adapter passes `html.stringify`
`quote: '"'`, a non-default option writing the same bytes, so the hast row costs the tree
alone. A real HTML plugin would add its own work, and a JavaScript or Python no-op plugin its
binding's JSON crossing, so neither is the fair stand-in

## Traps

- After merging `main` into a worktree, run `pnpm install` before `measure`: a stale
  `node_modules` makes every JavaScript adapter exit with code 1, which `measure` reports
  as `CRASH` and keeps running, so a full run silently loses four libraries
- The Node.js and Python adapters build and `freeze()` their processor once, outside the
  clock, then `processSync` / Python's `process` per document, as every rival reuses its parser.
  The Node.js adapter imports `typescript/index.ts` from source, the addon found through
  `TEXTLANE_ADDON`; the wheel is built by maturin outside `rust/`, so `setup:tools` pins
  `RUSTUP_TOOLCHAIN=stable` or the default toolchain builds it
- The Rust adapters are their own cargo workspace, so comrak and friends never enter
  `rust/Cargo.lock`; they build with release, LTO and one codegen unit, as `rust/` does
- `conformance` snippets that leave a fence or raw HTML block open, or define a reference, would
  change every later snippet: `readSnippets` drops them, and a thematic break at column 0
  closes any list, quote or indented code between snippets
- Runs go round-robin, one run of every tool per round: a series per tool let another
  session's build land on some tools and not others, and once showed textlane level with
  comrak when it was 1.6× behind. Still compare only on an idle machine (`uptime`)
- Memory is sampled every 50ms, so a run shorter than that shows none; it includes the runtime,
  so Node.js starts tens of megabytes above Rust
