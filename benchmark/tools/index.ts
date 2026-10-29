import { markdownItTool, markedTool, micromarkTool, unifiedTool } from "./javascript.ts"
import { cmarkTool, markdownItPyTool, mistuneTool, pythonMarkdownTool } from "./python.ts"
import { comrakTool, markdownRsTool, pulldownCmarkTool } from "./rust.ts"
import {
  textlanePythonTool,
  textlaneRustTool,
  textlaneTypescriptTool,
} from "./textlane.ts"
import type { Tool } from "./types.ts"

export const TOOLS: Tool[] = [
  textlaneRustTool,
  textlaneTypescriptTool,
  textlanePythonTool,
  comrakTool,
  pulldownCmarkTool,
  markdownRsTool,
  unifiedTool,
  micromarkTool,
  markdownItTool,
  markedTool,
  markdownItPyTool,
  mistuneTool,
  pythonMarkdownTool,
  cmarkTool,
]

export function selectTools(names?: string[]) {
  if (!names?.length) return TOOLS
  const unknown = names.filter(name => !TOOLS.some(tool => tool.name === name))
  if (unknown.length) {
    const known = TOOLS.map(tool => tool.name).join(", ")
    throw new Error(`Unknown tools ${unknown.join(", ")}; known: ${known}`)
  }
  return TOOLS.filter(tool => names.includes(tool.name))
}
