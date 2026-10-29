import { readFileSync, readdirSync } from "node:fs"
import { join } from "node:path"
import type { Profile } from "./types.ts"

const TESTS = join(import.meta.dirname, "..", "..", "conformance", "tests", "md")
const COMMONMARK = ["md.parse", "md.toHtml", "html.stringify"]
const FENCE = /^ {0,3}(`{3,}|~{3,})/gm
const UNCLOSED = [
  ["<!--", "-->"],
  ["<?", "?>"],
  ["<![CDATA[", "]]>"],
  ["<pre", "</pre>"],
  ["<script", "</script>"],
  ["<style", "</style>"],
  ["<textarea", "</textarea>"],
]

let snippets: string[] | undefined

/** The conformance suite's plain CommonMark inputs, one after another: the edge cases. */
export const specProfile: Profile = {
  name: "spec",
  description: "The CommonMark edge cases of the conformance suite, back to back",
  version: 1,
  createBlock: (_random, index) => {
    snippets ??= readSnippets()
    const snippet = snippets[index % snippets.length] ?? ""
    // a thematic break at column 0 ends any open list, quote or indented code
    return `${snippet.trimEnd()}\n\n---\n\n`
  },
}

/** Each snippet must close what it opens, or it would swallow every snippet after it. */
export function readSnippets() {
  const inputs: string[] = []
  for (const file of readdirSync(TESTS).toSorted()) {
    const groups: { pipeline: [string][]; tests: { input?: string }[] }[] = JSON.parse(
      readFileSync(join(TESTS, file), "utf8"),
    )
    for (const group of groups) {
      const names = group.pipeline.map(entry => entry[0])
      if (names.join() !== COMMONMARK.join()) continue
      for (const test of group.tests) {
        if (test.input !== undefined && isSelfContained(test.input))
          inputs.push(test.input)
      }
    }
  }
  return inputs
}

function isSelfContained(input: string) {
  if ((input.match(FENCE)?.length ?? 0) % 2) return false
  // a reference definition would apply to every later snippet's links
  if (/^ {0,3}\[[^\]]+\]:/m.test(input)) return false
  const lower = input.toLowerCase()
  return UNCLOSED.every(
    ([open = "", close = ""]) => !lower.includes(open) || lower.includes(close),
  )
}
