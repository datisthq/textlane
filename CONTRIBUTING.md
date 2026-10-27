# Contributing

Textlane is an open-source project and we'd love your help — whether that's fixing a bug, polishing the docs, or proposing a new feature. This guide walks through everything you need to get a change from your editor into a release.

## Quick start

You'll need [Node.js](https://nodejs.org/) 24+, [pnpm](https://pnpm.io/) 11+ and [rustup](https://rustup.rs/); for building the Python wheel, [uv](https://docs.astral.sh/uv/). Then:

```bash
git clone https://github.com/datisthq/textlane.git
cd textlane
pnpm install
pnpm start
```

`pnpm install` also fetches the Rust crates when `cargo` is on your `PATH`. `pnpm start` serves the docs site in `rust/docs/` with livemark, with live reload. To run every check:

```bash
pnpm test
```

## Repository layout

```
textlane/
├── rust/                # the crate and the crates.io package `textlane`
│   ├── src/
│   │   ├── actions/     #   package/version
│   │   ├── bindings/    #   node.rs (napi, `node` feature), python.rs (PyO3, `python` feature)
│   │   └── lib.rs       #   the library façade
│   ├── build.rs         #   napi setup under the `node` feature
│   └── docs/            #   the docs site's Markdown pages, built with livemark
├── typescript/          # the npm package `textlane`: loads the native addon
│   ├── actions/         #   resolveAddon — find the platform package's addon
│   ├── index.ts         #   the package entry
│   ├── scripts/         #   pack and publish the per-platform packages
│   └── settings.ts      #   the platform list
├── python/              # the PyPI package `textlane`: maturin PyO3 wheels
├── .skills/             # shipped agent skills for people who use textlane
├── .agents/skills/      # repo skills for coding agents (see AGENTS.md)
├── .livemark/           # the docs site's livemark escapes
├── .scripts/            # repo-wide guards and the `pnpm squash` script
├── livemark.config.ts   # the docs site's config
└── vite.config.ts       # vite-plus shared config (format, lint, the guards' tests)
```

All behaviour lives in `rust/`. The npm and PyPI packages only expose the same crate, so a feature never lands in a binding.

## Local commands

Run these at the repo root; each chains into the sections with `pnpm -C <section>`.

| Command            | What it does                                                     |
| ------------------ | ---------------------------------------------------------------- |
| `pnpm install`     | Install dependencies                                             |
| `pnpm build`       | Build the release crate, the npm loader and the Python wheel     |
| `pnpm check:types` | Type-check via `tsgo --noEmit` and `cargo check`                 |
| `pnpm check:lint`  | Format check + lint (`vp fmt`, `vp lint`, `cargo fmt`, `clippy`) |
| `pnpm check`       | `pnpm check:lint && pnpm check:types`                            |
| `pnpm format`      | Auto-fix formatting (`vp fmt`, `cargo fmt`)                      |
| `pnpm test:unit`   | Run the repo guards, the loader's tests and `cargo test`         |
| `pnpm test`        | `pnpm check && pnpm test:unit`                                   |
| `pnpm start`       | Serve the docs site with livemark and live reload                |
| `pnpm build:docs`  | Production build of the docs site                                |
| `pnpm scan`        | Secret and vulnerability scans (gitleaks, semgrep)               |

A change is ready to push when `pnpm test` is green.

## Code conventions

- **Rust**: edition 2024 on the pinned stable toolchain, clippy-clean with no `unwrap`/`expect` outside tests; unit tests beside the module as `<module>_unit.rs`, run one with `cargo test <name>` from `rust/`.
- **TypeScript**: strict mode is on. Don't use `any`, `as`-casts, or non-null `!` without permission. Full ESM imports with the `.ts` extension; tests beside the module as `<module>.unit.ts`.
- **Comments**: docs on public items only. Skip narrative comments inside function bodies.
- **Docs pages**: Markdown in `rust/docs/` with livemark frontmatter.
- **Formatting**: `pnpm format`. 2-space indent, no semicolons, double quotes (LF, UTF-8); rustfmt at 90 columns.

## Proposing a change

1. **Open an issue first** for anything bigger than a typo or a small bug. It's easier to align on direction before code.
2. **Branch from `main`** with a short descriptive name (e.g. `fix-heading-ids`, `feat-gfm`).
3. **Write a focused commit history**. Textlane uses [Conventional Commits](https://www.conventionalcommits.org/) — the prefix drives release-please:
   - `fix:` — bug fix → patch release
   - `feat:` — new feature → minor release
   - `chore:`, `docs:`, `refactor:`, `test:`, `perf:` — no release
   - Only changes to what an installer gets use `feat` or `fix`; never mark a commit breaking (`!` or a `BREAKING CHANGE:` footer)
4. **Run `pnpm test`** before pushing.
5. **Open a PR** against `main`. Describe the _why_ (link the issue), include input and output for any visible change, and call out anything reviewers should pay extra attention to.

## Releases

Releases are driven by [release-please](https://github.com/googleapis/release-please): a `feat` or `fix` landing on `main` opens a Release PR, and merging it cuts the version, tags it, builds the native addon and wheel for every platform and publishes them to crates.io, npm and PyPI at one shared version. Maintainers don't bump versions manually.

## Where to find help

- **Bugs and feature requests**: [GitHub Issues](https://github.com/datisthq/textlane/issues)
- **Discussion / ideas**: [GitHub Discussions](https://github.com/datisthq/textlane/discussions)
- **Source**: [github.com/datisthq/textlane](https://github.com/datisthq/textlane)

Thanks for helping make Textlane better.
