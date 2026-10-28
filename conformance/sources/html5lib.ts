import { readdirSync, readFileSync } from "node:fs"
import { basename, join, resolve } from "node:path"
import { h, s } from "hastscript"
import rehypeStringify from "rehype-stringify"
import { unified } from "unified"
import { writeGroups } from "../actions/group/write.ts"
import { isObject } from "../actions/node/evaluate.ts"
import { fetchSource } from "../actions/source/fetch.ts"

const OUTPUT = resolve(import.meta.dirname, "..", "tests", "html")
const HTML5LIB = {
  url: "https://github.com/html5lib/html5lib-tests",
  commit: "c777c408b61078ea2eb4acefc2535f54dbc8b28a",
}
const SVG = "http://www.w3.org/2000/svg"
// html5lib's serializer defaults, as hast-util-to-html options
const DEFAULTS = { omitOptionalTags: true, preferUnquoted: true, quoteSmart: true }

type Token = [string, ...unknown[]]
type Case = {
  description: string
  input: Token[]
  expected: string[]
  options?: Record<string, unknown>
}
type Parent = { type: "root" | "element"; tagName?: string; children: Node[] }
type Node = Parent | { type: "text" | "comment"; value: string } | { type: "doctype" }

const skipped = new Map<string, number>()
importSerializer()

// Serializer: html5lib states each case as a token stream, which is rebuilt here as the hast
// tree it walks; the expected HTML is the pinned rehype-stringify's, html5lib's own in a comment
function importSerializer() {
  const root = join(
    fetchSource({ repository: HTML5LIB.url, commit: HTML5LIB.commit }),
    "serializer",
  )
  let written = 0
  for (const file of readdirSync(root)
    .filter(name => name.endsWith(".test"))
    .sort()) {
    const name = basename(file, ".test")
    const groups = new Map<string, { options: object; tests: object[] }>()
    const cases: Case[] = JSON.parse(readFileSync(join(root, file), "utf8")).tests
    for (const found of cases) {
      const options = mapOptions(found.options ?? {})
      if (typeof options === "string") {
        countSkip(options)
        continue
      }
      const tree = buildTree(found.input)
      if (typeof tree === "string") {
        countSkip(tree)
        continue
      }
      // The rebuilt tree is plain hast data; unified types roots, not this importer's node shapes
      const processor = unified().use(rehypeStringify, options)
      const html = String(
        Reflect.apply(processor.stringify, processor, [structuredClone(tree)]),
      )
      const key = JSON.stringify(options)
      const group = groups.get(key) ?? { options, tests: [] }
      group.tests.push({
        description: found.description,
        // A stream is a fragment, so the tree prints tags outside it; only a real difference is noted
        ...(found.expected.some(expected => html.includes(expected))
          ? {}
          : {
              comment: `html5lib expects ${found.expected.map(x => JSON.stringify(x)).join(" or ")}`,
            }),
        tree,
        expected: { html },
      })
      groups.set(key, group)
      written += 1
    }
    if (groups.size === 0) continue
    writeGroups({
      path: join(OUTPUT, `html5lib-${name}.json`),
      repository: HTML5LIB.url,
      groups: [...groups.values()].map(group => ({
        description: `html5lib serializer ${name} ${JSON.stringify(group.options)}`,
        comment:
          "Rebuilt from html5lib token streams; an end tag with no start wraps the siblings before it, and the reference prints the tags a stream leaves outside",
        source: {
          repository: HTML5LIB.url,
          commit: HTML5LIB.commit,
          path: `serializer/${file}`,
          license: "MIT",
        },
        pipeline: [["html.stringify", group.options]],
        tests: group.tests,
      })),
    })
  }
  console.log(`html5lib: ${written} cases`)
  for (const [reason, count] of skipped) console.log(`  skipped ${count}: ${reason}`)
}

// hast-util-to-html options for html5lib's; a string names an option it has no equivalent of
function mapOptions(options: Record<string, unknown>) {
  const mapped: Record<string, unknown> = { ...DEFAULTS }
  for (const [key, value] of Object.entries(options)) {
    if (key === "encoding") continue
    else if (key === "quote_attr_values" && value === true) delete mapped.preferUnquoted
    else if (key === "quote_char" && typeof value === "string") {
      mapped.quote = value
      delete mapped.quoteSmart
    } else if (key === "use_trailing_solidus" && value === true)
      mapped.closeSelfClosing = true
    else return `an option hast-util-to-html lacks (${key})`
  }
  return mapped
}

// A start tag opens an element and its end tag closes it; an end tag with no open element means
// the stream began inside it, so it wraps the siblings before it. Unclosed elements end the stream
function buildTree(tokens: Token[]) {
  const root: Parent = { type: "root", children: [] }
  const stack: Parent[] = [root]
  for (const token of tokens) {
    const parent = stack[stack.length - 1] ?? root
    const [kind, ...rest] = token
    if (kind === "StartTag" || kind === "EmptyTag") {
      const [namespace, name, attributes] =
        kind === "StartTag" ? rest : ["http://www.w3.org/1999/xhtml", ...rest]
      if (typeof name !== "string") return "a malformed tag token"
      const element = createElement(namespace, name, attributes)
      if (typeof element === "string") return element
      parent.children.push(element)
      if (kind === "StartTag") stack.push(element)
    } else if (kind === "EndTag") {
      const [namespace, name] = rest
      if (typeof name !== "string") return "a malformed tag token"
      const index = stack.findLastIndex(node => node.tagName === name)
      if (index > 0) {
        stack.length = index
        continue
      }
      const element = createElement(namespace, name, [])
      if (typeof element === "string") return element
      element.children = parent.children.splice(0)
      parent.children.push(element)
    } else if (kind === "Characters" || kind === "SpaceCharacters") {
      const [value] = rest
      if (typeof value !== "string") return "a malformed text token"
      const last = parent.children.at(-1)
      if (last?.type === "text") last.value += value
      else parent.children.push({ type: "text", value })
    } else if (kind === "Comment") {
      const [value] = rest
      if (typeof value !== "string") return "a malformed comment token"
      parent.children.push({ type: "comment", value })
    } else if (kind === "Doctype") {
      // hast keeps no public or system identifier, nor a doctype name other than html
      if (rest.length > 1 || String(rest[0]).toLowerCase() !== "html")
        return "a doctype hast cannot hold"
      parent.children.push({ type: "doctype" })
    } else return `an unknown token (${kind})`
  }
  return root
}

function createElement(namespace: unknown, name: string, attributes: unknown) {
  const properties: Record<string, string> = {}
  const list: unknown[] = Array.isArray(attributes) ? attributes : []
  for (const attribute of list) {
    if (!isObject(attribute)) return "a malformed attribute"
    if (Reflect.get(attribute, "namespace")) return "a namespaced attribute"
    properties[String(Reflect.get(attribute, "name"))] = String(
      Reflect.get(attribute, "value"),
    )
  }
  const element: Parent = JSON.parse(
    JSON.stringify(namespace === SVG ? s(name, properties) : h(name, properties)),
  )
  return element
}

function countSkip(reason: string) {
  skipped.set(reason, (skipped.get(reason) ?? 0) + 1)
}
