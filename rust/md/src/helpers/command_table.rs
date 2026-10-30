//! `md.packageCommand`'s tables: npm-to-yarn 3.2.0's, copied as they are, then textlane's own
//! for Python and Rust, where each manager's entry is `None` when it has no equivalent.

/// npm-to-yarn's `unchangedCLICommands`, its duplicate `test` kept.
pub(crate) const UNCHANGED_COMMANDS: [&str; 10] = [
    "test", "login", "logout", "link", "unlink", "publish", "cache", "start", "stop",
    "test",
];

/// npm-to-yarn's `yarnCLICommands`.
pub(crate) const YARN_COMMANDS: [&str; 35] = [
    "init",
    "run",
    "add",
    "audit",
    "autoclean",
    "bin",
    "check",
    "config",
    "create",
    "dedupe",
    "generate-lock-entry",
    "global",
    "help",
    "import",
    "info",
    "install",
    "licenses",
    "list",
    "lockfile",
    "outdated",
    "owner",
    "pack",
    "policies",
    "prune",
    "remove",
    "self-update",
    "tag",
    "team",
    "upgrade",
    "upgrade-interactive",
    "version",
    "versions",
    "why",
    "workspace",
    "workspaces",
];

/// The executors `convert` recognizes at the start of a command, in the order it tries them.
pub(crate) const EXECUTORS: [&str; 5] = ["npx", "yarn dlx", "pnpm dlx", "bun x", "nubx"];

/// What npm-to-yarn appends to a command it leaves as npm's.
pub(crate) const UNCONVERTED: &str = "\n# couldn't auto-convert command";

/// npm's `version` arguments as yarn's flags.
pub(crate) const VERSION_FLAGS: [(&str, &str); 3] = [
    ("major", "--major"),
    ("minor", "--minor"),
    ("patch", "--patch"),
];

/// npm's install flags as yarn's (`convertInstallArgs`); an empty one is dropped.
pub(crate) const YARN_INSTALL_FLAGS: [(&str, &str); 13] = [
    ("--save-dev", "--dev"),
    ("-D", "--dev"),
    ("--save-prod", "--production"),
    ("-P", "--production"),
    ("--no-package-lock", "--no-lockfile"),
    ("--save-optional", "--optional"),
    ("-O", "--optional"),
    ("--save-exact", "--exact"),
    ("-E", "--exact"),
    ("--save", ""),
    ("-S", ""),
    ("--global", ""),
    ("-g", ""),
];

/// npm's install flags as pnpm's (`convertPnpmInstallArgs`).
pub(crate) const PNPM_INSTALL_FLAGS: [(&str, &str); 3] = [
    ("--save", ""),
    ("-S", ""),
    ("--no-package-lock", "--frozen-lockfile"),
];

/// npm's `ls` flags as pnpm's.
pub(crate) const PNPM_LIST_FLAGS: [(&str, &str); 2] =
    [("--production", "--prod"), ("--development", "--dev")];

/// npm's install flags as bun's.
pub(crate) const BUN_INSTALL_FLAGS: [(&str, &str); 14] = [
    ("--save-dev", "--dev"),
    ("--development", "--dev"),
    ("-D", "--dev"),
    ("--save-prod", "--production"),
    ("-P", "--production"),
    ("--no-package-lock", "--no-save"),
    ("--save-optional", "--optional"),
    ("-O", "--optional"),
    ("--save-exact", "--exact"),
    ("-E", "--exact"),
    ("--save", ""),
    ("-S", ""),
    ("--global", "--global"),
    ("-g", "--global"),
];

/// npm's install flags as `deno add`'s.
pub(crate) const DENO_ADD_FLAGS: [(&str, &str); 11] = [
    ("--save-dev", "--dev"),
    ("--development", "--dev"),
    ("-D", "--dev"),
    ("--save-exact", "--save-exact"),
    ("-E", "--save-exact"),
    ("--save-optional", "--save-optional"),
    ("-O", "--save-optional"),
    ("--save", ""),
    ("-S", ""),
    ("--save-prod", ""),
    ("-P", ""),
];

/// npm's install flags as nub's.
pub(crate) const NUB_INSTALL_FLAGS: [(&str, &str); 4] =
    [("--save", ""), ("-S", ""), ("--save-prod", ""), ("-P", "")];

/// What a Python manager writes for one operation, in `PipManager` order: pip, uv, poetry,
/// pdm. pip, the source syntax, has an entry only where a line is written for another
/// manager (`uvx`).
pub(crate) type PipEntry = [Option<&'static str>; 4];

/// `pip install <packages>`.
pub(crate) const PIP_ADD: PipEntry =
    [None, Some("uv add"), Some("poetry add"), Some("pdm add")];
/// `pip install -r <file>`.
pub(crate) const PIP_REQUIREMENTS: PipEntry = [None, Some("uv add"), None, None];
/// `pip install .` or `pip install -e .`: the project itself.
pub(crate) const PIP_PROJECT: PipEntry = [
    None,
    Some("uv sync"),
    Some("poetry install"),
    Some("pdm install"),
];
/// `pip install --group <group>` with no packages, followed by the group.
pub(crate) const PIP_GROUP_INSTALL: PipEntry = [
    None,
    Some("uv sync --group"),
    Some("poetry install --with"),
    Some("pdm install -G"),
];
/// `pip uninstall <packages>`.
pub(crate) const PIP_REMOVE: PipEntry = [
    None,
    Some("uv remove"),
    Some("poetry remove"),
    Some("pdm remove"),
];
/// `pip list`.
pub(crate) const PIP_LIST: PipEntry = [
    None,
    Some("uv pip list"),
    Some("poetry show"),
    Some("pdm list"),
];
/// `pip show <package>`.
pub(crate) const PIP_SHOW: PipEntry = [
    None,
    Some("uv pip show"),
    Some("poetry show"),
    Some("pdm show"),
];
/// `pip freeze`.
pub(crate) const PIP_FREEZE: PipEntry = [None, Some("uv pip freeze"), None, None];
/// `python …`, run inside the project's environment.
pub(crate) const PIP_RUN: PipEntry =
    [None, Some("uv run"), Some("poetry run"), Some("pdm run")];
/// `pipx run <tool>` or `uvx <tool>`.
pub(crate) const TOOL_RUN: PipEntry = [Some("pipx run"), Some("uvx"), None, None];
/// `pipx install <tool>` or `uv tool install <tool>`.
pub(crate) const TOOL_INSTALL: PipEntry =
    [Some("pipx install"), Some("uv tool install"), None, None];
/// `pipx uninstall <tool>` or `uv tool uninstall <tool>`.
pub(crate) const TOOL_UNINSTALL: PipEntry = [
    Some("pipx uninstall"),
    Some("uv tool uninstall"),
    None,
    None,
];

/// `--group dev` on an install with packages.
pub(crate) const PIP_DEV_FLAG: PipEntry =
    [None, Some("--dev"), Some("--group dev"), Some("--dev")];
/// `--group <group>` on an install with packages, followed by the group.
pub(crate) const PIP_GROUP_FLAG: PipEntry = [
    None,
    Some("--group"),
    Some("--group"),
    Some("--dev --group"),
];
/// `-e`/`--editable`, followed by the path; pdm takes editables in dev groups only.
pub(crate) const PIP_EDITABLE_FLAG: PipEntry =
    [None, Some("--editable"), Some("--editable"), None];
/// `-U`/`--upgrade`.
pub(crate) const PIP_UPGRADE_FLAG: PipEntry = [None, Some("--upgrade"), None, None];
/// `--pre`.
pub(crate) const PIP_PRE_FLAG: PipEntry = [
    None,
    Some("--prerelease allow"),
    Some("--allow-prereleases"),
    Some("--pre"),
];

/// `cargo install` flags taking a value, as cargo-binstall's.
pub(crate) const BINSTALL_VALUE_FLAGS: [(&str, &str); 6] = [
    ("--version", "--version"),
    ("--git", "--git"),
    ("--root", "--root"),
    ("--index", "--index"),
    ("--registry", "--registry"),
    ("--target", "--targets"),
];

/// `cargo install` switches, as cargo-binstall's.
pub(crate) const BINSTALL_SWITCHES: [(&str, &str); 3] = [
    ("--locked", "--locked"),
    ("--force", "--force"),
    ("-f", "--force"),
];
