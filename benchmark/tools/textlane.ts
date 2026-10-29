import { execFile } from "node:child_process"
import { copyFile, mkdir, readFile } from "node:fs/promises"
import { join } from "node:path"
import { promisify } from "node:util"
import { createJavascriptTool } from "./javascript.ts"
import { createPythonTool } from "./python.ts"
import { createRustTool } from "./rust.ts"
import type { Tool } from "./types.ts"

const ROOT = join(import.meta.dirname, "..", "..")
export const ADDON_PATH = join(
  import.meta.dirname,
  "..",
  ".cache",
  "addon",
  "textlane.node",
)
const RUN_CASE_CAVEAT =
  "drives the `testing` build's case runner, as the binding has no pipeline yet: each document " +
  "pays a JSON round trip and a pipeline rebuild"

export const textlaneRustTool: Tool = {
  ...createRustTool({
    name: "textlane-rust",
    title: "textlane (Rust)",
    binary: "textlane",
  }),
  isTextlane: true,
  getVersion: readTextlaneVersion,
}

const typescriptAdapter = createJavascriptTool({
  name: "textlane-typescript",
  title: "textlane (Node.js)",
  adapter: "textlane.ts",
  caveats: [RUN_CASE_CAVEAT],
})

export const textlaneTypescriptTool: Tool = {
  ...typescriptAdapter,
  isTextlane: true,
  getVersion: readTextlaneVersion,
  // cargo is a no-op when the addon is fresh, and runs before the clock starts
  createCommand: async (dataset, repeat) => {
    await buildAddon()
    const command = await typescriptAdapter.createCommand(dataset, repeat)
    return { ...command, env: { TEXTLANE_ADDON: ADDON_PATH } }
  },
}

export const textlanePythonTool: Tool = {
  ...createPythonTool({
    name: "textlane-python",
    title: "textlane (Python)",
    adapter: "convert_textlane.py",
    distribution: "textlane",
    caveats: [RUN_CASE_CAVEAT],
  }),
  isTextlane: true,
  getVersion: readTextlaneVersion,
}

const LIBRARY_NAMES: Record<string, string> = {
  darwin: "libtextlane.dylib",
  linux: "libtextlane.so",
}

/** The napi addon with the `testing` feature, the only build exposing `runCase`. */
export async function buildAddon() {
  const rust = join(ROOT, "rust")
  await promisify(execFile)(
    "cargo",
    ["build", "--release", "-p", "textlane", "--features", "node,testing"],
    { cwd: rust, maxBuffer: 64 * 1024 * 1024 },
  )
  const library = LIBRARY_NAMES[process.platform]
  if (!library) throw new Error(`No addon build for ${process.platform}`)
  await mkdir(join(ADDON_PATH, ".."), { recursive: true })
  await copyFile(join(rust, "target", "release", library), ADDON_PATH)
}

async function readTextlaneVersion() {
  const version = (await readFile(join(ROOT, "version.txt"), "utf8")).trim()
  const { stdout } = await promisify(execFile)("git", ["rev-parse", "--short", "HEAD"], {
    cwd: ROOT,
  })
  return `${version}+${stdout.trim()}`
}
