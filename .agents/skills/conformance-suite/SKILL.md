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
- `test.*` plugins live only there. Phase 2 ships `test.parseText` and `test.compileValue` as a
  stub; the real processor replaces the stub in phase 3 without changing a test file.
- Each runner compares exactly, except `expected.error`: its `code` must match and its
  `message`, if given, is a substring. Change that rule in all three runners at once.

## Adding cases

1. Put the group in `conformance/tests/<area>/<feature>.json`; `<area>` is `core`, `md`,
   `html`, `mdx` or `prose`. Group descriptions are unique within a file.
2. Anything ported from upstream carries `source` (repository, commit, path, test, licence).
3. Run the schema guard, then the Rust runner; the Node and Python runners read the same files.
4. A case an implementation cannot pass yet goes in that runner's `KNOWN_DIVERGENCES` as
   `"<area>/<file>#<group>": "<issue URL>"`. Nothing else skips, and an entry without an open
   issue is a bug.

A failing test names `<area>/<file>#<group> / <test>` with the expected and actual outcomes;
fix the implementation, never the expectation, unless upstream disagrees with it.
