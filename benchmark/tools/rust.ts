import { execFile } from "node:child_process"
import { readFile } from "node:fs/promises"
import { basename, join } from "node:path"
import { promisify } from "node:util"
import type { Tool } from "./types.ts"

export const RUST_ADAPTERS = join(import.meta.dirname, "..", "adapters", "rust")

export const comrakTool = createRustTool({
  name: "comrak",
  title: "comrak",
  binary: "comrak",
  distribution: "comrak",
  caveats: [],
})

export const pulldownCmarkTool = createRustTool({
  name: "pulldown-cmark",
  title: "pulldown-cmark",
  binary: "pulldown-cmark",
  distribution: "pulldown-cmark",
  caveats: [
    "renders from a pull parser's events, building no syntax tree",
    "no GFM autolink literals",
  ],
})

export const markdownRsTool = createRustTool({
  name: "markdown-rs",
  title: "markdown-rs",
  binary: "markdown-rs",
  distribution: "markdown",
  caveats: ["compiles straight to HTML, building no syntax tree"],
})

export function createRustTool(props: {
  name: string
  title: string
  binary: string
  distribution?: string
  caveats?: string[]
}): Tool {
  return {
    name: props.name,
    title: props.title,
    language: "rust",
    caveats: props.caveats ?? [],
    getVersion: async () => {
      const lock = await readFile(join(RUST_ADAPTERS, "Cargo.lock"), "utf8")
      const pattern = new RegExp(`name = "${props.distribution}"\\nversion = "([^"]+)"`)
      return pattern.exec(lock)?.[1] ?? "unknown"
    },
    // cargo is a no-op when the build is fresh, and runs before the clock starts
    createCommand: async (dataset, repeat) => {
      await promisify(execFile)("cargo", ["build", "--release", "--bin", props.binary], {
        cwd: RUST_ADAPTERS,
        maxBuffer: 64 * 1024 * 1024,
      })
      return {
        file: join(RUST_ADAPTERS, "target", "release", props.binary),
        args: [basename(dataset.path), String(repeat)],
      }
    },
  }
}
