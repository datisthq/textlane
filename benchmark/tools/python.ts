import { execFile } from "node:child_process"
import { basename, join } from "node:path"
import { promisify } from "node:util"
import type { Tool } from "./types.ts"

export const PYTHON_ADAPTERS = join(import.meta.dirname, "..", "adapters", "python")
export const PYTHON = join(PYTHON_ADAPTERS, ".venv", "bin", "python")

export const markdownItPyTool = createPythonTool({
  name: "markdown-it-py",
  title: "markdown-it-py",
  adapter: "convert_markdown_it.py",
  distribution: "markdown-it-py",
  caveats: ["renders from a token stream, building no syntax tree"],
})

export const mistuneTool = createPythonTool({
  name: "mistune",
  title: "mistune",
  adapter: "convert_mistune.py",
  distribution: "mistune",
  caveats: ["not CommonMark compliant"],
})

export const pythonMarkdownTool = createPythonTool({
  name: "python-markdown",
  title: "Python-Markdown",
  adapter: "convert_markdown.py",
  distribution: "markdown",
  caveats: ["implements Markdown.pl's syntax, not CommonMark"],
})

export const cmarkTool = createPythonTool({
  name: "cmark",
  title: "cmark (C, via cmarkgfm)",
  adapter: "convert_cmarkgfm.py",
  distribution: "cmarkgfm",
  caveats: ["the C reference implementation, called through a CFFI binding"],
})

export function createPythonTool(props: {
  name: string
  title: string
  adapter: string
  distribution: string
  caveats: string[]
}): Tool {
  return {
    name: props.name,
    title: props.title,
    language: "python",
    caveats: props.caveats,
    getVersion: async () => {
      const { stdout } = await promisify(execFile)(PYTHON, [
        "-c",
        `import importlib.metadata; print(importlib.metadata.version("${props.distribution}"))`,
      ])
      return stdout.trim()
    },
    createCommand: async (dataset, repeat, flavor) => ({
      file: PYTHON,
      args: [
        join(PYTHON_ADAPTERS, props.adapter),
        basename(dataset.path),
        String(repeat),
        flavor,
      ],
    }),
  }
}
