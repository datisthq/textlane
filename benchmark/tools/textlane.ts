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

export const textlaneRustTool: Tool = {
  ...createRustTool({
    name: "textlane-rust",
    title: "textlane (Rust, mdast)",
    binary: "textlane",
  }),
  isTextlane: true,
  getVersion: readTextlaneVersion,
}

const typescriptAdapter = createJavascriptTool({
  name: "textlane-typescript",
  title: "textlane (Node.js, mdast)",
  adapter: "textlane.ts",
  caveats: [],
})

export const textlaneTypescriptTool: Tool = {
  ...typescriptAdapter,
  isTextlane: true,
  getVersion: readTextlaneVersion,
  // cargo is a no-op when the addon is fresh, and runs before the clock starts
  createCommand: async (dataset, repeat, flavor) => {
    await buildAddon()
    const command = await typescriptAdapter.createCommand(dataset, repeat, flavor)
    return { ...command, env: { TEXTLANE_ADDON: ADDON_PATH } }
  },
}

export const textlanePythonTool: Tool = {
  ...createPythonTool({
    name: "textlane-python",
    title: "textlane (Python, mdast)",
    adapter: "convert_textlane.py",
    distribution: "textlane",
    caveats: [],
  }),
  isTextlane: true,
  getVersion: readTextlaneVersion,
}

export const textlaneRustHastTool = createHastTool(textlaneRustTool)
export const textlaneTypescriptHastTool = createHastTool(textlaneTypescriptTool)
export const textlanePythonHastTool = createHastTool(textlanePythonTool)

// The same build with an HTML plugin between `md.toHtml` and `html.stringify`, which keeps
// hast; a no-op stands in for it, so only the tree's cost shows
function createHastTool(tool: Tool): Tool {
  return {
    ...tool,
    name: `${tool.name}-hast`,
    title: tool.title.replace(", mdast)", ", mdast + hast)"),
    caveats: [
      "builds hast as any HTML plugin makes it, with a no-op in the plugin's place",
    ],
    createCommand: async (dataset, repeat, flavor) => {
      const command = await tool.createCommand(dataset, repeat, flavor)
      return { ...command, env: { ...command.env, TEXTLANE_HAST: "1" } }
    },
  }
}

const LIBRARY_NAMES: Record<string, string> = {
  darwin: "libtextlane.dylib",
  linux: "libtextlane.so",
}

/** The napi addon, as the platform packages ship it. */
export async function buildAddon() {
  const rust = join(ROOT, "rust")
  await promisify(execFile)(
    "cargo",
    ["build", "--release", "-p", "textlane", "--features", "node"],
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
