import { readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { parse } from "acorn"
import type { AnyNode, Program } from "acorn"
import { ancestor, simple } from "acorn-walk"
import retextContractions from "retext-contractions"
import retextEnglish from "retext-english"
import retextEquality from "retext-equality"
import retextIndefiniteArticle from "retext-indefinite-article"
import retextLatin from "retext-latin"
import retextPassive from "retext-passive"
import retextQuotes from "retext-quotes"
import retextReadability from "retext-readability"
import retextRepeatedWords from "retext-repeated-words"
import retextSentenceSpacing from "retext-sentence-spacing"
import retextSmartypants from "retext-smartypants"
import retextStringify from "retext-stringify"
import { unified } from "unified"
import type { PluggableList } from "unified"
import type { VFile } from "vfile"
import { writeGroups } from "../actions/group/write.ts"
import { evaluateNode, isObject } from "../actions/node/evaluate.ts"
import { resolveConstants } from "../actions/node/scope.ts"
import { fetchSource } from "../actions/source/fetch.ts"
import { findTestTitle } from "../actions/test/find.ts"

const OUTPUT = resolve(import.meta.dirname, "..", "tests", "prose")

// Each plugin's repository, the identifier its test imports it as, and how a run is rendered:
// smartypants changes the text, the others report messages
const PLUGINS = [
  {
    name: "retext-smartypants",
    commit: "0ffca5eeb04b3d163624aad28ef36911edd224d7",
    identifier: "retextSmartypants",
    plugin: "prose.smartypants",
    outcome: "txt",
    run: (english: boolean, options: object, input: string) =>
      process(english, [retextSmartypants, options], input),
  },
  {
    name: "retext-equality",
    commit: "192deddf13bc2823540b0d9a0a6e17fb6d995bcd",
    identifier: "retextEquality",
    plugin: "prose.equality",
    outcome: "messages",
    run: (english: boolean, options: object, input: string) =>
      process(english, [retextEquality, options], input),
  },
  {
    name: "retext-indefinite-article",
    commit: "960875ee40ffd0806054d634f96052955fff8c18",
    identifier: "retextIndefiniteArticle",
    plugin: "prose.indefiniteArticle",
    outcome: "messages",
    run: (english: boolean, _options: object, input: string) =>
      process(english, [retextIndefiniteArticle], input),
  },
  {
    name: "retext-quotes",
    commit: "e43bb5f39b79b92c30f96485c7f7751a892b5700",
    identifier: "retextQuotes",
    plugin: "prose.quotes",
    outcome: "messages",
    run: (english: boolean, options: object, input: string) =>
      process(english, [retextQuotes, options], input),
  },
  {
    name: "retext-contractions",
    commit: "5568b4fb4dffd1fd9235c296780b7b4b58077200",
    identifier: "retextContractions",
    plugin: "prose.contractions",
    outcome: "messages",
    run: (english: boolean, options: object, input: string) =>
      process(english, [retextContractions, options], input),
  },
  {
    name: "retext-readability",
    commit: "ece962416e72edeea12f670d3120da4f32e0d423",
    identifier: "retextReadability",
    plugin: "prose.readability",
    outcome: "messages",
    run: (english: boolean, options: object, input: string) =>
      process(english, [retextReadability, options], input),
  },
  {
    name: "retext-sentence-spacing",
    commit: "63807dc0c23cc1e303ac891a3a450d3809c81bc9",
    identifier: "retextSentenceSpacing",
    plugin: "prose.sentenceSpacing",
    outcome: "messages",
    run: (english: boolean, options: object, input: string) =>
      process(english, [retextSentenceSpacing, options], input),
  },
  {
    name: "retext-repeated-words",
    commit: "d6d53fb61144ccabda6ca2fe1420014c0c070b7e",
    identifier: "retextRepeatedWords",
    plugin: "prose.repeatedWords",
    outcome: "messages",
    run: (english: boolean, _options: object, input: string) =>
      process(english, [retextRepeatedWords], input),
  },
  {
    name: "retext-passive",
    commit: "66ca2c564252bcfd395927a03b184f3dc19fc50a",
    identifier: "retextPassive",
    plugin: "prose.passive",
    outcome: "messages",
    run: (english: boolean, options: object, input: string) =>
      process(english, [retextPassive, options], input),
  },
] as const

type Plugin = (typeof PLUGINS)[number]
type Found = {
  title: string
  input: string
  options: Record<string, unknown>
  english: boolean
}

const skipped = new Map<string, number>()
let written = 0
for (const plugin of PLUGINS) importPlugin(plugin)
console.log(`prose plugins: ${written} cases`)
for (const [reason, count] of skipped) console.log(`  skipped ${count}: ${reason}`)

function importPlugin(plugin: Plugin) {
  const repository = `https://github.com/retextjs/${plugin.name}`
  const root = fetchSource({ repository, commit: plugin.commit })
  const program = parse(readFileSync(join(root, "test.js"), "utf8"), {
    ecmaVersion: "latest",
    sourceType: "module",
  })
  const groups = new Map<string, { pipeline: unknown[]; tests: object[] }>()
  const seen = new Set<string>()
  for (const found of extractCases(program, plugin)) {
    const key = JSON.stringify([found.english, found.options, found.input])
    if (seen.has(key)) {
      countSkip("duplicate of an earlier case")
      continue
    }
    seen.add(key)
    const file = plugin.run(found.english, found.options, found.input)
    const expected =
      plugin.outcome === "txt"
        ? { txt: String(file) }
        : { messages: describeMessages(file) }
    const parsePlugin = found.english
      ? ["prose.parse", { language: "english" }]
      : ["prose.parse"]
    const pipeline = [
      parsePlugin,
      Object.keys(found.options).length > 0
        ? [plugin.plugin, found.options]
        : [plugin.plugin],
      ["prose.stringify"],
    ]
    const groupKey = JSON.stringify(pipeline)
    const group = groups.get(groupKey) ?? { pipeline, tests: [] }
    group.tests.push({ description: found.title, input: found.input, expected })
    groups.set(groupKey, group)
    written += 1
  }
  const source = { repository, commit: plugin.commit, path: "test.js", license: "MIT" }
  writeGroups({
    path: join(
      OUTPUT,
      `${plugin.plugin.replace("prose.", "").replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)}.json`,
    ),
    repository,
    groups: [...groups.values()].map(group => {
      const options = group.pipeline[1]
      const suffix =
        Array.isArray(options) && options[1] ? ` ${JSON.stringify(options[1])}` : ""
      const english = JSON.stringify(group.pipeline[0]).includes("english")
        ? " (English)"
        : ""
      return { description: `${plugin.name}${english}${suffix}`, source, ...group }
    }),
  })
}

// `<chain>.process(input)` / `.processSync(input)` where the chain uses the plugin, or a local
// helper whose body does with its parameters; an input `list[index]` of a constant list is each
// of its entries
function extractCases(program: Program, plugin: Plugin) {
  const found: Found[] = []
  const counts = new Map<string, number>()
  const helpers = findHelpers(program, plugin)
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      const constants = resolveConstants({ program, ancestors })
      let english = false
      let optionsNode: AnyNode | undefined
      let inputNode: AnyNode | undefined
      if (
        node.callee.type === "MemberExpression" &&
        node.callee.property.type === "Identifier" &&
        ["process", "processSync"].includes(node.callee.property.name)
      ) {
        const chain = readChain(node.callee.object, constants)
        if (!chain?.has(plugin.identifier)) return
        english = chain.has("retextEnglish")
        optionsNode = chain.get(plugin.identifier)
        inputNode = node.arguments[0]
      } else if (node.callee.type === "Identifier" && helpers.has(node.callee.name)) {
        const helper = helpers.get(node.callee.name)
        if (!helper) return
        english = helper.english
        inputNode = node.arguments[helper.input]
        optionsNode =
          helper.options === undefined ? undefined : node.arguments[helper.options]
      } else {
        return
      }
      const options = optionsNode ? evaluateNode(optionsNode, constants) : {}
      if (!isObject(options))
        return countSkip("options not statically evaluable (loops, callbacks)")
      const inputs = resolveInputs(inputNode, constants)
      if (inputs.length === 0) return countSkip("input not statically evaluable")
      const title = findTestTitle(ancestors, constants) ?? plugin.name
      for (const input of inputs) {
        const count = (counts.get(title) ?? 0) + 1
        counts.set(title, count)
        found.push({
          title: count === 1 ? title : `${title} (${count})`,
          input,
          options: { ...options },
          english,
        })
      }
    },
  })
  return found
}

function resolveInputs(
  node: AnyNode | undefined,
  constants: Map<string, AnyNode>,
): string[] {
  const value = evaluateNode(node, constants)
  if (typeof value === "string") return [value]
  const target = node?.type === "Identifier" ? constants.get(node.name) : node
  if (target?.type === "MemberExpression" && target.computed) {
    const list = evaluateNode(target.object, constants)
    if (Array.isArray(list) && list.every(item => typeof item === "string")) return list
  }
  return []
}

// A local function wrapping a chain that uses the plugin: which parameter is the input and which
// the options
function findHelpers(program: Program, plugin: Plugin) {
  const helpers = new Map<string, { input: number; options?: number; english: boolean }>()
  simple(program, {
    FunctionDeclaration(node) {
      if (!node.id) return
      const parameters = node.params.map(parameter =>
        parameter.type === "Identifier" ? parameter.name : "",
      )
      let match: { input: number; options?: number; english: boolean } | undefined
      simple(node.body, {
        CallExpression(call) {
          if (
            call.callee.type !== "MemberExpression" ||
            call.callee.property.type !== "Identifier"
          )
            return
          if (!["process", "processSync"].includes(call.callee.property.name)) return
          const chain = readChain(call.callee.object, new Map())
          const input = call.arguments[0]
          if (!chain?.has(plugin.identifier) || input?.type !== "Identifier") return
          const options = chain.get(plugin.identifier)
          match = {
            input: parameters.indexOf(input.name),
            options:
              options?.type === "Identifier"
                ? parameters.indexOf(options.name)
                : undefined,
            english: chain.has("retextEnglish"),
          }
        },
      })
      if (match && match.input >= 0) helpers.set(node.id.name, match)
    },
  })
  return helpers
}

// The plugins of a `retext().use(a).use(b, options)` chain, each with its options node
function readChain(
  node: AnyNode,
  constants: Map<string, AnyNode>,
  depth = 0,
): Map<string, AnyNode | undefined> | undefined {
  if (depth > 10) return undefined
  if (node.type === "Identifier") {
    const bound = constants.get(node.name)
    return bound ? readChain(bound, constants, depth + 1) : undefined
  }
  if (node.type !== "CallExpression") return undefined
  if (node.callee.type === "Identifier" && node.callee.name === "retext") return new Map()
  if (
    node.callee.type !== "MemberExpression" ||
    node.callee.property.type !== "Identifier"
  )
    return undefined
  const chain = readChain(node.callee.object, constants, depth + 1)
  if (
    chain &&
    node.callee.property.name === "use" &&
    node.arguments[0]?.type === "Identifier"
  ) {
    chain.set(node.arguments[0].name, node.arguments[1])
  }
  return chain
}

// `retext()` is retext-latin with retext-stringify; a chain adding retext-english parses English
function process(english: boolean, plugin: PluggableList[number], input: string): VFile {
  return unified()
    .use(english ? [retextEnglish] : [retextLatin])
    .use([plugin])
    .use(retextStringify)
    .processSync(input)
}

// Messages in place order, as retext-equality's own test sorts them, so emission order is contract
function describeMessages(file: VFile) {
  const messages = file.messages.map(message => ({
    ruleId: message.ruleId ?? "",
    source: message.source ?? "",
    place: JSON.parse(JSON.stringify(message.place ?? null)),
  }))
  const start = (place: unknown) =>
    isObject(place) &&
    "start" in place &&
    isObject(place.start) &&
    "offset" in place.start &&
    typeof place.start.offset === "number"
      ? place.start.offset
      : -1
  return messages.sort(
    (left, right) =>
      start(left.place) - start(right.place) || left.ruleId.localeCompare(right.ruleId),
  )
}

function countSkip(reason: string) {
  skipped.set(reason, (skipped.get(reason) ?? 0) + 1)
  return undefined
}
