import { readdirSync, readFileSync } from "node:fs"
import { basename, join, resolve } from "node:path"
import { parse } from "acorn"
import type { AnyNode, CallExpression } from "acorn"
import { ancestor } from "acorn-walk"
import { toHast } from "mdast-util-to-hast"
import { writeGroups } from "../actions/group/write.ts"
import { parseMarkdown } from "../actions/markdown/render.ts"
import { evaluateNode, isObject } from "../actions/node/evaluate.ts"
import { resolveConstants } from "../actions/node/scope.ts"
import { fetchSource } from "../actions/source/fetch.ts"
import { isAssertEqual } from "../actions/test/assert.ts"
import { findTestTitle } from "../actions/test/find.ts"

const OUTPUT = resolve(import.meta.dirname, "..", "tests", "md")
const REPOSITORY = {
  url: "https://github.com/syntax-tree/mdast-util-to-hast",
  commit: "174795b21f7757fffb54dd8d5fb4012f4751f791",
}

// One test file per node type, onto the construct files that node belongs to
const CONSTRUCTS: Record<string, string> = {
  blockquote: "block-quote",
  break: "hard-break",
  code: "code-fenced",
  core: "document",
  definition: "definition",
  delete: "gfm-strikethrough",
  emphasis: "emphasis",
  "footnote-definition": "gfm-footnote",
  footnote: "gfm-footnote",
  "footnote-reference": "gfm-footnote",
  heading: "heading-atx",
  html: "html-flow",
  image: "image",
  "image-reference": "image",
  "inline-code": "code-text",
  link: "link",
  "link-reference": "link",
  "list-item": "list-item",
  list: "list",
  paragraph: "paragraph",
  root: "document",
  strong: "emphasis",
  table: "gfm-table",
  text: "text",
  "thematic-break": "thematic-break",
  toml: "frontmatter",
  yaml: "frontmatter",
}

type Group = {
  description: string
  source: { repository: string; commit: string; path: string; license: string }
  pipeline: unknown[]
  tests: object[]
}

const skipped = new Map<string, number>()
importToHast()

// The expected hast is rendered by the pinned mdast-util-to-hast, never by running upstream's
// `h()` expectations
function importToHast() {
  const root = join(
    fetchSource({ repository: REPOSITORY.url, commit: REPOSITORY.commit }),
    "test",
  )
  const byConstruct = new Map<string, Map<string, Group>>()
  let written = 0
  for (const file of readdirSync(root)
    .filter(name => name.endsWith(".js") && name !== "index.js")
    .sort()) {
    const name = basename(file, ".js")
    const construct = CONSTRUCTS[name]
    if (!construct) {
      countSkip(`a test file without a construct (${name})`)
      continue
    }
    for (const found of extractCases(readFileSync(join(root, file), "utf8"))) {
      const optionsKey =
        Object.keys(found.options).length > 0 ? ` ${JSON.stringify(found.options)}` : ""
      const kind = "input" in found.test ? " from Markdown" : ""
      const description = `mdast-util-to-hast ${name}${kind}${optionsKey}`
      const groups = byConstruct.get(construct) ?? new Map<string, Group>()
      byConstruct.set(construct, groups)
      const toHtml =
        Object.keys(found.options).length > 0
          ? ["md.toHtml", found.options]
          : ["md.toHtml"]
      const pipeline =
        "input" in found.test
          ? [["md.parse"], ...(found.gfm ? [["md.gfm"]] : []), toHtml]
          : [toHtml]
      const group = groups.get(description) ?? {
        description,
        source: {
          repository: REPOSITORY.url,
          commit: REPOSITORY.commit,
          path: `test/${file}`,
          license: "MIT",
        },
        pipeline,
        tests: [],
      }
      group.tests.push(found.test)
      groups.set(description, group)
      written += 1
    }
  }
  for (const [construct, groups] of byConstruct) {
    writeGroups({
      path: join(OUTPUT, `${construct}.json`),
      repository: REPOSITORY.url,
      groups: [...groups.values()],
    })
  }
  console.log(`to-hast: ${written} cases in ${byConstruct.size} files`)
  for (const [reason, count] of skipped) console.log(`  skipped ${count}: ${reason}`)
}

// `assert.deepEqual(toHast(tree | fromMarkdown(md, gfm), options?), …)`
function extractCases(source: string) {
  const program = parse(source, { ecmaVersion: "latest", sourceType: "module" })
  const cases: { options: Record<string, unknown>; gfm: boolean; test: object }[] = []
  const counts = new Map<string, number>()
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      if (!isAssertEqual(node.callee)) return
      const call = node.arguments[0]
      if (call?.type !== "CallExpression" || !isCallTo(call, "toHast")) return
      const constants = resolveConstants({ program, ancestors })
      const options = call.arguments[1] ? evaluateNode(call.arguments[1], constants) : {}
      if (!isObject(options))
        return countSkip("options not statically evaluable (handlers, functions)")
      const title = findTestTitle(ancestors, constants) ?? "mdast-util-to-hast"
      const count = (counts.get(title) ?? 0) + 1
      counts.set(title, count)
      const description = count === 1 ? title : `${title} (${count})`
      const subject = call.arguments[0]
      const parsed =
        subject?.type === "Identifier" ? constants.get(subject.name) : subject
      if (parsed?.type === "CallExpression" && isCallTo(parsed, "fromMarkdown")) {
        const input = evaluateNode(parsed.arguments[0], constants)
        if (typeof input !== "string") return countSkip("not statically evaluable")
        const gfm = usesGfm(parsed.arguments[1])
        const tree = render(parseMarkdown(input, gfm ? { gfm: {} } : {}), options)
        cases.push({
          options: { ...options },
          gfm,
          test: { description, input, expected: { tree } },
        })
        return
      }
      const tree = evaluateNode(subject, constants)
      if (!isObject(tree)) return countSkip("a tree not statically evaluable")
      cases.push({
        options: { ...options },
        gfm: false,
        test: { description, tree, expected: { tree: render(tree, options) } },
      })
    },
  })
  return cases
}

function render(tree: object, options: object) {
  // A JSON tree is valid mdast; to-hast types its nodes, not plain objects
  return JSON.parse(JSON.stringify(toHast(JSON.parse(JSON.stringify(tree)), options)))
}

function usesGfm(node: AnyNode | undefined) {
  if (node?.type !== "ObjectExpression") return false
  return node.properties.some(
    property =>
      property.type === "Property" &&
      property.value.type === "ArrayExpression" &&
      property.value.elements.some(
        element =>
          element?.type === "CallExpression" &&
          element.callee.type === "Identifier" &&
          element.callee.name.startsWith("gfm"),
      ),
  )
}

function isCallTo(call: CallExpression, name: string) {
  return call.callee.type === "Identifier" && call.callee.name === name
}

function countSkip(reason: string) {
  skipped.set(reason, (skipped.get(reason) ?? 0) + 1)
  return undefined
}
