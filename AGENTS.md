# textlane

Textlane is a Rust-native text processing pipeline: a replacement for unified whose plugins run natively, with existing JavaScript plugins still able to join a pipeline. It is one cargo workspace in `rust/`, shipped three ways: crates.io (`cargo add textlane`), npm (`textlane` in `typescript/`, a loader plus one prebuilt napi addon per platform) and PyPI (`textlane` in `python/`, PyO3 wheels built by maturin), all at one version. The repo root holds the tooling shared across them and textlane's own docs site, written in Markdown under `python/docs/`, `typescript/docs/` and `rust/docs/` and built with livemark.

## Terminology

- **crate** — the `textlane` library in `rust/textlane/` and the crates beside it that it re-exports (`textlane-core` and one `textlane-<domain>` per domain: `md`, `html`, `mdx`, `prose`); the only place behaviour lives
- **binding** — a package that only exposes the crate: `typescript/` for npm (a napi addon, the `node` feature), `python/` for PyPI (a PyO3 extension, the `python` feature). A binding never reimplements a plugin
- **platform package** — `@textlane/<os>-<arch>`, one per entry in `PLATFORMS` (`typescript/settings.ts`), each carrying one `textlane.node` addon; the main `textlane` package pins them all as `optionalDependencies` at its own version
- **pipeline** — a processor built with `.use()`: parse, transform and serialize stages run in order over a syntax tree
- **plugin** — one pipeline stage, native Rust or a host-language (JavaScript, Python) function called through a binding
- **conformance suite** — language-neutral JSON cases, each a pipeline, an input and the expected output, in the style of the JSON-Schema test suite; every implementation runs the same cases
- **docs site** — this repo's livemark project: `livemark.config.ts`, `.livemark/` and the Python, TypeScript and Rust sections, each from its `docs/` pages, `README.md` and `CONTRIBUTING.md` symlink, plus the root `CHANGELOG.md` as the header's Changelog section, built by `pnpm build:docs` and deployed to textlane.dev

## Rules

- **Never commit unless asked to** — except that a session working directly on `main` commits each task once done. The root's index is shared, so always commit with a pathspec: `git commit -m "…" -- <your paths>` — `git add` new files first, since a pathspec commit silently skips untracked ones.
- **Never add co-authored by Claude Code to commits!**
- **Never mark a commit breaking** — no `!` and no `BREAKING CHANGE:` footer; release-please turns either into a major bump.
- **Never push main or pull requests to origin unless asked to.**
- **Never amend or rewrite existing commits unless asked** (`--amend`, `reset`, `rebase`, force-squash). Fold changes in with a follow-up commit.
- **Never change code unless asked to.** Usually just answer.
- When resolving a TODO, follow its instructions literally. Prefer LSP capabilities where available.
- Start a plan from a summary of the new or changed API (plugins, options, AST nodes, bindings) and the hardest points.
- **Every plan ends by asking how to work it** (`AskUserQuestion`, before finalising): **the current branch** or **a `.worktrees/<name>` branch** (**`worktree-workflow`**).
- **Behaviour lives in the crate only.** A binding loads the addon and maps types, nothing more; a feature added to `typescript/` or `python/` would exist for one language and not the others.
- **Plugin names, their options and the AST node shapes are public API.** What `.use()` accepts, each plugin's options and the mdast/hast nodes a pipeline produces: renaming one silently breaks every pipeline and plugin relying on it. Treat it like an export change (**`change-review`**).
- **A platform entry is a release contract.** `PLATFORMS` in `typescript/settings.ts` and the `addons` matrix in `.github/workflows/release.yaml` must list the same platforms with the same Rust targets; one missing from the matrix fails the npm pack (**`package-distribution`**).
- Rust: each `actions/<entity>/<verb>.rs` holds one action, named `verb_noun` (`actions/package/version.rs` exports `read_version`); the module tree is declared inline in each crate's `src/lib.rs`. Beside `actions/` sit `models/` (types), `helpers/` (small free functions), `plugins/<name>/` (one per `.use()` stage: its `plugin.rs` and own `actions/`) and `utils/<name>/`, a bounded internal system (a tokenizer, a serializer) whose files may be mutually recursive and are exempt from one action per file. TypeScript: the same `actions/<entity>/<verb>.ts` shape, full relative imports with the `.ts` extension, no barrels.
- Don't destructure props in a TypeScript function signature — use `props.x`. Put higher-level items above the ones they compose, public before private.
- **Comments must not outnumber code**, and one is at most ~3 lines. The bar is: **name the specific mistake this prevents** — if you cannot, delete it. A file with none is normal. Rationale and history belong in git; only a non-obvious invariant (an ordering types cannot express, a failure that looks like success, a deliberate deviation from a rule here) earns lines.
- Doc comments only on public API — a brief line on what an exported item does; none on files.
- Update docs when a change requires it, and **after every task check whether a skill needs the lesson it taught** — update it in the same commit, or add one when no skill covers the area. Where a fact belongs: the **`human-documentation`** skill.
- **End every response that finishes a task with a status line: one bold sentence of about ten words, as the final paragraph, nothing after it** (e.g. **Ready to commit; lint, types and units green.**).

## Skills

Depth lives in skills: this file carries invariants, the skills carry procedure. Reach for one before working in its area. **A repo skill beats a plugin skill on the same subject** — `change-verification` over verification-before-completion, `change-review` over a generic code review. Each is named `<subject>-<facet>`, subject first.

Repo skills live in `.agents/skills/` (reached by Claude Code through the `.claude/skills` symlink `pnpm install` creates) and are never published:

| skill                   | when                                                                       |
| ----------------------- | -------------------------------------------------------------------------- |
| `benchmark-development` | speed and memory against other Markdown libraries, in `benchmark/`         |
| `change-review`         | reviewing a pull request or the working diff — also what CI review runs    |
| `change-verification`   | checking a change works — the smallest lint/type/unit slice and a look     |
| `conformance-suite`     | adding or porting conformance cases, the runners and the `testing` feature |
| `human-documentation`   | a page for people under `.humans/`, the docs-truth pass, where facts go    |
| `issue-resolution`      | taking one issue or bug report to one tested commit in a worktree          |
| `package-distribution`  | the npm and PyPI bindings, platform packages and the release addons matrix |
| `rust-development`      | working in `rust/` — layout, cargo commands, features, the bindings        |
| `worktree-workflow`     | starting, running and landing a `.worktrees/<name>` branch                 |

Shipped skills live in `.skills/` and teach coding agents to **use** textlane; they are published on the docs site and in the npm and PyPI packages. Read the matching one before changing the behaviour it describes, and update it in the same commit:

| skill            | when                                                            |
| ---------------- | --------------------------------------------------------------- |
| `textlane-usage` | installing textlane and calling it from Rust, Node.js or Python |

**There is exactly one `AGENTS.md`, at the root.** `.scripts/-test/docs.unit.ts` enforces that, along with every path and `pnpm` command these documents name and the two tables above.

## Commands

- `pnpm install` — dependencies, lefthook, the `.claude/skills` symlink and `cargo fetch`
- `pnpm start` — the livemark dev server over the docs site · `pnpm build:docs` — its production build into `.livemark/build`
- `pnpm read` — the htmlpress dev server over `.humans/`, the design notes and decisions for people
- `pnpm build` — the release crate (`cargo build --release`), the npm loader into `typescript/build/` with `tsgo`, then the wheel into `python/build/` with maturin (needs `uv`)
- `pnpm test` — full gate: lint + type + unit · `pnpm check` — lint + type
- `pnpm check:lint` — `vp fmt --check && vp lint`, then `cargo fmt --check` and clippy · `pnpm format` — `vp fmt`, `cargo fmt` · `pnpm check:types` — `tsgo --noEmit`, then `cargo check`
- `pnpm test:unit` — the repo guards, the loader's units, `cargo test` with the `testing` feature, then the Python conformance runner (needs `uv`). Single test: `cargo test name` from `rust/`, or `vp test path/to/file.unit.ts` from `typescript/`. While iterating, verify the slice the diff reaches, not the full gate (**`change-verification`**)
- `pnpm measure:coverage` — the unit tests with coverage
- `pnpm scan:leaks` / `pnpm scan:vulns` / `pnpm scan` — gitleaks / semgrep / both
- `pnpm update:deps` — bump npm dependencies past a 7-day cooldown
- `pnpm -C benchmark setup:tools` / `pnpm -C benchmark measure` / `pnpm -C benchmark show:report` — the Markdown-to-HTML benchmark against other libraries (**`benchmark-development`**)
- `pnpm -C typescript pack:platforms --addons <dir>` / `pnpm -C typescript release` / `pnpm -C python release` / `pnpm -C rust release` — lay out and publish npm / PyPI / crates.io; run by the release workflow only
- `pnpm generate:bindings` — rewrite `typescript/generated/<domain>.ts` and `python/textlane/<domain>.py` (one function per plugin, option types from the crate's `schema` feature); `.scripts/-test/bindings.unit.ts` fails when they drift (**`package-distribution`**)
- `pnpm squash <branch> -m "<subject>"` — land a worktree branch on local `main` as one commit (**`worktree-workflow`**)

## Code structure

The root is a private pnpm workspace (`package.json`, `pnpm-workspace.yaml`): format, lint, the docs site, the repo guards and the scripts that chain each section with `pnpm -C <section>` — never `pnpm -r`.

- `rust/` — the cargo workspace, a virtual `Cargo.toml` over one sibling directory per crate, on the toolchain `rust/rust-toolchain.toml` pins (**`rust-development`**)
  - `textlane/` — the crates.io package `textlane`: `src/actions/<entity>/<verb>.rs` (`package/version`, `case/run`, `plugin/find`, the one table of public plugins); `src/models/processor.rs` the processor (`.with()`, `freeze()`, `process()`); `src/models/pipeline.rs` the name-based `Pipeline` the bindings drive, with unified's `use`, `data` and `freeze`; `src/bindings/` — `node.rs` (napi, the `node` feature) and `python.rs` (PyO3, the `python` feature), each only mapping `Pipeline` to its language; `examples/schema.rs` prints every plugin's option schema (the `schema` feature); `build.rs` runs napi's setup under `node`; `src/lib.rs` the façade, re-exporting each domain crate as a module of its name
  - `core/` — `textlane-core`: the unist, mdast, hast and nlcst trees in a per-document arena, positions, the file and the plugin contract (its `json` feature reads and writes unified's JSON) every other crate shares
  - `md/`, `html/`, `mdx/`, `prose/` — `textlane-<domain>`, each domain's plugins, depending on `textlane-core` only
  - `md/src/` — a port of micromark and the mdast utilities: `utils/` holds the engines (`tokenizer`, `compiler`, `serializer`, `converter`), `actions/<construct|node>/<verb>.rs` one handler each (`tokenize`, `resolve`, `compile`, `stringify`, `convert`), `plugins/` the public stages and syntax extensions
  - `html/src/` — a port of parse5 and hast-util-from-parse5: `utils/tokenizer` (the tokenizer, pulled one token at a time), `utils/builder` (the tree builder over a linked arena DOM, then the conversion to hast), `utils/converter` (hast to mdast, hast-util-to-mdast's state, its flat node store and `wrap`), `helpers/` the tag, entity, foreign-content and property-information tables
  - Unit tests beside their module as `<module>_unit.rs`
  - `docs/` — the docs site's Rust section, outside the crate's `include`
- `typescript/` — the npm package `textlane`: `index.ts` loads the addon `actions/addon/resolve.ts` finds and exports `textlane()`; `models/processor.ts` the processor over the addon's pipeline; `generated/` the `textlane/<domain>` subpaths; `settings.ts` lists the platforms; `scripts/` packs and publishes the platform packages into `typescript/compile/` (**`package-distribution`**); `docs/` the docs site's TypeScript section
- `conformance/` — the language-neutral conformance suite and a private workspace package: `test-schema.json`, `tests/<area>/<feature>.json`, `pending.json` (what the crate does not pass yet) and `sources/` importers (`pnpm -C conformance import:commonmark`, one script per source), run by `rust/textlane/tests/conformance.rs`, `typescript/-test/conformance.unit.ts` and `python/-test/conformance_unit.py` (**`conformance-suite`**)
- `python/` — the PyPI package `textlane`: `pyproject.toml`, which points maturin at `rust/textlane/Cargo.toml` with `bindings = "pyo3"` and the `python` feature, building `textlane._native` into the Python package `textlane/` (the processor in `_processor.py`, the generated domain modules), `scripts/copy-skills.sh`, which stages the shipped skills as wheel data, and `scripts/test.sh`, which builds a `testing` wheel and runs the unit tests and the conformance suite; `docs/` the docs site's Python section
- `benchmark/` — a private workspace package timing textlane against other Markdown libraries: seeded documents, `adapters/` per language (npm deps here, a uv project, an own-workspace cargo crate), `.cache/report.html` (**`benchmark-development`**)
- `.skills/` — the shipped agent skills
- `README.md`, `CONTRIBUTING.md` — the GitHub overview and the contributor guide, symlinked as `python/CONTRIBUTING.md`, `typescript/CONTRIBUTING.md` and `rust/CONTRIBUTING.md`, since livemark mounts a file at one path only; `rust/textlane/README.md`, `typescript/README.md` and `python/README.md` are each registry's page

Dot-directories hold non-package code: `.agents/` repo skills, `.humans/` design notes, decisions and research for people (htmlpress pages, served by `pnpm read`), `.skills/` shipped skills, `.livemark/` the docs site's escapes, `.release/` release-please config, `.scripts/` repo-wide guards (`.scripts/-test/`), the `pnpm generate:bindings` and `pnpm squash` scripts, whose tests the root `vite.config.ts` runs.

## Conventions

- pnpm `^11`, Node `^24`, ES Modules; vite-plus format and lint (2-space, LF, UTF-8, no semicolons, 90 columns)
- Strict TypeScript with null checks; no explicit return types. Never use `any`, an `as` cast or `!` without permission
- PascalCase types; camelCase values; TypeScript functions are **verbNoun** (`resolveAddon`)
- `package.json` script names are `verb[:object]` (`check:lint`, `build:docs`), never noun-first
- Specs: Vitest via `vite-plus/test`, `<module>.unit.ts`, no "Arrange / Act / Assert" comments
- Rust: edition 2024, rustfmt (90 columns), clippy with `-D warnings`; snake_case **verb_noun** functions; no `unwrap` / `expect` / `unsafe` outside tests (**`rust-development`**)
- Docs pages in `python/docs/`, `typescript/docs/` and `rust/docs/` are Markdown with livemark frontmatter (`title`, `description`, `path` under the language's prefix, `icon`, `order`); each package's `README.md` becomes its section's Introduction and its `CONTRIBUTING.md` symlink its Contributing page, and the root `CHANGELOG.md` is a `changelog` section in the header, split per version with the latest beside its tab, all through `patches` and `sections` in `livemark.config.ts`
- Commits: Conventional Commits, typed by what an installer gets (below)

## Releases

Conventional Commits drive [release-please](https://github.com/googleapis/release-please): a `feat` or `fix` landing on `main` opens a Release PR, and merging that PR cuts the version, tags it and publishes. Merging is the gate — nothing ships without it.

- **One version for everything.** The repo is one release-please component at `.` (`release-type: simple`, `version.txt`), tagged `vX.Y.Z` with one root `CHANGELOG.md`; `rust/Cargo.toml` (the workspace version and member pins), `rust/Cargo.lock` and `typescript/package.json` are its `extra-files`, and the wheel takes the crate's version through maturin. The manifest holds the current version and release-please maintains it — never hand-edit it
- **IMPORTANT: only changes to what an installer gets use `feat` or `fix`** — each crate's `rust/<crate>/src/`, the `[package]`, `[lib]`, `[features]` and `[dependencies]` tables of its `Cargo.toml`, `[workspace.dependencies]` in `rust/Cargo.toml`, `typescript/index.ts`, `typescript/actions/`, `typescript/settings.ts`, the `exports` and `files` of `typescript/package.json`, `python/pyproject.toml`, the shipped `.skills/`. Everything else MUST be `chore`, `docs`, `test` or `refactor`: CI and workflows, release config, dependencies, tooling, `AGENTS.md`, `.agents/skills/`, the READMEs, the docs site. `test` is for the conformance suite and unit tests; `refactor` is for restructuring that changes nothing an installer observes, even inside the paths above. release-please bumps neither
- This is not a style preference. release-please reads the commit type: a `fix:` on a workflow file cuts a release and burns a version number on a change no consumer can observe. Version numbers can never be reused
- When in doubt, ask whether the change alters what someone installing the package gets. If it does not, it is `chore`
