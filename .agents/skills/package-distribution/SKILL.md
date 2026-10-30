---
name: package-distribution
description: Use when touching how textlane ships — the npm loader and its platform packages, the PyPI pyo3 wheels, crates.io, the release addons matrix, versions, or adding a platform. Covers the invariants between settings, workflow and registries.
---

# Package distribution

One workspace, three registries, one version. Nothing here may change what the crate does.

## npm: `textlane`

- `typescript/index.ts` loads the napi addon that `resolveAddon`
  (`typescript/actions/addon/resolve.ts`) finds and exports `textlane()`, a `Processor`
  (`typescript/models/processor.ts`) over the addon's `Pipeline` class: chaining, Promises
  and `data`'s overloads only; values cross napi as JSON (`serde-json`), so a JavaScript
  function in `use` fails in napi rather than vanishing.
- The subpaths `textlane/md`, `textlane/html`, `textlane/mdx` and `textlane/nl` are
  `exports` entries pointing at `build/generated/<domain>.js`. Their sources,
  `typescript/generated/<domain>.ts`, and Python's `python/textlane/<domain>.py` are written by
  `pnpm generate:bindings` from `cargo run --example schema --features schema`: the plugin
  table in `rust/textlane/src/actions/plugin/find.rs` and each options struct's
  `schemars::JsonSchema`, derived only under each crate's `schema` feature, so no release build
  pays for schemars. A plugin added to the table or an option changed in Rust is one
  `pnpm generate:bindings` away; `.scripts/-test/bindings.unit.ts` fails until it runs.
  A custom `Deserialize` needs a hand-written `JsonSchema` beside it (`md::Frontmatter`,
  smartypants' modes), and a `deserialize_with` field a `schemars(with = …)`.
  A plugin struct is its own options (`md::Gfm`), so schemars would name the binding type
  `Gfm`: each carries `schemars(rename = "GfmOptions")`, and a hand-written `schema_name`
  returns the `…Options` name too, or the generated types silently lose their suffix.
- `resolveAddon` takes `TEXTLANE_ADDON` first (a cargo build, an unsupported platform), else
  resolves `@textlane/<platform>-<arch>/textlane.node` from the installed packages.
- `PLATFORMS` in `typescript/settings.ts` is the single list. `scripts/pack.ts` reads it to lay
  out `compile/` in `typescript/`: one platform package per entry (`os`, `cpu`, the addon)
  and the main package with `optionalDependencies` pinned to its own version. Those pins are
  written at pack time, never kept in `typescript/package.json`: a pin to an unpublished version
  would break `pnpm install` in this workspace.
- The main package also carries `.skills/` as `skills/`, copied by `scripts/pack.ts`, so
  `npx skills add ./node_modules/textlane/skills` installs the version in the lockfile.
- `scripts/release.ts` publishes platform packages before the main one and skips any version
  already on the registry, so rerunning a half-failed release finishes it.
- Never a `postinstall` download: pnpm 10+ blocks lifecycle scripts by default, and it breaks
  offline installs.
- Local check without publishing: `cargo build --release --features node` from `rust/`, copy
  `libtextlane.so` from its `target/release/` (`.dylib` on macOS, `textlane.dll` on Windows) to
  `<dir>/<platform>/textlane.node`, `pnpm -C typescript build`, then
  `TEXTLANE_ADDON=<dir>/<platform>/textlane.node node -e 'import("./typescript/build/index.js").then(m=>console.log(m.version()))'`;
  `import("./typescript/build/generated/md.js")` reaches a subpath the same way.
  `pnpm -C typescript pack:platforms --addons <dir>` needs every platform's addon.

## PyPI: `textlane`

`python/pyproject.toml` and the package `python/textlane/` are the whole package: maturin's
mixed layout with `bindings = "pyo3"`, `features = ["python"]`,
`module-name = "textlane._native"` (the PyO3 module is `#[pymodule(name = "_native")]`) and
`manifest-path = "../rust/textlane/Cargo.toml"`. `textlane/__init__.py` re-exports the
extension's `version`, `plugin_names` and `TextlaneError`, and `run_case` when a `testing`
build has it; `_processor.py` is the processor, crossing the boundary as JSON text;
`_native.pyi` and `py.typed` type it. `scripts/test.sh` runs every `-test/*_unit.py` against
the installed wheel, never the source folder, which lacks the extension.
`dynamic = ["version"]` gives the wheel the crate's version. PyO3's `abi3-py39` makes one
wheel per platform serve every CPython from 3.9. A wheel holds the skills under
`.data/data/share/textlane/skills/`, installed to `<venv>/share/textlane/skills`. maturin reads
them from the `textlane.data` folder beside `pyproject.toml` (its `data` key), which
`python/scripts/copy-skills.sh` stages and git ignores: the local build and the release
workflow's Stage Skills step both run it, and a wheel built without it ships no skills. CI builds
one wheel per target with `PyO3/maturin-action`; no sdist is published, since the crate sits
outside `python/`. Check locally: `pnpm -C python build`, then
`uv run --with python/build/<wheel> --no-project python -I -c 'import textlane; print(textlane.version())'`;
`-I` keeps a `textlane/` source folder in the working directory from shadowing the wheel.
maturin runs from `python/`, outside `rust/`, so `rust/rust-toolchain.toml` does not
apply there: `python/package.json` and `scripts/test.sh` pin `RUSTUP_TOOLCHAIN=stable`, or a
machine whose rustup default is an older nightly fails on oxc's minimum rustc.

## crates.io: `textlane`

`cargo publish --workspace` from `rust/` publishes every member in dependency order, the domain
crates before `textlane`. A binding never splits along them: one npm or PyPI package per domain
would mean one addon each, and two addons cannot share a tree without serializing it. The `include` list in `rust/textlane/Cargo.toml` keeps `src/`, `build.rs`,
`README.md` and `LICENSE.md`; a file the crate reads from outside them would compile here and
fail for `cargo add`.

## The release workflow

`.github/workflows/release.yaml`: release-please → an `addons` matrix (one job per platform:
`cargo build --release --target <t> --features node`, the library collected as
`addons/<platform>/textlane.node`, the skills staged, the wheel built; both uploaded as
artifacts) → `npm`, `pypi` and `crates` jobs. Adding a platform means two edits in one commit: a
`PLATFORMS` entry and a matrix row with the same key and target. The pack script refuses to run
while any platform's addon is missing.

A run that failed after release-please tagged cannot be fixed by a re-run: it reuses the old
workflow file and `release_created` stays false on later pushes. Delete the GitHub release and
tag, relabel the merged Release PR `autorelease: pending`, then push the fix to `main`;
release-please re-creates the release and the fixed workflow publishes it.

## Versions

One release-please component at `.`; `rust/Cargo.toml` (`workspace.package.version` and each
member's pin in `workspace.dependencies`), `rust/Cargo.lock` (one entry per crate) and
`typescript/package.json` are `extra-files`, `version.txt` the simple strategy's own file. Never
bump any of them by hand. The lockfile entry needs `@.name.value` in its jsonpath, since
release-please wraps TOML values; without it `cargo publish` finds a dirty lockfile and refuses.

## First publish

Trusted publishing only covers packages that exist: the first npm publish of each platform
package and the first `cargo publish` of each crate need tokens (`NPM_TOKEN`, `CARGO_REGISTRY_TOKEN`); PyPI
accepts a pending trusted publisher for a new project. Publish a `0.0.0` placeholder by hand so
release-please's first version still reaches CI; a token made for that first publish may lack
`publish-update` and cannot publish later versions. Unscoped npm names can trip npm's spam
check, hence the `@textlane` scope.
