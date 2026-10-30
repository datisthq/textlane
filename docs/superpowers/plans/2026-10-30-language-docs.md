# Language Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fill the Python, TypeScript and Rust sections of textlane.dev with the same pages: guides, a generated plugin reference and the syntax trees.

**Architecture:** The crate's plugin registry gains each plugin's upstream library. `pnpm generate:bindings` renders every plugin's options section into hand-written domain pages between markers, in each language, and `--check` fails on drift or a missing plugin. Then the guides are written in TypeScript first and ported to Python and Rust.

**Tech Stack:** Rust (schemars, `schema` feature), Node 24 TypeScript scripts, vite-plus tests, livemark Markdown.

**Spec:** `docs/superpowers/specs/2026-10-30-language-docs-design.md`

## Global Constraints

- Pages live in `typescript/docs/`, `python/docs/`, `rust/docs/`, with livemark frontmatter `title`, `description`, `path` (under `/typescript/`, `/python/`, `/rust/`), `icon`, `order`
- Page order per section: 1 Introduction, 2 Overview, 3 Getting started, 4 Processor, 5 Recipes, 6 Markdown plugins, 7 HTML plugins, 8 MDX plugins, 9 Natural language plugins, 10 Syntax trees, 11 Memory, 12 Errors, 13 Agent skills, -1 Contributing. Existing paths stay unchanged
- Code snippets ≤ 100 columns; every path and `pnpm` script named must exist (`.scripts/-test/docs.unit.ts`)
- Python and TypeScript option keys are camelCase (`singleTilde`). Rust shows the struct field (`single_tilde`)
- Rust examples use `Processor::new().with(...)`
- Every snippet is run once against a local build (`pnpm build`) before its commit
- Commit types: crate registry change `refactor:`, generator and tests `chore:`, pages `docs:`. Never `feat`/`fix`, never `!`
- Commit with a pathspec: `git add <new>` then `git commit -m "…" -- <paths>`
- Docs show behaviour only. A behaviour that turns out wrong or missing is reported, not fixed

## Review Focus

1. Options that are unions or refs (`md.frontmatter`: `"yaml" | "toml" | Matter | […]`, with the nested `Matter`/`Fence` defs) must render as readable types in all three languages, not `[object Object]` or an empty row. Pinned by a test in Task 2
2. A field renamed with `serde(rename)` (`unsafe` in `rust/md/src/models/options.rs:93`) must show its real Rust field name in the Rust table. Pinned in Tasks 1–2
3. Unit plugins (`md.gemoji`, `md.definitionList`, `nl.repeatedWords`, `nl.stringify`, `nl.indefiniteArticle`) render "no options" and are still required to have a section. Pinned in Task 2
4. A plugin added to the crate without a docs section must fail `pnpm generate:bindings --check`, not just get skipped. Pinned in Task 2
5. Hand-written text outside the markers survives regeneration byte for byte. Pinned in Task 2

---

### Task 1: Upstream twin and Rust field names in the schema output

**Files:**

- Modify: `rust/textlane/src/actions/plugin/find.rs` (`PluginEntry`, `plugin!`, `PLUGINS`)
- Modify: `rust/textlane/src/actions/plugin/describe.rs`
- Modify: `rust/md/src/models/options.rs:93` (the `unsafe` field)
- Test: `rust/textlane/src/actions/plugin/describe_unit.rs` (create; declare it inline in `src/lib.rs` beside the module, as the other `_unit.rs` files are)

**Interfaces:**

- Produces: `describe_plugins()` lists each plugin as `{"name", "upstream", "options"}`. `upstream` is a string such as `"remark-gfm"`, or `null` for a plugin with no upstream (`md.parse` follows `remark-parse`, so few should be null). Any property whose Rust field differs from the camelCase-to-snake_case of its key carries `"x-rust-field": "<field>"`

- [ ] **Step 1: Write the failing tests** `describe_unit.rs`:
  - `lists_each_plugins_upstream`: `describe_plugins()["md"]["plugins"]` has an entry with `name == "md.gfm"` and `upstream == "remark-gfm"`, and an entry `html.sanitize` → `rehype-sanitize`
  - `marks_renamed_rust_fields`: the `$defs` property `unsafe` in the `md` domain has `x-rust-field` equal to the field's real name
- [ ] **Step 2: Run** `cargo test -p textlane --features schema describe_unit` from `rust/`. Expected: FAIL
- [ ] **Step 3: Implement.** Add `upstream: Option<&'static str>` to `PluginEntry` under `#[cfg(feature = "schema")]` and make it a required macro argument: `plugin!("md.gfm", textlane_md::Gfm, "remark-gfm")`. Fill all 31 entries, taking the names from each crate's plugin doc comments and the schema descriptions ("named as X"). `describe_plugins` emits it. Add `#[schemars(extend("x-rust-field" = "<field>"))]` on the renamed field
- [ ] **Step 4: Run** the same test. Expected: PASS. Also run `cargo clippy -p textlane --features schema -- -D warnings`
- [ ] **Step 5: Commit** `refactor: name each plugin's upstream library in the plugin schema`

### Task 2: Render options sections into docs pages

**Files:**

- Create: `.scripts/render-options.ts`
- Modify: `.scripts/generate-bindings.ts` (read the `upstream` field and the docs pages, add them to `files`, and extend the drift check)
- Test: `.scripts/-test/render-options.unit.ts`

**Interfaces:**

- Consumes: Task 1's `{name, upstream, options}` and `x-rust-field`
- Produces:
  - `type Language = "typescript" | "python" | "rust"`
  - `renderOptions(props: { language: Language; plugin: { name: string; upstream: string | null; options: Schema | null }; definitions: Record<string, Schema> }): string` returns the Markdown between the markers: an upstream line (`Follows [remark-gfm](https://github.com/remarkjs/remark-gfm).`), then either a table with Option, Type, Default and Description columns or the line `No options.`
  - `replaceSections(props: { text: string; render: (name: string) => string; names: string[] }): { text: string; missing: string[] }` replaces each `<!-- generated:options <name> -->…<!-- /generated -->` and returns names with no marker
  - Pages: `<lang>/docs/plugins-<domain>.md` for `md`, `html`, `mdx`, `nl`. Shared types (`Schema`) move from the generator into `render-options.ts` and are exported

- [ ] **Step 1: Write failing tests** in `render-options.unit.ts`:
  - `md.gfm` in TypeScript renders a row ``| `singleTilde` | `boolean` | `true` |`` and the remark-gfm line. Python renders the same key `singleTilde` with type `bool`. Rust renders `single_tilde`, with `bool` as the type and `true` as the default
  - `md.frontmatter` renders a non-empty Type column in each language, contains no `[object Object]`, and has rows for `Matter` linked or inlined
  - The `unsafe` property renders its `x-rust-field` name in Rust and `unsafe` in TypeScript
  - A unit plugin renders `No options.`
  - `replaceSections` keeps text outside the markers byte-identical, is idempotent when run twice, and returns `missing: ["md.gemoji"]` when that marker is absent
- [ ] **Step 2: Run** `vp test .scripts/-test/render-options.unit.ts`. Expected: FAIL
- [ ] **Step 3: Implement** `renderOptions` and `replaceSections`. Reuse the generator's `writeType` and `writePythonType` for the TypeScript and Python type cells. Rust types are shown in their JSON form (`bool`, `string`, `number`, a union written as `"yaml" | "toml"`). Escape `|` in cells. In the generator, for each language and domain, add the replaced page to `files`. With `--check`, exit 1 with `missing docs section: <lang> <plugin>` when `missing` is non-empty. Without `--check`, report missing sections as errors too, because a section needs hand-written prose
- [ ] **Step 4: Create the twelve pages**, `<lang>/docs/plugins-<domain>.md`, each with frontmatter (orders 6–9) and, per plugin in `PLUGINS` order, `## <plugin name>` plus an empty marker pair. Run `pnpm generate:bindings`
- [ ] **Step 5: Run** `vp test .scripts/-test/render-options.unit.ts .scripts/-test/bindings.unit.ts .scripts/-test/docs.unit.ts`. Expected: PASS
- [ ] **Step 6:** Update `.agents/skills/package-distribution` (or the skill that documents `generate:bindings`) to say it now writes the docs options sections too
- [ ] **Step 7: Commit** `chore: generate each plugin's options into the docs pages`

### Task 3: TypeScript plugin pages

**Files:** Modify `typescript/docs/plugins-{md,html,mdx,nl}.md`

- [ ] **Step 1:** Above each plugin's marker, write one sentence on what it does and one runnable example (import from `textlane` and `textlane/<domain>`, plus its output as a comment). Start each page with a short paragraph on the domain and which stage its parse/stringify plugins run
- [ ] **Step 2: Run** every snippet against `pnpm build`'s `typescript/build/`. Each must print its stated output
- [ ] **Step 3: Run** `pnpm generate:bindings --check && vp test .scripts/-test/docs.unit.ts`. Expected: PASS
- [ ] **Step 4: Commit** `docs: describe every plugin in the TypeScript section`

### Task 4: TypeScript guides

**Files:**

- Create in `typescript/docs/`: `getting-started.md` (3), `processor.md` (4), `recipes.md` (5), `syntax-trees.md` (10), `memory.md` (11), `errors.md` (12)
- Modify: `typescript/docs/overview.md` (move Memory out, reword the lead so JavaScript plugins joining is stated as the goal, not current behaviour), `typescript/docs/agent-skills.md` (order 13)

- [ ] **Step 1: Write the pages** with the content listed per page in the spec's Pages table:
  - **Processor:** every `Processor` method in `typescript/models/processor.ts`
  - **Errors:** each code the crate raises. Find them with `grep -rn 'Error::new("' rust/*/src`
  - **Syntax trees:** the node `type`s as `md.parse`, `html.parse` and `nl.parse` produce them, plus `position`
- [ ] **Step 2: Run** every snippet against the local build
- [ ] **Step 3: Run** `vp test .scripts/-test/docs.unit.ts` and `pnpm build:docs`. Expected: both succeed and the section lists pages in the global order
- [ ] **Step 4:** Point `.skills/textlane-usage/SKILL.md` at the new pages where it explains the same thing
- [ ] **Step 5: Commit** `docs: add the TypeScript guides`

### Task 5: Python port

**Files:** `python/docs/`: the same six new pages, the four plugin pages, `overview.md`, `agent-skills.md`

- [ ] **Step 1: Port Tasks 3–4** to Python. Use `from textlane import md, html, textlane`, snake_case plugin functions (`md.to_html()`), camelCase option keys, `process_sync`/`await process`, and files and trees as dicts. Processor methods come from `python/textlane/_processor.py`
- [ ] **Step 2: Run** every snippet against the wheel from `pnpm build` (`uv run` with `python/build/`)
- [ ] **Step 3: Run** `pnpm generate:bindings --check && vp test .scripts/-test/docs.unit.ts && pnpm build:docs`. Expected: PASS
- [ ] **Step 4: Commit** `docs: add the Python guides and plugin pages`

### Task 6: Rust port

**Files:** `rust/docs/`: the same six new pages, the four plugin pages, `overview.md`, `agent-skills.md`

- [ ] **Step 1: Port Tasks 3–4** to Rust:
  - Plugins use `Processor::new().with(md::Gfm { single_tilde: false, ..Default::default() })`
  - **Processor** covers `Processor`/`FrozenProcessor` (`rust/textlane/src/models/processor.rs`) and a short `Pipeline` section (`rust/textlane/src/models/pipeline.rs`), plus `Output`
  - **Syntax trees** describes the arena `Tree` from `textlane-core` and points item-level detail to docs.rs
- [ ] **Step 2: Run** every snippet as a scratch example (`cargo run --example` in a throwaway file under `$CLAUDE_JOB_DIR/tmp`, never committed)
- [ ] **Step 3: Run** `pnpm generate:bindings --check && vp test .scripts/-test/docs.unit.ts && pnpm build:docs`. Expected: PASS
- [ ] **Step 4: Commit** `docs: add the Rust guides and plugin pages`
