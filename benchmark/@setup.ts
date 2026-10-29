import { execFileSync } from "node:child_process"
import { readdirSync, rmSync } from "node:fs"
import { join } from "node:path"
import { PYTHON, PYTHON_ADAPTERS } from "./tools/python.ts"
import { RUST_ADAPTERS } from "./tools/rust.ts"
import { ADDON_PATH, buildAddon } from "./tools/textlane.ts"

const ROOT = join(import.meta.dirname, "..")
const WHEELS = join(import.meta.dirname, ".cache", "wheel")

execFileSync("uv", ["sync", "--frozen"], { cwd: PYTHON_ADAPTERS, stdio: "inherit" })

console.log("Building the textlane addon")
await buildAddon()
console.log(ADDON_PATH)

rmSync(WHEELS, { recursive: true, force: true })
const python = join(ROOT, "python")
execFileSync("bash", ["scripts/copy-skills.sh"], { cwd: python, stdio: "inherit" })
execFileSync(
  "uvx",
  ["maturin@1.15.0", "build", "--release", "--out", WHEELS],
  // outside rust/, its rust-toolchain.toml does not apply: pin the same channel
  { cwd: python, stdio: "inherit", env: { ...process.env, RUSTUP_TOOLCHAIN: "stable" } },
)
const wheel = readdirSync(WHEELS).find(file => file.endsWith(".whl"))
if (!wheel) throw new Error(`maturin wrote no wheel to ${WHEELS}`)
// installed after the sync, which would remove it as not in the lock
execFileSync(
  "uv",
  ["pip", "install", "--python", PYTHON, "--reinstall", join(WHEELS, wheel)],
  { stdio: "inherit" },
)

execFileSync("cargo", ["build", "--release"], { cwd: RUST_ADAPTERS, stdio: "inherit" })
