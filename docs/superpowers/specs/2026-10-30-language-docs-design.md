# Language docs on textlane.dev

Design for filling the Python, TypeScript and Rust sections of the docs site, which today hold
only an Introduction (the package README), an Overview, Agent skills and Contributing.

## Intent

Someone who installs textlane in one language should find, in that language's section, how
to build a pipeline, every plugin with its options, and the trees a pipeline produces — with
code they can paste. Today they must read the generated bindings or the crate source.

The three packages are twins: the same crate behind three bindings. Their sections are twins
too — the same pages in the same order, each written in its language.

## Goals

- One page set, mirrored across `python/docs/`, `typescript/docs/` and `rust/docs/`
- A plugin reference that cannot drift from the crate: option names, types, defaults and
  descriptions come from the crate's `schema` feature
- Every plugin names the upstream unified library it follows (`md.gfm` → remark-gfm), so a
  unified user can map their pipeline across
- Every code snippet is real and runs against the current version

## Non-goals

- API docs generated from rustdoc or typedoc; docs.rs covers the Rust item-level reference
- Documenting internals (the arena, the tokenizers) — those stay in `.humans/`
- JavaScript or Python plugins joining a pipeline: not implemented yet, so documented only as
  "not yet" in Status

## Anti-goals

- Three sections that say different things about the same behaviour
- Hand-copied option tables that silently go stale
- Prose that claims behaviour the conformance suite doesn't back

## Pages

Each language gets these pages (`path` under its prefix, `order` as listed). Existing pages
keep their paths.

| order | page                                             | content                                                                                                                                          |
| ----- | ------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1     | Introduction                                     | the README, unchanged role                                                                                                                       |
| 2     | Overview                                         | existing; Memory moves to its own page                                                                                                           |
| 3     | Getting started                                  | install, first pipeline (Markdown → HTML), reading the result, GFM, frontmatter                                                                  |
| 4     | Processor                                        | `use` (plugin, options, presets/lists), `data`, `freeze`, `parse` / `run` / `stringify` / `process` and their sync forms, the file it returns    |
| 5     | Recipes                                          | common pipelines: Markdown → HTML, HTML → Markdown, sanitizing, highlighting, slugs, MDX compile, prose linting with `nl.*` and reading messages |
| 6–9   | Markdown / HTML / MDX / Natural language plugins | one page per domain: each plugin, what it does, its upstream twin, an example, its options table                                                 |
| 10    | Syntax trees                                     | mdast, hast and nlcst as textlane produces them, how a binding exposes them (dicts in Python, objects in Node, arena `Tree` in Rust), positions  |
| 11    | Memory                                           | moved from Overview                                                                                                                              |
| 12    | Errors                                           | error codes (`invalid-options`, unknown plugin, …), how each language surfaces them                                                              |
| 13    | Agent skills                                     | existing                                                                                                                                         |
| -1    | Contributing                                     | existing                                                                                                                                         |

The Rust section's Processor page covers both `Processor::new().with()` (typed) and
`Pipeline` (by name, JSON options); the Python and TypeScript pages cover their processor
only.

## Plugin reference — how it is generated

- `pnpm generate:bindings` (`.scripts/generate-bindings.ts`) already reads every plugin's
  schema; it also writes the options section of each plugin page. Decided: extend this script
  rather than add a second one, so one command keeps bindings and docs in step and its
  `--check` (run by `.scripts/-test/bindings.unit.ts`) fails on docs drift too
- Pages are hand-written; the script only replaces the text between markers such as
  `<!-- generated:options md.gfm -->` … `<!-- /generated -->`, rendering the options table
  in that page's language: the option's name, its type in that language, default,
  description. Python and TypeScript take the upstream camelCase keys (`singleTilde`), as the
  bindings already do. Rust shows the struct field (`single_tilde`), derived from the key the
  way the crate's `rename_all = "camelCase"` does, honouring explicit `serde(rename)`s
  (`unsafe`); Rust types are shown as their JSON shape (my call: no new schema source)
- Plugins with no options get a generated "no options" line, and `--check` also fails when a
  plugin in the crate has no section on its domain page, so the reference stays complete
- The upstream twin comes from the schema description. Only about 9 of 31 plugins name it
  today, so each plugin's doc comment in the crate gains it (my call: a `docs:` commit, since
  it changes no behaviour); the generator renders it, never three hand copies
- Hand-written per plugin: one sentence on what it does, one example in that language; Rust
  examples use the typed `Processor::new().with(...)`

## Constraints

- Livemark frontmatter on every page (`title`, `description`, `path`, `icon`, `order`);
  `livemark.config.ts` already includes `*/docs/**/*.md`, so new pages need no config change
- `.scripts/-test/docs.unit.ts`: snippets ≤ 100 columns, named paths and `pnpm` scripts must
  exist
- Commits are `docs:` (docs site) and `chore:` (generator); none `feat`/`fix`
- The shipped `.skills/textlane-usage` skill should point at the new pages where it now
  explains the same thing
- Any behaviour the docs reveal as wrong or missing is reported, not fixed in the docs work

## Version and current claims

- The docs describe `main` (pre-release, `0.0.0`, nothing on a registry yet); snippets are
  verified against a local build. Install commands stay as written: they are what users will
  run once the first release lands, and Overview's Status already says pre-release (my call)
- The Overview pages and READMEs say existing JavaScript plugins "can still take part". That
  is the goal, not current behaviour. Overview's lead is reworded to say so; the README
  pitch stays, since it states intent and Status covers the gap (my call)

## Left to the builder

- Exact page titles, icons and descriptions
- Table layout of the generated options
- Order of work across languages (suggested: write one language fully, then port)
- Whether snippets are executed by a test; at minimum each is run once by hand while writing
