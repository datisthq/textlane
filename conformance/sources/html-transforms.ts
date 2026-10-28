import { readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { parse } from "acorn"
import type { AnyNode, CallExpression } from "acorn"
import { ancestor } from "acorn-walk"
import { raw } from "hast-util-raw"
import { defaultSchema, sanitize } from "hast-util-sanitize"
import { h, s } from "hastscript"
import rehypeParse from "rehype-parse"
import rehypeSanitize from "rehype-sanitize"
import rehypeSlug from "rehype-slug"
import rehypeStringify from "rehype-stringify"
import { unified } from "unified"
import { u } from "unist-builder"
import { writeGroups } from "../actions/group/write.ts"
import { evaluateNode, isObject } from "../actions/node/evaluate.ts"
import type { Builders } from "../actions/node/evaluate.ts"
import { resolveConstants } from "../actions/node/scope.ts"
import { fetchSource } from "../actions/source/fetch.ts"
import { isAssertEqual } from "../actions/test/assert.ts"
import { findTestTitle } from "../actions/test/find.ts"

const OUTPUT = resolve(import.meta.dirname, "..", "tests", "html")
const RAW = {
  url: "https://github.com/syntax-tree/hast-util-raw",
  commit: "ad15a52350463223504999a6c4c763e1994522d4",
}
const SANITIZE = {
  url: "https://github.com/syntax-tree/hast-util-sanitize",
  commit: "7f30d9e6261583efc544ff6a93ba54ca6e53e1b5",
}
const REHYPE_SANITIZE = {
  url: "https://github.com/rehypejs/rehype-sanitize",
  commit: "b3ee205aeda4e0fc276cb6664b2d6d339305fcd6",
}
const SLUG = {
  url: "https://github.com/rehypejs/rehype-slug",
  commit: "f90fc8f4b469b54b26c13ce4b754bd7e6da3e351",
}
const SLUGGER = {
  url: "https://github.com/Flet/github-slugger",
  commit: "3461c4350868329c8530904d170358bca1d31448",
}
// Trees are built by the pinned builders; `defaultSchema` is read as the pinned value
const BUILDERS: Builders = { h, s, u, defaultSchema }

type Repository = { url: string; commit: string }
type Source = { repository: string; commit: string; path: string; license: string }
type Group = { description: string; source: Source; pipeline: unknown[]; tests: object[] }

const skipped = new Map<string, number>()
let written = 0
importTransforms()

function importTransforms() {
  writeTreeCases({
    repository: RAW,
    path: "test.js",
    callee: "raw",
    plugin: "html.raw",
    construct: "raw",
    render: (tree, options) => raw(tree, options),
  })
  writeTreeCases({
    repository: SANITIZE,
    path: "test/index.js",
    callee: "sanitize",
    plugin: "html.sanitize",
    construct: "sanitize",
    render: (tree, options) => sanitize(tree, options),
  })
  writeChainCases({
    repository: REHYPE_SANITIZE,
    path: "test.js",
    construct: "sanitize",
    plugin: "html.sanitize",
    name: "rehypeSanitize",
    render: (input, parse, options) =>
      String(
        unified()
          .use(rehypeParse, parse)
          .use(rehypeSanitize, options)
          .use(rehypeStringify)
          .processSync(input),
      ),
  })
  writeChainCases({
    repository: SLUG,
    path: "test.js",
    construct: "slug",
    plugin: "html.slug",
    name: "rehypeSlug",
    render: (input, parse, options) =>
      String(
        unified()
          .use(rehypeParse, parse)
          .use(rehypeSlug, options)
          .use(rehypeStringify)
          .processSync(input),
      ),
  })
  writeSluggerCases()
  console.log(`html transforms: ${written} cases`)
  for (const [reason, count] of skipped) console.log(`  skipped ${count}: ${reason}`)
}

// `assert.deepEqual(callee(tree, options?), …)`, plus hast-util-sanitize's `testAllUrls(tag,
// property, {valid, invalid})` expanded into one literal case per URL, as its helper loops
function writeTreeCases(props: {
  repository: Repository
  path: string
  callee: string
  plugin: string
  construct: string
  render: (tree: never, options: never) => unknown
}) {
  const program = readProgram(props.repository, props.path)
  const groups = new Map<string, Group>()
  const counts = new Map<string, number>()
  const add = (title: string, tree: object, options: object) => {
    const count = (counts.get(title) ?? 0) + 1
    counts.set(title, count)
    const optionsKey =
      Object.keys(options).length > 0 ? ` ${JSON.stringify(options)}` : ""
    const description = `${props.repository.url.split("/").pop()}${optionsKey}`
    const pipeline = [
      Object.keys(options).length > 0 ? [props.plugin, options] : [props.plugin],
    ]
    // A JSON tree is valid hast; the utilities type their nodes, not plain objects
    const expected = JSON.parse(
      JSON.stringify(
        Reflect.apply(props.render, undefined, [structuredClone(tree), options]),
      ),
    )
    const group = groups.get(description) ?? {
      description,
      source: source(props.repository, props.path),
      pipeline,
      tests: [],
    }
    group.tests.push({
      description: count === 1 ? title : `${title} (${count})`,
      tree,
      expected: { tree: expected },
    })
    groups.set(description, group)
    written += 1
  }
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      const constants = resolveConstants({ program, ancestors })
      if (isCallTo(node, "testAllUrls")) {
        const [tagName, property, all] = node.arguments.map(item =>
          evaluateNode(item, constants, 0, BUILDERS),
        )
        if (typeof tagName !== "string" || typeof property !== "string" || !isObject(all))
          return countSkip("not statically evaluable")
        for (const [kind, urls] of Object.entries(all)) {
          for (const [name, url] of Object.entries(isObject(urls) ? urls : {})) {
            const tree = JSON.parse(
              JSON.stringify(h(tagName, { [property]: String(url) })),
            )
            add(
              `should ${kind === "valid" ? "allow" : "clean"} ${name} in ${tagName}[${property}]`,
              tree,
              {},
            )
          }
        }
        return
      }
      if (!isAssertEqual(node.callee)) return
      const call = node.arguments[0]
      if (call?.type !== "CallExpression" || !isCallTo(call, props.callee)) return
      const tree = evaluateNode(call.arguments[0], constants, 0, BUILDERS)
      const options = call.arguments[1]
        ? evaluateNode(call.arguments[1], constants, 0, BUILDERS)
        : {}
      if (!isObject(tree))
        return countSkip("a tree JSON cannot hold or not statically evaluable")
      if (!isObject(options))
        return countSkip("options not statically evaluable (a file, functions)")
      add(findTestTitle(ancestors, constants) ?? props.callee, tree, { ...options })
    },
  })
  writeGroups({
    path: join(OUTPUT, `${props.construct}.json`),
    repository: props.repository.url,
    groups: [...groups.values()],
  })
}

// `rehype().data('settings', {fragment: true}).use(plugin, options?).process(html)`, its HTML
// output asserted
function writeChainCases(props: {
  repository: Repository
  path: string
  construct: string
  plugin: string
  name: string
  render: (
    input: string,
    parse: Record<string, unknown>,
    options: Record<string, unknown>,
  ) => string
}) {
  const program = readProgram(props.repository, props.path)
  const groups = new Map<string, Group>()
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      if (
        node.callee.type !== "MemberExpression" ||
        node.callee.property.type !== "Identifier"
      )
        return
      if (!["process", "processSync"].includes(node.callee.property.name)) return
      const constants = resolveConstants({ program, ancestors })
      const chain = readChain(node.callee.object, constants)
      if (!chain?.plugins.has(props.name)) return
      const input = evaluateNode(node.arguments[0], constants, 0, BUILDERS)
      const optionsNode = chain.plugins.get(props.name)
      const options = optionsNode ? evaluateNode(optionsNode, constants, 0, BUILDERS) : {}
      if (typeof input !== "string") return countSkip("not statically evaluable")
      if (!isObject(options)) return countSkip("options not statically evaluable")
      const parseOptions = chain.fragment ? { fragment: true } : {}
      const html = props.render(input, parseOptions, { ...options })
      const optionsKey =
        Object.keys(options).length > 0 ? ` ${JSON.stringify(options)}` : ""
      const description = `${props.repository.url.split("/").pop()}${optionsKey}`
      const pipeline = [
        chain.fragment ? ["html.parse", { fragment: true }] : ["html.parse"],
        Object.keys(options).length > 0 ? [props.plugin, options] : [props.plugin],
        ["html.stringify"],
      ]
      const group = groups.get(description) ?? {
        description,
        source: source(props.repository, props.path),
        pipeline,
        tests: [],
      }
      group.tests.push({
        description: findTestTitle(ancestors, constants) ?? props.plugin,
        input,
        expected: { html },
      })
      groups.set(description, group)
      written += 1
    },
  })
  writeGroups({
    path: join(OUTPUT, `${props.construct}.json`),
    repository: props.repository.url,
    groups: [...groups.values()],
  })
}

// github-slugger's fixtures run through one stateful slugger; as headings in fresh documents they
// pin the slug of each input on its own
function writeSluggerCases() {
  const root = fetchSource({ repository: SLUGGER.url, commit: SLUGGER.commit })
  const fixtures: { name: string; input: string }[] = JSON.parse(
    readFileSync(join(root, "test", "fixtures.json"), "utf8"),
  )
  const processor = unified()
    .use(rehypeParse, { fragment: true })
    .use(rehypeSlug)
    .use(rehypeStringify)
  const seen = new Set<string>()
  const tests = fixtures.flatMap(fixture => {
    if (seen.has(fixture.input) || !fixture.input.isWellFormed()) return []
    seen.add(fixture.input)
    const input = `<h1>${fixture.input.replace(/&/g, "&amp;").replace(/</g, "&lt;")}</h1>`
    written += 1
    return [
      {
        description: fixture.name,
        input,
        expected: { html: String(processor.processSync(input)) },
      },
    ]
  })
  writeGroups({
    path: join(OUTPUT, "slug.json"),
    repository: SLUGGER.url,
    groups: [
      {
        description: "github-slugger fixtures as headings",
        source: source(SLUGGER, "test/fixtures.json"),
        pipeline: [["html.parse", { fragment: true }], ["html.slug"], ["html.stringify"]],
        tests,
      },
    ],
  })
}

function readChain(
  node: AnyNode,
  constants: Map<string, AnyNode>,
  depth = 0,
): { plugins: Map<string, AnyNode | undefined>; fragment: boolean } | undefined {
  if (depth > 10) return undefined
  if (node.type === "Identifier") {
    const bound = constants.get(node.name)
    return bound ? readChain(bound, constants, depth + 1) : undefined
  }
  if (node.type !== "CallExpression") return undefined
  if (isCallTo(node, "rehype")) return { plugins: new Map(), fragment: false }
  if (
    node.callee.type !== "MemberExpression" ||
    node.callee.property.type !== "Identifier"
  )
    return undefined
  const chain = readChain(node.callee.object, constants, depth + 1)
  if (!chain) return undefined
  const method = node.callee.property.name
  if (method === "data") {
    const settings = evaluateNode(node.arguments[1], constants)
    if (isObject(settings) && "fragment" in settings && settings.fragment === true)
      chain.fragment = true
  }
  if (method === "use" && node.arguments[0]?.type === "Identifier")
    chain.plugins.set(node.arguments[0].name, node.arguments[1])
  return chain
}

function readProgram(repository: Repository, path: string) {
  const root = fetchSource({ repository: repository.url, commit: repository.commit })
  return parse(readFileSync(join(root, path), "utf8"), {
    ecmaVersion: "latest",
    sourceType: "module",
  })
}

function source(repository: Repository, path: string): Source {
  return {
    repository: repository.url,
    commit: repository.commit,
    path,
    license: repository === SLUGGER ? "ISC" : "MIT",
  }
}

function isCallTo(call: CallExpression, name: string) {
  return call.callee.type === "Identifier" && call.callee.name === name
}

function countSkip(reason: string) {
  skipped.set(reason, (skipped.get(reason) ?? 0) + 1)
  return undefined
}
