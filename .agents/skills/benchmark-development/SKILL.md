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
  for a smoke run. After a textlane change re-measure only
  `--tool textlane-rust --tool textlane-typescript --tool textlane-python`; rivals carry over
- `pnpm -C benchmark show:report` — merges `.cache/results/`, latest per case, size and tool
- Publish a full run in `.humans/benchmark.html`: its tables, chart and cards, with the date,
  commit and machine. A stale result file from before a textlane change still merges into
  `show:report`, so pass the run's own file (`show:report <path>`) when the page is regenerated
- A full run takes about half an hour; unified and micromark dominate it and grow faster than
  their input, so keep the largest sizes at a few megabytes and the timeout at 180s

## Layout

`@*.ts` are the entry points; `generate/` the seeded writer, `profiles/` the document shapes,
`cases/` what `measure` runs, `harness/` running and timing, `tools/` one `Tool` per library,
`report/` the HTML, `adapters/<language>/` the scripts and binaries that convert. Everything
generated lives in its `.cache/` (gitignored repo-wide).

## The adapter contract

An adapter takes `<document> <repeat>`, imports its library and reads the document untimed,
then converts `repeat` times and prints `{"elapsedMs", "outputBytes"}` last. The harness picks
`repeat` so a run converts at least 8MB, and divides. Every library gets the same job:
CommonMark only, raw HTML passed through (`allowDangerousHtml`, `unsafe`, `html: true`).

## Adding a library

Write the adapter beside the others, add a `Tool` in `tools/<language>.ts`, list it in `TOOLS`,
and run `measure --size 10KB --size 100KB --runs 1 --warmup 0 --tool <name>`: its HTML size
must sit near the others', or it renders a different document. Pin it: npm by exact version
(the workspace's seven-day quarantine applies), Python in `pyproject.toml` under
`exclude-newer`, Rust in `adapters/rust/Cargo.lock`. Whatever it does differently from textlane
(no tree, not CommonMark) goes in its `caveats`, which the report prints.

## Traps

- The Node.js and Python adapters build and `freeze()` their processor once, outside the
  clock, then `processSync` / `process_sync` per document, as every rival reuses its parser.
  The Node.js adapter imports `typescript/index.ts` from source, the addon found through
  `TEXTLANE_ADDON`; the wheel is built by maturin outside `rust/`, so `setup:tools` pins
  `RUSTUP_TOOLCHAIN=stable` or the default toolchain builds it
- The Rust adapters are their own cargo workspace, so comrak and friends never enter
  `rust/Cargo.lock`; they build with release, LTO and one codegen unit, as `rust/` does
- `spec` snippets that leave a fence or raw HTML block open, or define a reference, would
  change every later snippet: `readSnippets` drops them, and a thematic break at column 0
  closes any list, quote or indented code between snippets
- Memory is sampled every 50ms, so a run shorter than that shows none; it includes the runtime,
  so Node.js starts tens of megabytes above Rust
