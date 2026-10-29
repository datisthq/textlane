---
name: human-documentation
description: Use when writing a design or decision page for people under `.humans/`, before committing — to find what AGENTS.md, the READMEs, the skills, the docs site and .humans now falsely claim — and when deciding where a fact belongs.
---

# Documentation

Keep every document true when the code changes, and put each fact where the next reader will
find it.

## Writing for `.humans/`

`.humans/` holds what people read about how textlane is designed and why: `architecture.html`,
`decisions.html` (the register of what is decided, the agent's calls, and what is open) and
`conformance.html` (the suite and its upstream catalogue). It is not the docs site, which teaches
users. `pnpm read` serves it with htmlpress; `.humans/htmlpress.yaml` holds the site title.

- **Pages follow htmlpress's contract** (its `htmlpress-authoring` skill): a `<title>`,
  `description`, `order` and `date` metas, one `<h1>`, then content only. No `<nav>`, no page
  chrome, no styling of text: htmlpress's theme and components do that (`htmlpress-grid` and
  `htmlpress-card` for headline numbers, `htmlpress-badge` for statuses, `htmlpress-steps`,
  `htmlpress-callout`, `htmlpress-file-tree`, `htmlpress-table`).
- **Tables hold short cells only.** htmlpress never wraps a cell and scrolls a wide table
  sideways, so a row of prose becomes a `<dl>`: the item as `<dt>`, its why, rejected or
  contents as `<dd>`.
- **Diagrams are inline SVG**, not Mermaid: drawn by hand to show the real structure (tiers,
  swim lanes per crate), styled from a page `<style>` scoped to `figure.diagram` that uses only
  `--htmlpress-*` tokens, so light and dark follow the theme. Wide ones scroll inside their
  `figure`.
- **Statuses**: `success` decided, `warning` the agent's call, `danger` open.
- **Numbers carry their date and source**, near the top.
- **A decision changes `decisions.html` in the same commit** as the code or spec it decides:
  add the entry with its status and the alternatives rejected, and move it out of Open.

`.scripts/-test/docs.unit.ts` reads only Markdown, so paths and commands an HTML page names go
unchecked: verify them by hand, and run `htmlpress build .humans --json` to confirm every page
builds.

## Keeping the docs true

Run this **before committing**, once the work is done and the diff is final:

```sh
git diff main...HEAD --stat
git diff main...HEAD
git status --short
```

The guard in `.scripts/-test/docs.unit.ts` already fails on a path or `pnpm` command that no
longer exists. This is the other half: a paragraph that still parses, still points at real
files, and is no longer true.

- **A rename.** Grep the docs for the _old_ name — plugin, option, AST node, binding export,
  script, folder. The single most common miss.
- **A new or deleted command, script or source folder.** The command list and code structure in
  `AGENTS.md` and the tables in `CONTRIBUTING.md` enumerate them.
- **A changed default or contract.** An option's default, a node's shape, what a binding
  accepts or returns.
- **A removed mechanism.** Search the docs for the thing you deleted.

## Where to look

- `AGENTS.md` — the only one, at the repo root.
- the repo skills in `.agents/skills/` and the shipped ones in `.skills/`, including each
  `description`, which decides whether a skill gets picked up at all.
- `README.md`, `rust/textlane/README.md`, `typescript/README.md`, `python/README.md` — the three package
  READMEs share their usage section; change all three together.
- `python/docs/`, `typescript/docs/` and `rust/docs/` — the user manual, one site section per language, in Markdown with livemark frontmatter; a page should build
  cleanly with `pnpm build:docs`. The three sections mirror each other: a page added to one gets its
  counterpart in the others, its `path` under that language's prefix.
- `.humans/` — the design and decision pages; a changed decision or crate layout makes them
  false like any other document.

## Where a fact belongs

`AGENTS.md` loads into every session, so it carries only what must hold before anyone knows which
area they are in: hard rules, invariants that fail silently, terminology, one line per command
and per source folder, and the skills tables. Procedure, rationale and an area's traps go in that
area's skill. Keep `AGENTS.md` near 10 KB.

- How to **use** textlane (pipelines, plugins, the bindings) → the matching page in each language's `docs/`
  and the shipped skill under `.skills/` covering it (`textlane-usage`), kept in step; each fact
  in exactly one.
- How to **work on** textlane → a repo skill in `.agents/skills/`.
- How a person contributes → `CONTRIBUTING.md`.

Do not restate the change in the docs; the story is what `git log` is for. If the diff adds no
invariant and removes none, the right outcome is **no documentation change**.
