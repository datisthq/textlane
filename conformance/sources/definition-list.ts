import { readFileSync } from "node:fs"
import { stripTypeScriptTypes } from "node:module"
import { join, resolve } from "node:path"
import { parse } from "acorn"
import { simple } from "acorn-walk"
import { writeGroups } from "../actions/group/write.ts"
import { normalizeHtml } from "../actions/html/normalize.ts"
import { createPipeline, renderMarkdown } from "../actions/markdown/render.ts"
import type { RenderOptions } from "../actions/markdown/render.ts"
import { evaluateNode, isObject } from "../actions/node/evaluate.ts"
import { fetchSource } from "../actions/source/fetch.ts"

const OUTPUT = join(
  resolve(import.meta.dirname, "..", "tests", "md"),
  "definition-list.json",
)
const MICROMARK = {
  url: "https://github.com/wataru-chocola/micromark-extension-definition-list",
  commit: "7a9a3841bce369f4d3dfb401f27106daac2bad45",
}

// Each test file's case arrays and the variants upstream runs them in
const FILES: { path: string; variants: Variant[] }[] = [
  {
    path: "dev/coreTestcases.ts",
    variants: ["plain", "dangerous HTML allowed", "with GFM"],
  },
  { path: "dev/index.test.ts", variants: ["dangerous HTML allowed"] },
  { path: "dev/index.gfm-table.test.ts", variants: ["with GFM"] },
  { path: "dev/index.non-compliant.test.ts", variants: ["plain"] },
]
const VARIANTS = {
  plain: { allowDangerousHtml: false, definitionList: true },
  "dangerous HTML allowed": { allowDangerousHtml: true, definitionList: true },
  "with GFM": { allowDangerousHtml: false, gfm: {}, definitionList: true },
} satisfies Record<string, RenderOptions>
type Variant = keyof typeof VARIANTS

importDefinitionList()

function importDefinitionList() {
  const root = fetchSource({ repository: MICROMARK.url, commit: MICROMARK.commit })
  const seen = new Set<string>()
  const groups = new Map<Variant, object[]>()
  let skipped = 0
  for (const file of FILES) {
    for (const found of readCases(readFileSync(join(root, file.path), "utf8"))) {
      for (const variant of file.variants) {
        const key = `${variant}\u0000${found.markdown}`
        if (seen.has(key)) {
          skipped += 1
          continue
        }
        seen.add(key)
        const actual = renderMarkdown({ ...VARIANTS[variant], markdown: found.markdown })
        const differs = normalizeHtml(actual) !== normalizeHtml(found.html)
        const test = {
          description: found.title,
          ...(differs ? { comment: `micromark renders: ${found.html}` } : {}),
          input: found.markdown,
          expected: { html: actual },
        }
        groups.set(variant, [...(groups.get(variant) ?? []), test])
      }
    }
  }
  const source = {
    repository: MICROMARK.url,
    commit: MICROMARK.commit,
    path: "dev",
    license: "MIT",
  }
  writeGroups({
    path: OUTPUT,
    repository: MICROMARK.url,
    groups: [...groups].map(([variant, tests]) => ({
      description: `micromark-extension-definition-list${variant === "plain" ? "" : ` ${variant}`}`,
      source,
      pipeline: createPipeline(VARIANTS[variant]),
      tests,
    })),
  })
  const written = [...groups.values()].reduce((total, tests) => total + tests.length, 0)
  console.log(`definition-list: ${written} cases`)
  console.log(`  skipped ${skipped}: duplicate of an earlier case`)
}

// Every `{title, markdown, html}` object in the file, its template strings dedented as the
// tests' `dedent()` does before running them
function readCases(source: string) {
  const program = parse(stripTypeScriptTypes(source), {
    ecmaVersion: "latest",
    sourceType: "module",
  })
  const cases: { title: string; markdown: string; html: string }[] = []
  simple(program, {
    ObjectExpression(node) {
      const value = evaluateNode(node, new Map())
      if (
        !isObject(value) ||
        !("title" in value) ||
        !("markdown" in value) ||
        !("html" in value)
      )
        return
      const { title, markdown, html } = value
      if (
        typeof title !== "string" ||
        typeof markdown !== "string" ||
        typeof html !== "string"
      )
        return
      cases.push({ title, markdown: dedent(markdown), html: dedent(html) })
    },
  })
  return cases
}

// ts-dedent: drop a blank first and last line, then the indentation the remaining lines share
function dedent(text: string) {
  const lines = text.split("\n")
  if (lines[0]?.trim() === "") lines.shift()
  if (lines.at(-1)?.trim() === "") lines.pop()
  const indents = lines
    .filter(line => line.trim() !== "")
    .map(line => line.length - line.trimStart().length)
  const indent = Math.min(...indents)
  return lines.map(line => line.slice(indent)).join("\n")
}
