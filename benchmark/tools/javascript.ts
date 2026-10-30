import { readFile } from "node:fs/promises"
import { basename, join } from "node:path"
import type { Tool } from "./types.ts"

const ADAPTERS = join(import.meta.dirname, "..", "adapters", "javascript")
const MODULES = join(import.meta.dirname, "..", "node_modules")

export const unifiedTool = createJavascriptTool({
  name: "unified",
  title: "unified (remark + rehype)",
  adapter: "unified.ts",
  distribution: "unified",
  caveats: [],
})

export const micromarkTool = createJavascriptTool({
  name: "micromark",
  title: "micromark",
  adapter: "micromark.ts",
  distribution: "micromark",
  caveats: ["compiles straight to HTML, building no syntax tree"],
})

export const markdownItTool = createJavascriptTool({
  name: "markdown-it",
  title: "markdown-it",
  adapter: "markdown-it.ts",
  distribution: "markdown-it",
  caveats: ["renders from a token stream, building no syntax tree"],
})

export const markedTool = createJavascriptTool({
  name: "marked",
  title: "marked",
  adapter: "marked.ts",
  distribution: "marked",
  caveats: ["not CommonMark compliant"],
})

export function createJavascriptTool(props: {
  name: string
  title: string
  adapter: string
  distribution?: string
  caveats: string[]
}): Tool {
  return {
    name: props.name,
    title: props.title,
    language: "typescript",
    caveats: props.caveats,
    getVersion: async () => {
      const path = join(MODULES, props.distribution ?? "", "package.json")
      const manifest: { version: string } = JSON.parse(await readFile(path, "utf8"))
      return manifest.version
    },
    createCommand: async (dataset, repeat) => ({
      file: process.execPath,
      args: [join(ADAPTERS, props.adapter), basename(dataset.path), String(repeat)],
    }),
  }
}
