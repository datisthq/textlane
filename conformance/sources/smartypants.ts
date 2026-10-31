import { readFileSync } from "node:fs"
import { stripTypeScriptTypes } from "node:module"
import { join, resolve } from "node:path"
import { parse } from "acorn"
import type { AnyNode } from "acorn"
import { ancestor } from "acorn-walk"
import rehypeStringify from "rehype-stringify"
import remarkGfm from "remark-gfm"
import remarkMdx from "remark-mdx"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import remarkSmartypants from "remark-smartypants"
import remarkStringify from "remark-stringify"
import { unified } from "unified"
import { writeGroups } from "../actions/group/write.ts"
import { evaluateNode } from "../actions/node/evaluate.ts"
import { resolveConstants } from "../actions/node/scope.ts"
import { fetchSource } from "../actions/source/fetch.ts"
import { findTestTitle } from "../actions/test/find.ts"

const OUTPUT = join(resolve(import.meta.dirname, "..", "tests", "md"), "smartypants.json")
const SMARTYPANTS = {
  url: "https://github.com/silvenon/remark-smartypants",
  commit: "a7a30398af761cafce6b1b2372f7eaf428db3908",
}
type Syntax = "md" | "gfm" | "mdx"
const STAGES: Record<Syntax, string[][]> = {
  md: [],
  gfm: [["md.gfm"]],
  mdx: [["md.mdx"]],
}
// textlane's own inputs: the joined text is spliced back by UTF-16 offsets, so astral
// characters, code holding them and options that change what the passes touch
const OWN: [string, Record<string, unknown>][] = [
  ['😀 "emoji" -- then... done', {}],
  ['`😀` "after code" and `x`', {}],
  ["「引用」 \"中文\" — '混合'", {}],
  ["\"*emphasis* inside\" and '**strong** too'", {}],
  ["Wait... what.... -- --- ``this''", { dashes: "oldschool", ellipses: "spaced" }],
  ['`single\' and "double"', { backticks: "all", quotes: false }],
  [
    "\"quoted\" 'single'",
    {
      openingQuotes: { double: "«", single: "‹" },
      closingQuotes: { double: "»", single: "›" },
    },
  ],
  [
    '"quoted" and more "words"',
    {
      openingQuotes: { double: "<<", single: "<" },
      closingQuotes: { double: ">>", single: ">" },
    },
  ],
  ['- "item" one\n- \'item\' two\n\n> "quote"', {}],
]

const groups = new Map<
  string,
  {
    syntax: Syntax
    options: Record<string, unknown>
    output: "md" | "html"
    tests: object[]
  }
>()
let written = 0
for (const { title, input, syntax } of readTests()) {
  addCase(title, input, syntax, {}, "md")
  // MDX elements have no plain HTML: `md.toHtml` keeps them for `mdx.compile`
  if (syntax !== "mdx") addCase(title, input, syntax, {}, "html")
}
for (const [input, options] of OWN) {
  addCase(JSON.stringify(input), input, "gfm", options, "md")
  addCase(JSON.stringify(input), input, "gfm", options, "html")
}
writeGroups({
  path: OUTPUT,
  repository: SMARTYPANTS.url,
  groups: [...groups].map(([key, group]) => ({
    description: `remark-smartypants ${key}`,
    source: {
      repository: SMARTYPANTS.url,
      commit: SMARTYPANTS.commit,
      path: "plugin.test.ts",
      license: "MIT",
    },
    pipeline: [
      ["md.parse"],
      ...STAGES[group.syntax],
      Object.keys(group.options).length > 0
        ? ["md.smartypants", group.options]
        : ["md.smartypants"],
      ...(group.output === "md"
        ? [["md.stringify"]]
        : [["md.toHtml"], ["html.stringify"]]),
    ],
    tests: group.tests,
  })),
})
console.log(`smartypants: ${written} cases`)

// Each case through the pinned packages, as Markdown and as HTML, so the text spliced back
// is held apart from how the serializer escapes it
function addCase(
  title: string,
  input: string,
  syntax: Syntax,
  options: Record<string, unknown>,
  output: "md" | "html",
) {
  const extensions = syntax === "gfm" ? [remarkGfm] : syntax === "mdx" ? [remarkMdx] : []
  const processor = unified()
    .use(remarkParse)
    .use(extensions)
    .use(remarkSmartypants, options)
  const value =
    output === "md"
      ? String(processor.use(remarkStringify).processSync(input))
      : String(processor.use(remarkRehype).use(rehypeStringify).processSync(input))
  const key = `${syntax} ${output}${Object.keys(options).length > 0 ? ` ${JSON.stringify(options)}` : ""}`
  const group = groups.get(key) ?? { syntax, options, output, tests: [] }
  group.tests.push({ description: title, input, expected: { [output]: value } })
  groups.set(key, group)
  written += 1
}

// `process(input)` inside each `it`, with the compiler of its `describe`: plain remark, or
// remark with remark-gfm or remark-mdx
function readTests() {
  const root = fetchSource({ repository: SMARTYPANTS.url, commit: SMARTYPANTS.commit })
  const source = stripTypeScriptTypes(readFileSync(join(root, "plugin.test.ts"), "utf8"))
  const program = parse(source, { ecmaVersion: "latest", sourceType: "module" })
  const tests: { title: string; input: string; syntax: Syntax }[] = []
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      if (node.callee.type !== "Identifier" || node.callee.name !== "process") return
      const constants = resolveConstants({ program, ancestors })
      const input = evaluateNode(node.arguments[0], constants)
      if (typeof input !== "string") return
      tests.push({
        title: findTestTitle(ancestors, constants) ?? JSON.stringify(input),
        input,
        syntax: readSyntax(source, ancestors),
      })
    },
  })
  return tests
}

function readSyntax(source: string, ancestors: AnyNode[]): Syntax {
  const describe = ancestors.find(
    node =>
      node.type === "CallExpression" &&
      node.callee.type === "Identifier" &&
      node.callee.name === "describe",
  )
  const text = describe ? source.slice(describe.start, describe.end) : ""
  if (text.includes("use(remarkGfm)")) return "gfm"
  if (text.includes("use(remarkMdx)")) return "mdx"
  return "md"
}
