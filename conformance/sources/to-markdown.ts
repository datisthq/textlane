import { readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { parse } from "acorn"
import type { AnyNode, CallExpression } from "acorn"
import { ancestor } from "acorn-walk"
import { writeGroups } from "../actions/group/write.ts"
import { isStressInput } from "../actions/input/check.ts"
import { parseMarkdown, stringifyTree } from "../actions/markdown/render.ts"
import { evaluateNode, isObject } from "../actions/node/evaluate.ts"
import { resolveConstants } from "../actions/node/scope.ts"
import { fetchSource } from "../actions/source/fetch.ts"
import { isAssertEqual } from "../actions/test/assert.ts"
import { findTestTitle } from "../actions/test/find.ts"

const REPOSITORY = "https://github.com/syntax-tree/mdast-util-to-markdown"
const COMMIT = "a31fe0608ff3c68f92f79bc5acc35235d8b69cee"
const OUTPUT = resolve(import.meta.dirname, "..", "tests", "md")

// Upstream's top-level tests, one per node type, onto construct files
const CONSTRUCTS: Record<string, string> = {
  core: "document",
  blockquote: "block-quote",
  break: "hard-break",
  "code (flow)": "code-fenced",
  definition: "definition",
  emphasis: "emphasis",
  heading: "heading-atx",
  html: "html-flow",
  image: "image",
  imageReference: "image",
  "code (text)": "code-text",
  link: "link",
  linkReference: "link",
  list: "list",
  listItem: "list-item",
  paragraph: "paragraph",
  strong: "emphasis",
  text: "text",
  "thematic break": "thematic-break",
  escape: "escape",
  roundtrip: "roundtrip",
  attention: "emphasis",
  "roundtrip attention": "roundtrip",
  "roundtrip underscores": "roundtrip",
}

type Group = {
  description: string
  source: { repository: string; commit: string; path: string; license: string }
  pipeline: unknown[]
  tests: object[]
}

const skipped = new Map<string, number>()
importToMarkdown()

function importToMarkdown() {
  const root = fetchSource({ repository: REPOSITORY, commit: COMMIT })
  const source = readFileSync(join(root, "test", "index.js"), "utf8")
  const program = parse(source, { ecmaVersion: "latest", sourceType: "module" })
  const groups = new Map<string, Map<string, Group>>()
  const counts = new Map<string, number>()
  let written = 0
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      if (!isAssertEqual(node.callee)) return
      const [call, expected] = node.arguments
      // The round trip sits on either side of the assertion
      const scope = resolveConstants({ program, ancestors })
      const treeRoundTrip =
        readTreeRoundTrip(call, scope) ?? readTreeRoundTrip(expected, scope)
      if (treeRoundTrip) {
        const topName = findTopTest(ancestors) ?? "roundtrip"
        const construct = CONSTRUCTS[topName] ?? "roundtrip"
        const optionsKey =
          Object.keys(treeRoundTrip.options).length > 0
            ? ` ${JSON.stringify(treeRoundTrip.options)}`
            : ""
        const description = `to-markdown ${topName} (tree round trip)${optionsKey}`
        const byGroup = groups.get(construct) ?? new Map<string, Group>()
        groups.set(construct, byGroup)
        const stringify = optionsKey
          ? ["md.stringify", treeRoundTrip.options]
          : ["md.stringify"]
        const group = byGroup.get(description) ?? {
          description,
          source: {
            repository: REPOSITORY,
            commit: COMMIT,
            path: "test/index.js",
            license: "MIT",
          },
          pipeline: [stringify],
          tests: [],
        }
        const title =
          findTestTitle(ancestors, resolveConstants({ program, ancestors })) ?? topName
        group.tests.push({
          description: title,
          tree: treeRoundTrip.tree,
          expected: { md: treeRoundTrip.md },
        })
        byGroup.set(description, group)
        written += 1
        return
      }
      if (call?.type !== "CallExpression" || !isCallTo(call, "to")) return
      const topName = findTopTest(ancestors)
      const construct = topName ? CONSTRUCTS[topName] : undefined
      if (!topName || !construct)
        return countSkip("outside a node type's tests (position, api)")
      const constants = resolveConstants({ program, ancestors })
      const found = readCase({ call, expected, constants })
      if (!found) return
      const title = findTestTitle(ancestors, constants) ?? topName
      const count = (counts.get(title) ?? 0) + 1
      counts.set(title, count)
      const description = count === 1 ? title : `${title} (${count})`
      const byGroup = groups.get(construct) ?? new Map<string, Group>()
      groups.set(construct, byGroup)
      const group = getGroup({
        byGroup,
        topName,
        roundTrip: "input" in found,
        options: found.options,
      })
      group.tests.push({ description, ...found.test })
      written += 1
    },
  })
  for (const [construct, byGroup] of groups) {
    const path = join(OUTPUT, `${construct}.json`)
    writeGroups({ path, repository: REPOSITORY, groups: [...byGroup.values()] })
  }
  console.log(`to-markdown: ${written} cases in ${groups.size} files`)
  for (const [reason, count] of skipped) console.log(`  skipped ${count}: ${reason}`)
}

// `removePosition(from(to(from(markdown), options?)))` asserts a tree survives serializing; a
// pipeline cannot re-parse, so the serializing half is the case: the parsed tree in, Markdown out
function readTreeRoundTrip(node: AnyNode | undefined, constants: Map<string, AnyNode>) {
  if (node?.type !== "CallExpression" || !isCallTo(node, "removePosition"))
    return undefined
  const reparse = node.arguments[0]
  if (reparse?.type !== "CallExpression" || !isCallTo(reparse, "from")) return undefined
  const serialize = reparse.arguments[0]
  if (serialize?.type !== "CallExpression" || !isCallTo(serialize, "to")) return undefined
  const subject = serialize.arguments[0]
  const parsed = subject?.type === "Identifier" ? constants.get(subject.name) : subject
  if (parsed?.type !== "CallExpression" || !isCallTo(parsed, "from")) return undefined
  const input = evaluateNode(parsed.arguments[0], constants)
  const options = serialize.arguments[1]
    ? evaluateNode(serialize.arguments[1], constants)
    : {}
  if (typeof input !== "string" || !isObject(options)) return undefined
  const tree = JSON.parse(
    JSON.stringify(parseMarkdown(input), (key, value) =>
      key === "position" ? undefined : value,
    ),
  )
  return {
    tree,
    options: { ...options },
    md: stringifyTree({ tree, stringify: { ...options } }),
  }
}

// `to(tree, options?)` stringifies a tree; `to(from(markdown), options?)` round-trips text
function readCase(props: {
  call: CallExpression
  expected: AnyNode | undefined
  constants: Map<string, AnyNode>
}) {
  const [subject, rawOptions] = props.call.arguments
  const options = rawOptions ? evaluateNode(rawOptions, props.constants) : undefined
  const markdown = evaluateNode(props.expected, props.constants)
  // to-markdown's nested serializer extensions are its plugin mechanism, not a textlane option
  if (isObject(options) && "extensions" in options)
    return countSkip("nested to-markdown extensions")
  if ((rawOptions && !isObject(options)) || typeof markdown !== "string") {
    return countSkip("options or output not statically evaluable (functions, extensions)")
  }
  if (props.call.arguments.length > 2) return countSkip("more than tree and options")
  // `const tree = from(markdown)` then `to(tree)` is the same round trip, split in two
  const parsed =
    subject?.type === "Identifier" ? props.constants.get(subject.name) : subject
  if (parsed?.type === "CallExpression" && isCallTo(parsed, "from")) {
    const input = evaluateNode(parsed.arguments[0], props.constants)
    if (parsed.arguments.length > 1)
      return countSkip("a round trip parsed with extensions")
    if (typeof input !== "string")
      return countSkip(
        "options or output not statically evaluable (functions, extensions)",
      )
    if (isStressInput(input)) return countSkip("a stress test")
    return { options, input, test: { input, expected: { md: markdown } } }
  }
  const tree = evaluateNode(subject, props.constants)
  if (!isObject(tree)) return countSkip("a tree not statically evaluable")
  return { options, test: { tree, expected: { md: markdown } } }
}

function getGroup(props: {
  byGroup: Map<string, Group>
  topName: string
  roundTrip: boolean
  options: unknown
}) {
  const optionsKey = props.options === undefined ? "" : JSON.stringify(props.options)
  const id = `${props.topName}#${props.roundTrip}#${optionsKey}`
  const existing = props.byGroup.get(id)
  if (existing) return existing
  const stringify =
    props.options === undefined ? ["md.stringify"] : ["md.stringify", props.options]
  const kind = props.roundTrip ? " round trip" : ""
  const group: Group = {
    description: `to-markdown ${props.topName}${kind}${optionsKey ? ` ${optionsKey}` : ""}`,
    source: {
      repository: REPOSITORY,
      commit: COMMIT,
      path: "test/index.js",
      license: "MIT",
    },
    pipeline: props.roundTrip ? [["md.parse"], stringify] : [stringify],
    tests: [],
  }
  props.byGroup.set(id, group)
  return group
}

function findTopTest(ancestors: AnyNode[]) {
  for (const node of ancestors) {
    if (node.type !== "CallExpression" || !isCallTo(node, "test")) continue
    const title = node.arguments[0]
    if (title?.type === "Literal" && typeof title.value === "string") return title.value
  }
  return undefined
}

function isCallTo(call: CallExpression, name: string) {
  return call.callee.type === "Identifier" && call.callee.name === name
}

function countSkip(reason: string) {
  skipped.set(reason, (skipped.get(reason) ?? 0) + 1)
  return undefined
}
