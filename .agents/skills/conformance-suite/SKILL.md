---
name: conformance-suite
description: Use when adding, porting or debugging conformance cases in `conformance/` — the test-file format, the `testing` feature and its test plugins, the three runners, skip lists and the schema guard.
---

# Conformance suite

`conformance/` is the contract every implementation runs: JSON test files, each an array of
groups that fix a pipeline, each test an input and one expected outcome. `conformance/README.md`
holds the format; this skill holds the procedure.

## Running it

| what             | command                                                                     | cost                                    |
| ---------------- | --------------------------------------------------------------------------- | --------------------------------------- |
| the schema guard | `vp test .scripts/-test/conformance.unit.ts`                                | instant                                 |
| Rust             | `cargo test -p textlane --features testing --test conformance` from `rust/` | seconds                                 |
| Node             | `vp test ./-test/conformance.unit.ts` from `typescript/`                    | builds the addon with cargo first       |
| Python           | `pnpm -C python test:unit`                                                  | builds a wheel with maturin, needs `uv` |

A path starting with `-test` must be written `./-test/…`: vitest and pytest read it as a flag
otherwise.

## How a case runs

- The crate's `testing` feature adds `run_case` (`rust/textlane/src/actions/case/run.rs`):
  `{pipeline, input | tree}` in, `{text}`, `{tree}` or `{error: {code, message}}` out. The
  napi and PyO3 bindings expose it as `runCase` / `run_case` only in builds with `testing`, so
  it never reaches the published packages or the loader's types.
- Serialized output is keyed by its file extension (`html`, `md`, `js`, `txt`), never by the
  domain that produced it; `tree` and `error` are the other outcomes.
- `test.*` plugins live only there. Phase 2 ships `test.parseText` and `test.compileValue` as a
  stub; the real processor replaces the stub in phase 3 without changing a test file.
- Each runner compares exactly, except `expected.error`: its `code` must match and its
  `message`, if given, is a substring. Change that rule in all three runners at once.

## Corpus first

Cases are ported before the code that passes them. A new source lands with its files listed in
`conformance/pending.json` (an area wildcard such as `md/*` is fine); the runners run pending
cases and swallow their failures. When an implementation makes every case of an entry pass, the
run fails with `<entry> passes now; remove it from conformance/pending.json`: delete the entry,
or narrow a wildcard to the files still failing. A wildcard only trips once its whole area
passes.

## Importing a source

`conformance/` is a private workspace package holding the importers and their pinned upstream
dependencies, one `import:<source>` script per importer: `pnpm -C conformance import:commonmark` runs `sources/commonmark.ts` and formats the
output; commit the result. Inputs are extracted as data; where upstream's expectation is not what
textlane's pipeline produces (micromark's own HTML compiler, `h()` trees), the importer renders
the input once through the pinned unified packages instead. Record the spec or upstream output in
a test's `comment` only where it means something different.

- `actions/source/fetch.ts` checks out a pinned commit into the git-ignored `conformance/.cache/`;
  `actions/group/write.ts` replaces one source's groups in a construct file and keeps the rest;
  `actions/markdown/render.ts` is the reference pipeline (`renderMarkdown`, `parseMarkdown`) and
  its textlane equivalent; `actions/node/evaluate.ts` and `actions/test/find.ts` read upstream
  test files statically with acorn.
- An importer skips, and counts, what a pipeline case cannot express: options with no textlane
  equivalent yet (micromark's construct switches, `defaultLineEnding`), input that is not
  statically evaluable, lone UTF-16 surrogates, which UTF-8 input cannot hold, and stress tests
  over 2,000 characters (`actions/input/check.ts`), which belong in benchmarks.
- Upstream's own expectations are kept verbatim where they describe textlane's path; they follow
  upstream's latest release, while the pinned reference lags behind the 7-day quarantine. After
  bumping a pin, rerun the importers and compare: a reference that still disagrees with a
  verbatim expectation is a real difference to look at.
- `actions/node/scope.ts` resolves an identifier in its test's own function before the module, so
  one test's `const value` never leaks into another's case.

## Adding cases

1. Put the group in `conformance/tests/<area>/<construct>.json`; `<area>` is `core`, `md`,
   `html`, `mdx` or `prose`, and a file holds one construct with a group per source (micromark's
   test-file names decide construct names in `md`). Group descriptions are unique within a file.
2. Anything ported from upstream carries `source` (repository, commit, path, test, licence).
3. Run the schema guard, then the Rust runner; the Node and Python runners read the same files.
4. A case the crate cannot pass yet goes in `conformance/pending.json`. A case one runner
   alone cannot pass goes in that runner's `KNOWN_DIVERGENCES` as
   `"<area>/<file>#<group>": "<issue URL>"`; an entry without an open issue is a bug.

A failing test names `<area>/<file>#<group> / <test>` with the expected and actual outcomes;
fix the implementation, never the expectation, unless upstream disagrees with it.
