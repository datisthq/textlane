import { readdirSync, readFileSync } from "node:fs"
import { basename, join, relative, resolve } from "node:path"
import { parse } from "acorn"
import type { AnyNode } from "acorn"
import { ancestor } from "acorn-walk"
import { commonmark } from "commonmark.json"
import { writeGroups } from "../actions/group/write.ts"
import { evaluateNode, isObject } from "../actions/node/evaluate.ts"
import { findTestTitle } from "../actions/test/find.ts"
import { normalizeHtml } from "../actions/html/normalize.ts"
import { createPipeline, renderMarkdown } from "../actions/markdown/render.ts"
import { fetchSource } from "../actions/source/fetch.ts"

const REPOSITORY = "https://github.com/micromark/micromark"
const COMMIT = "6577c200155e9c6b85a42b26d298f8affc348f2f"
const OUTPUT = resolve(import.meta.dirname, "..", "tests", "md")

// Test files onto construct files where the file name alone would clash or split a construct
const CONSTRUCTS: Record<string, string> = {
  "flow/html.js": "html-flow",
  "text/html.js": "html-text",
  "text/code.js": "code-text",
  "text/link-resource.js": "link",
  "text/link-reference.js": "link",
}
// Input as bytes, or micromark's internal constants: nothing a pipeline case can express
const SKIPPED_FILES = new Set(["misc/typed-array.js", "misc/constants.js"])
const SUPPORTED_OPTIONS = new Set(["allowDangerousHtml", "allowDangerousProtocol"])

type Case = {
  file: string
  description: string
  markdown: string
  html: string
  options: object
}

const skipped = new Map<string, number>()
importMicromark()

function importMicromark() {
  const root = join(fetchSource({ repository: REPOSITORY, commit: COMMIT }), "test", "io")
  const seen = new Set(commonmark.map(example => caseKey(example.markdown, true)))
  const groups = new Map<string, Map<string, Group>>()
  let written = 0
  for (const file of listFiles(root)) {
    const name = relative(root, file).replace(/\\/g, "/")
    if (SKIPPED_FILES.has(name) || basename(name) === "index.js") continue
    for (const found of extractCases(name, readFileSync(file, "utf8"))) {
      if (!found.markdown.isWellFormed()) {
        countSkip("a lone surrogate, which UTF-8 input cannot hold")
        continue
      }
      const allowDangerousHtml = readHtmlOption(found)
      if (allowDangerousHtml === undefined) continue
      const key = caseKey(found.markdown, allowDangerousHtml)
      if (seen.has(key)) {
        countSkip("duplicate of an earlier case")
        continue
      }
      seen.add(key)
      const construct = CONSTRUCTS[name] ?? basename(name, ".js")
      const byGroup = groups.get(construct) ?? new Map<string, Group>()
      groups.set(construct, byGroup)
      const group = getGroup({ byGroup, name, allowDangerousHtml })
      group.tests.push(createTest({ found, allowDangerousHtml }))
      written += 1
    }
  }
  for (const [construct, byGroup] of groups) {
    const path = join(OUTPUT, `${construct}.json`)
    writeGroups({ path, repository: REPOSITORY, groups: [...byGroup.values()] })
  }
  console.log(`micromark: ${written} cases in ${groups.size} files`)
  for (const [reason, count] of skipped) console.log(`  skipped ${count}: ${reason}`)
}

type Group = {
  description: string
  source: { repository: string; commit: string; path: string; license: string }
  pipeline: unknown[]
  tests: object[]
}

function getGroup(props: {
  byGroup: Map<string, Group>
  name: string
  allowDangerousHtml: boolean
}) {
  const id = `${props.name}#${props.allowDangerousHtml}`
  const existing = props.byGroup.get(id)
  if (existing) return existing
  const suffix = props.allowDangerousHtml ? " with dangerous HTML allowed" : ""
  const group: Group = {
    description: `micromark ${props.name.replace(/\.js$/, "")}${suffix}`,
    source: {
      repository: REPOSITORY,
      commit: COMMIT,
      path: `test/io/${props.name}`,
      license: "MIT",
    },
    pipeline: createPipeline({ allowDangerousHtml: props.allowDangerousHtml }),
    tests: [],
  }
  props.byGroup.set(id, group)
  return group
}

// micromark's own HTML compiler is not textlane's path: the expectation is unified's output
function createTest(props: { found: Case; allowDangerousHtml: boolean }) {
  const markdown = props.found.markdown
  const actual = renderMarkdown({
    markdown,
    allowDangerousHtml: props.allowDangerousHtml,
  })
  const differs = normalizeHtml(actual) !== normalizeHtml(props.found.html)
  return {
    description: props.found.description,
    ...(differs ? { comment: `micromark renders: ${props.found.html}` } : {}),
    input: markdown,
    expected: { html: actual },
  }
}

// allowDangerousProtocol only changes micromark's own compiler; unified never strips URLs
function readHtmlOption(found: Case) {
  const options: Record<string, unknown> = { ...found.options }
  const unsupported = Object.keys(options).filter(key => !SUPPORTED_OPTIONS.has(key))
  if (unsupported.length > 0) {
    countSkip(
      `micromark options without a pipeline equivalent (${unsupported.join(", ")})`,
    )
    return undefined
  }
  return options.allowDangerousHtml === true
}

function extractCases(name: string, source: string) {
  const program = parse(source, { ecmaVersion: "latest", sourceType: "module" })
  const constants = new Map<string, AnyNode>()
  ancestor(program, {
    VariableDeclarator(node) {
      if (node.id.type === "Identifier" && node.init)
        constants.set(node.id.name, node.init)
    },
  })
  const cases: Case[] = []
  const counts = new Map<string, number>()
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      if (!isAssertEqual(node.callee)) return
      const [call, expected] = node.arguments
      if (call?.type !== "CallExpression" || call.callee.type !== "Identifier") return
      if (call.callee.name !== "micromark") return
      const markdown = evaluateNode(call.arguments[0], constants)
      const options = call.arguments[1] ? evaluateNode(call.arguments[1], constants) : {}
      const html = evaluateNode(expected, constants)
      if (
        typeof markdown !== "string" ||
        typeof html !== "string" ||
        !isObject(options)
      ) {
        countSkip("not statically evaluable")
        return
      }
      const title = findTestTitle(ancestors, constants) ?? name
      const count = (counts.get(title) ?? 0) + 1
      counts.set(title, count)
      const description = count === 1 ? title : `${title} (${count})`
      cases.push({ file: name, description, markdown, html, options })
    },
  })
  return cases
}

function isAssertEqual(callee: AnyNode) {
  if (callee.type !== "MemberExpression") return false
  const { object, property } = callee
  return (
    object.type === "Identifier" &&
    object.name === "assert" &&
    property.type === "Identifier" &&
    (property.name === "equal" || property.name === "strictEqual")
  )
}

function caseKey(markdown: string, allowDangerousHtml: boolean) {
  return `${allowDangerousHtml}\u0000${markdown}`
}

function countSkip(reason: string) {
  skipped.set(reason, (skipped.get(reason) ?? 0) + 1)
}

function listFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const path = join(directory, entry.name)
    if (entry.isDirectory()) return listFiles(path)
    return entry.name.endsWith(".js") ? [path] : []
  })
}
