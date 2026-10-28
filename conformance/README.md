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
  tests/<area>/<feature>.json  one area per domain: core, md, html, mdx, prose
```

## Test files

A file is an array of groups. A group fixes the pipeline; its tests vary only the input:

```json
[
  {
    "description": "a parser and a compiler round-trip text",
    "pipeline": [["test.parseText"], ["test.compileValue"]],
    "tests": [{ "description": "a word", "input": "alpha", "expected": { "text": "alpha" } }]
  }
]
```

- A group has a `description`, unique within its file, a `pipeline` of `[name, options?]`
  entries naming public plugins, and `tests`. It may carry a `comment`, `specification` links,
  and a `source` (repository, commit, path, test name, licence), required for anything ported
  from upstream.
- A test has a `description`, an `input` string or a `tree`, and exactly one `expected`
  outcome: `text`, `tree` (positions in UTF-16 code units) or `error`, matched on its `code`
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

Each runner skips only what its `KNOWN_DIVERGENCES` lists: `<area>/<file>#<group>` mapped to
the open issue whose fix unblocks it. `.scripts/-test/conformance.unit.ts` validates every test
file against `test-schema.json`.
