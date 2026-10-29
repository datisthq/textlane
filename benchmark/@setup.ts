import { execFileSync } from "node:child_process"
import { readdirSync, rmSync } from "node:fs"
import { join } from "node:path"
import { PYTHON, PYTHON_ADAPTERS } from "./tools/python.ts"
import { RUST_ADAPTERS } from "./tools/rust.ts"
import { ADDON_PATH, buildAddon } from "./tools/textlane.ts"

const ROOT = join(import.meta.dirname, "..")
const WHEELS = join(import.meta.dirname, ".cache", "wheel")

execFileSync("uv", ["sync", "--frozen"], { cwd: PYTHON_ADAPTERS, stdio: "inherit" })

// the bindings expose no pipeline yet, so textlane's adapters use the `testing` builds
console.log("Building the textlane addon with `testing`")
await buildAddon()
console.log(ADDON_PATH)

rmSync(WHEELS, { recursive: true, force: true })
const python = join(ROOT, "python")
execFileSync("bash", ["scripts/copy-skills.sh"], { cwd: python, stdio: "inherit" })
execFileSync(
  "uvx",
  [
    "maturin@1.15.0",
    "build",
    "--release",
    "--features",
    "python,testing",
    "--out",
    WHEELS,
  ],
  { cwd: python, stdio: "inherit" },
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
