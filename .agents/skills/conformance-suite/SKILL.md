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
- Every case runs through `Pipeline` (`models/pipeline.rs`), the name-based processor the
  bindings drive: `find_plugin` in `actions/plugin/find.rs` maps each name (`md.parse`, `md.definitionList`, `md.directive`, `md.frontmatter`, `md.gemoji`, `md.gfm`, `md.math`, `md.stringify`, `md.toHtml`, `html.parse`, `html.sanitize`, `html.slug`, `html.stringify`, `html.toMarkdown`) and reads its options into the
  plugin's options struct, failing with `invalid-options`; a new plugin joins there. A tree
  input is read as the kind the first stage takes. The `test.*` plugins live in
  `utils/testing/plugins.rs`, compiled only under `testing`; `mode: "async"` makes a stage
  hand back a job, as a JavaScript plugin returning a promise would.
- A case with `operations` calls the pipeline once per entry (`use`, `data`, `freeze`,
  `parse`, `run`, `stringify`, `process`); the last call's output is the case's, beside
  `data`, `messages` and the labels `test.trace:<label>` recorded (`trace`).
- Every result also carries the file's `messages`, in the order they were reported, as
  `{ruleId, source, place}`; a test names the one outcome it compares.
- Each runner compares exactly, except `expected.error`: its `code` must match and its
  `message`, if given, is a substring. Change that rule in all three runners at once.

## Corpus first

Cases are ported before the code that passes them. A new source lands with its files listed in
`conformance/pending.json` (an area wildcard such as `md/*` is fine); the runners run pending
cases and swallow their failures. When an implementation makes every case of an entry pass, the
run fails with `<entry> passes now; remove it from conformance/pending.json`: delete the entry,
or narrow a wildcard to the files still failing. A wildcard only trips once its whole area
passes, so it protects nothing that already passes: once some groups of an area pass, list the
files that fail whole and the failing `<file>#<group>` of the rest. To see what passes, run the
Rust runner with `pending.json` emptied to `{}` and restore it after.

## Importing a source

`conformance/` is a private workspace package holding the importers and their pinned upstream
dependencies, one `import:<source>` script per importer: `pnpm -C conformance import:commonmark` runs `sources/commonmark.ts` and formats the
output; commit the result. Inputs are extracted as data; where upstream's expectation is not what
textlane's pipeline produces (micromark's own HTML compiler, `h()` trees), the importer renders
the input once through the pinned unified packages instead. Record the spec or upstream output in
a test's `comment` only where it means something different.

- `actions/source/fetch.ts` checks out a pinned commit (sparsely, given `paths`, for a repository
  as large as WPT) into the git-ignored `conformance/.cache/`;
  `actions/group/write.ts` replaces one source's groups in a construct file and keeps the rest;
  `actions/markdown/render.ts` is the reference pipeline (`renderMarkdown`, `parseMarkdown`) and
  its textlane equivalent; `actions/mdast/extract.ts` reads an mdast-util's
  `fromMarkdown` / `toMarkdown` tests for any syntax plugin; `actions/node/evaluate.ts` and
  `actions/test/find.ts` read upstream
  test files statically with acorn.
- Trees a test builds with `h()` / `s()` are evaluated by calling the pinned hastscript on
  statically evaluated arguments (`evaluateNode`'s `builders`); upstream test code never runs.
- A TypeScript test file is read through Node's `stripTypeScriptTypes` before acorn parses it.
- An importer skips, and counts, what a pipeline case cannot express: options with no textlane
  equivalent yet (micromark's construct switches, `defaultLineEnding`), input that is not
  statically evaluable, lone UTF-16 surrogates, which UTF-8 input cannot hold, and stress tests
  over 2,000 characters (`actions/input/check.ts`), which belong in benchmarks.
- Upstream's own expectations are kept verbatim where they describe textlane's path; they follow
  upstream's latest release, while the pinned reference lags behind the 7-day quarantine. After
  bumping a pin, rerun the importers and compare: a reference that still disagrees with a
  verbatim expectation is a real difference to look at.
- `actions/node/scope.ts` resolves an identifier lexically, through each enclosing block from the
  module inward, so one loop's or test's `const value` never leaks into another's case.

`sources/differential.ts` holds textlane's own md inputs where `md.parse` once differed from
the reference, found by fuzzing, and its directive cases for `md.toHtml`, rendered by the
pinned mdast-util-to-hast; add an input there and rerun `import:differential`, never
write its tree by hand.

## Core cases

unified's tests call inline JavaScript plugins, so `sources/unified.ts` holds the core cases as
data, each citing the unified test it ports, and computes every expectation by running its
`operations` through the pinned unified with JavaScript versions of the test plugins. Never write
an expected outcome by hand: add the case, rerun `pnpm -C conformance import:unified`, read what
unified did. The crate's test plugins must behave as those JavaScript ones do, and an error
`Pipeline` raises carries unified's own message (`` Cannot `processSync` without `parser` ``).

## Errors

A syntax error carries the VFileMessage's `source:ruleId` as its code (`micromark-extension-mdx-jsx:unexpected-eof`).
An invalid plugin option is `expected.error` with the code `invalid-options`, shared by every
plugin, and upstream's message (from its `assert.throws` pattern, `^Error: ` and escapes dropped)
as the substring a runner matches. Error codes are public API: add a new one only for a new kind
of failure, never per message.

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
