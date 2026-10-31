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
  and `data`'s overloads only; options and data cross napi as JSON (`serde-json`), so a
  JavaScript function in `use` fails in napi rather than vanishing; trees cross as JSON text,
  which `JSON.parse` reads faster than napi builds objects, and napi caps a `Value` at 1M
  nodes. `process` returns a `#[napi(object)]`
  `ProcessedFile` rather than a `Value`, which would copy the HTML once more, its ASCII
  `value` as a `Latin1String`, which V8 copies without decoding UTF-8 (1–2%); taking
  `Uint8Array` input instead of a string measured no faster, the string copy being cheap.
- The subpaths `textlane/md`, `textlane/html`, `textlane/mdx` and `textlane/nl` are
  `exports` entries pointing at `build/generated/<domain>.js`. Their sources,
  `typescript/generated/<domain>.ts`, and Python's `python/textlane/<domain>.py` are written by
  `pnpm generate:bindings` from `cargo run --example schema --features schema`: the plugin
  table in `rust/textlane/src/actions/plugin/find.rs` and each options struct's
  `schemars::JsonSchema`, derived only under each crate's `schema` feature, so no release build
  pays for schemars. The same run renders each plugin's options section, between
  `<!-- generated:options <plugin> -->` and `<!-- /generated -->`, into the docs pages
  `<language>/docs/plugins-<domain>.md` (`.scripts/render-options.ts`). A plugin added to the
  table or an option changed in Rust is one `pnpm generate:bindings` away;
  `.scripts/-test/bindings.unit.ts` fails until it runs, and the script fails outright while
  a plugin has no section on its page in each language — write its prose and marker first.
  Each `plugin!` entry names its unified upstream (`"remark-gfm"`). Every language's tables
  show the JSON keys; the Rust pages list them as `Pipeline` takes them.
  A custom `Deserialize` needs a hand-written `JsonSchema` beside it (`md::Frontmatter`,
  smartypants' modes), and a `deserialize_with` field a `schemars(with = …)`.
  A plugin struct is its own options (`md::Gfm`), so schemars would name the binding type
  `Gfm`: each carries `schemars(rename = "GfmOptions")`, and a hand-written `schema_name`
  returns the `…Options` name too, or the generated types silently lose their suffix.
- The generator marks each `<domain>.parse` with `Parser<Root>` (`PARSED_TREES`: mdast, hast,
  nlcst), a phantom optional key `use()`'s conditional return reads, so `parse` is typed after
  `use(md.parse())` with no cast. A plugin lacking the key infers `unknown` and keeps the tree;
  making the key required breaks every generated `return { name }`. `@types/unist`, `mdast`,
  `hast`, `nlcst` and `estree-jsx` are runtime `dependencies` with caret ranges, so consumers
  dedupe them; `index.ts` re-exports them as type namespaces from `generated/trees/<tree>.ts`.
- The extension nodes reach npm without the utilities' code: `.scripts/render-extensions.ts`
  copies the `declare module "mdast" | "hast"` blocks and node interfaces of
  `mdast-util-{definition-list,directive,math,mdx-*,mdxjs-esm,to-hast}` (resolved through the
  root's devDependencies) into `typescript/generated/extensions/`, plus a hand-written `toml`,
  which no package declares. The copies must stay identical to the utilities': a drifted
  property makes TS2717 for anyone installing both. A new syntax extension whose utility
  augments mdast needs an `EXTENSIONS` entry, or its nodes are missing in both languages.
- Python's trees mirror them: the same run renders `python/textlane/{unist,mdast,hast,nlcst,estree}.py`
  (`.scripts/render-trees.ts`) with the TypeScript checker over the `@types` that `typescript/`
  resolves plus those copies, served from memory, so module resolution needs their directory
  faked too (`directoryExists`), or a clean run silently drops every extension node; each
  namespace re-exports the copies' names the renderer filed under that tree. Each interface becomes a
  flattened `TypedDict` (optional keys in a `total=False` subclass, since 3.9 has no
  `NotRequired`), each alias a module-level `Union` at the end, cross-module names
  `TYPE_CHECKING`-only (mdast and hast name each other). Keys class syntax cannot hold
  (`async`, `update`) use the functional form with quoted values. `md.parse()` returns
  `md.Parser`, a `Plugin` with a never-set `parsed: Root` key that `Processor.use`'s
  overloads match, making `Processor[mdast.Root]`; a plain `Plugin` lacks the key, so it
  falls through to `Processor[Tree]`. ty narrows these unions on `node["type"] == …` but not
  on `"key" in node` or `type in (…)`. `pnpm -C python check:types` runs ty over the package
  and `-test/trees_unit.py`, whose runtime test asserts every parsed node's keys fit its
  `TypedDict`.
- `resolveAddon` takes `TEXTLANE_ADDON` first (a cargo build, an unsupported platform), else
  resolves `@textlane/<platform>-<arch>/textlane.node` from the installed packages, with a
  `-musl` suffix when `detectLibc` finds no glibc in `process.report`'s header, the check npm
  itself makes for a package's `libc` field. Every Linux platform package carries `libc`
  (`glibc` or `musl`), so Alpine installs only its own addon.
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
build has it; `_processor.py` is the processor, crossing the boundary as JSON text, except that
`process` gets the HTML back as a `str` of its own beside the file's messages and data:
inside the JSON, encoding and parsing it cost Python 28% of a 1 MB README;
`_native.pyi` and `py.typed` type it. `scripts/test.sh` runs every `-test/*_unit.py` against
the installed wheel, a release build, since a debug one overflows Windows' stack on deep trees, never the source folder, which lacks the extension.
`dynamic = ["version"]` gives the wheel the crate's version. PyO3's `abi3-py39` makes one
wheel per platform serve every CPython from 3.9. A wheel holds the skills under
`.data/data/share/textlane/skills/`, installed to `<venv>/share/textlane/skills`. maturin reads
them from the `textlane.data` folder beside `pyproject.toml` (its `data` key), which
`python/scripts/copy-skills.sh` stages and git ignores: the local build and the release
workflow's Stage Skills step both run it, and a wheel built without it ships no skills. CI builds
one wheel per target with `PyO3/maturin-action`, manylinux and musllinux for Linux. No sdist
is published: building it needs Rust and a C compiler (oniguruma), which a `pip install` on
an unlisted platform rarely has, so a clear "no wheel" error beats a failed compile. Check locally: `pnpm -C python build`, then
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
`cargo build --release --target <t> --features node`, or for a musl row the same in a
`rust:1-alpine` container with `-C target-feature=-crt-static`, since rustc drops a cdylib
against static musl; the musl wheel takes `manylinux: musllinux_1_2`; the library collected as
`addons/<platform>/textlane.node`, the skills staged, the wheel built; both uploaded as
artifacts) → `npm`, `pypi` and `crates` jobs. Adding a platform means two edits in one commit: a
`PLATFORMS` entry and a matrix row with the same key and target. The pack script refuses to run
while any platform's addon is missing. Its new `@textlane/<key>` package also needs the first
publish below before that release runs, or the tokenless `npm` job fails on it.

A run that failed after release-please tagged cannot be fixed by a re-run: it reuses the old
workflow file and `release_created` stays false on later pushes. Delete the GitHub release and
tag, relabel the merged Release PR `autorelease: pending`, then push the fix to `main`;
release-please re-creates the release and the fixed workflow publishes it.

Every checkout after `release` takes `ref: needs.release.outputs.tag`. release-please reads
`main` live, so a run for an earlier push tags a Release PR merged while its tests ran, and a
checkout of the pushed commit then builds the previous version: npm skips it as already
published, crates.io and PyPI refuse it, and the new tag ships nothing.

## Versions

One release-please component at `.`; `rust/Cargo.toml` (`workspace.package.version` and each
member's pin in `workspace.dependencies`), `rust/Cargo.lock` (one entry per crate) and
`typescript/package.json` are `extra-files`, `version.txt` the simple strategy's own file. Never
bump any of them by hand. The lockfile entry needs `@.name.value` in its jsonpath, since
release-please wraps TOML values; without it `cargo publish` finds a dirty lockfile and refuses.

## First publish

Trusted publishing only covers packages that exist: each npm package and each crate gets a
`0.0.0` placeholder published by hand (npm's 2FA is a browser prompt, so run it yourself), then
a trusted publisher; PyPI accepts a pending trusted publisher for a new project. The `npm` job
carries no token: each package is `npm trust github <pkg> --file release.yaml --repo
datisthq/textlane --env release --allow-publish` and `npm access set mfa=publish <pkg>`, which
forbids tokens, so a package missing either fails CI. A crates.io token made for the first
publish may lack `publish-update`. Publish the crates with a fresh `CARGO_TARGET_DIR`: a local
`cargo publish` treats each `0.0.0` it packaged before as immutable and verifies against that
stale build. crates.io rate-limits a burst of new crates with a 429 naming when to retry, so the last one
may need a rerun with `-p textlane` once the 429's time passes. Unscoped npm names can trip npm's spam check, hence the
`@textlane` scope.
