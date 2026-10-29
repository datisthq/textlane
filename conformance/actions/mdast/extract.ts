import { parse } from "acorn"
import type { AnyNode, CallExpression } from "acorn"
import { ancestor } from "acorn-walk"
import { isStressInput } from "../input/check.ts"
import { parseMarkdown } from "../markdown/render.ts"
import type { Extensions } from "../markdown/render.ts"
import { evaluateNode, isObject } from "../node/evaluate.ts"
import { resolveConstants } from "../node/scope.ts"
import { isAssertEqual } from "../test/assert.ts"
import { findTestTitle } from "../test/find.ts"

/** The syntax extension an mdast-util's tests exercise, and the textlane plugin it maps onto. */
export type MdastSyntax = {
  plugin: string
  extension: keyof Extensions
  isExtension: (name: string) => boolean
  // Named object options, or the extension's one argument taken whole (frontmatter's matters)
  options: Set<string> | "positional"
}

/** A case's kind: a tree from Markdown, Markdown from a tree, or Markdown back to Markdown. */
export type MdastKind = "tree" | "stringify" | "round trip"

/** Options read from a test: the syntax plugin's and to-markdown's own. */
export type MdastOptions = { plugin: unknown; stringify: Record<string, unknown> }

/** A group-description suffix naming a case's options, empty when it has none. */
export function describeMdastOptions(options: MdastOptions) {
  if (isObject(options.plugin) || options.plugin === undefined) {
    const plugin = isObject(options.plugin) ? options.plugin : {}
    const key = JSON.stringify({ ...plugin, ...options.stringify })
    return key === "{}" ? "" : ` ${key}`
  }
  const stringify =
    Object.keys(options.stringify).length > 0
      ? ` ${JSON.stringify(options.stringify)}`
      : ""
  return ` ${JSON.stringify(options.plugin)}${stringify}`
}

/** Read an mdast-util's test file: `fromMarkdown(md, ext)` is a tree case, `toMarkdown(tree,
 * ext)` a stringify case and `toMarkdown(fromMarkdown(md, …), …)` a round trip. */
export function extractMdastCases(props: {
  source: string
  syntax: MdastSyntax
  countSkip: (reason: string) => undefined
}) {
  const program = parse(props.source, { ecmaVersion: "latest", sourceType: "module" })
  const cases: { kind: MdastKind; options: MdastOptions; test: object }[] = []
  const counts = new Map<string, number>()
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      if (!isAssertEqual(node.callee)) return
      const [call, expected] = node.arguments
      if (call?.type !== "CallExpression") return
      const constants = resolveConstants({ program, ancestors })
      const title = findTestTitle(ancestors, constants) ?? "test"
      const count = (counts.get(title) ?? 0) + 1
      counts.set(title, count)
      const description = count === 1 ? title : `${title} (${count})`
      const found = readCall({ ...props, call, expected, constants })
      if (found) cases.push({ ...found, test: { description, ...found.test } })
    },
  })
  return cases
}

/** The textlane pipeline a case of that kind runs, as data. */
export function createMdastPipeline(props: {
  syntax: MdastSyntax
  kind: MdastKind
  options: MdastOptions
}) {
  const plugin = withOptions(props.syntax.plugin, props.options.plugin)
  const stringify = withOptions("md.stringify", props.options.stringify)
  if (props.kind === "tree") return [["md.parse"], plugin]
  if (props.kind === "stringify") return [plugin, stringify]
  return [["md.parse"], plugin, stringify]
}

function readCall(props: {
  syntax: MdastSyntax
  countSkip: (reason: string) => undefined
  call: CallExpression
  expected: AnyNode | undefined
  constants: Map<string, AnyNode>
}) {
  if (isCallTo(props.call, "fromMarkdown")) {
    const input = evaluateNode(props.call.arguments[0], props.constants)
    const options = readOptions({ ...props, node: props.call.arguments[1] })
    if (typeof input !== "string") return props.countSkip("not statically evaluable")
    if (typeof options === "string") return props.countSkip(options)
    if (isStressInput(input)) return props.countSkip("a stress test")
    let tree: object
    try {
      tree = parseMarkdown(input, toExtensions(props.syntax, options.plugin))
    } catch {
      return props.countSkip("an input the reference parser rejects")
    }
    return { kind: "tree" as const, options, test: { input, expected: { tree } } }
  }
  if (!isCallTo(props.call, "toMarkdown")) return undefined
  const md = evaluateNode(props.expected, props.constants)
  const options = readOptions({ ...props, node: props.call.arguments[1] })
  if (typeof md !== "string") return props.countSkip("not statically evaluable")
  if (typeof options === "string") return props.countSkip(options)
  const subject = props.call.arguments[0]
  const parsed =
    subject?.type === "Identifier" ? props.constants.get(subject.name) : subject
  if (parsed?.type === "CallExpression" && isCallTo(parsed, "fromMarkdown")) {
    const input = evaluateNode(parsed.arguments[0], props.constants)
    if (typeof input !== "string") return props.countSkip("not statically evaluable")
    return { kind: "round trip" as const, options, test: { input, expected: { md } } }
  }
  const tree = evaluateNode(subject, props.constants)
  if (!isObject(tree)) return props.countSkip("a tree not statically evaluable")
  return { kind: "stringify" as const, options, test: { tree, expected: { md } } }
}

// `{extensions: [ext(args?)], mdastExtensions: [extFromMarkdown()], …}`; to-markdown's own options
// pass through to `md.stringify`; a string result names why the case is skipped
function readOptions(props: {
  syntax: MdastSyntax
  node: AnyNode | undefined
  constants: Map<string, AnyNode>
}): MdastOptions | string {
  const target =
    props.node?.type === "Identifier" ? props.constants.get(props.node.name) : props.node
  if (!target) return "no extension enabled"
  if (target.type !== "ObjectExpression") return "options not statically evaluable"
  const named: Record<string, unknown> = {}
  const positional = props.syntax.options === "positional"
  const options: MdastOptions = { plugin: positional ? undefined : named, stringify: {} }
  let enabled = false
  for (const property of target.properties) {
    if (property.type !== "Property" || property.key.type !== "Identifier") {
      return "options not statically evaluable"
    }
    const key = property.key.name
    if (key !== "extensions" && key !== "mdastExtensions") {
      const value = evaluateNode(property.value, props.constants)
      if (value === undefined) return "options not statically evaluable"
      options.stringify[key] = value
      continue
    }
    if (property.value.type !== "ArrayExpression")
      return "options not statically evaluable"
    for (const element of property.value.elements) {
      if (element?.type !== "CallExpression" || element.callee.type !== "Identifier") {
        return "options not statically evaluable"
      }
      const name = element.callee.name
      if (!props.syntax.isExtension(name)) return `another syntax extension (${name})`
      enabled = true
      const raw = element.arguments[0]
      const argument = raw ? evaluateNode(raw, props.constants) : undefined
      if (raw && argument === undefined)
        return "extension options not statically evaluable (functions)"
      if (props.syntax.options === "positional") {
        options.plugin ??= argument
        continue
      }
      if (argument === undefined) continue
      if (!isObject(argument))
        return "extension options not statically evaluable (functions)"
      for (const [option, value] of Object.entries(argument)) {
        if (!props.syntax.options.has(option)) {
          return `an extension option without a pipeline equivalent (${option})`
        }
        named[option] = value
      }
    }
  }
  return enabled ? options : "no extension enabled"
}

function withOptions(name: string, options: unknown) {
  if (options === undefined || options === null) return [name]
  if (isObject(options) && Object.keys(options).length === 0) return [name]
  return [name, options]
}

// Typed reference options for the syntax under test; frontmatter's `null` means its defaults
/** The reference's extension settings for a syntax and a case's plugin options. */
export function toExtensions(syntax: MdastSyntax, plugin: unknown): Extensions {
  if (syntax.extension === "frontmatter") {
    return { frontmatter: isFrontmatterOptions(plugin) ? plugin : null }
  }
  return { [syntax.extension]: isObject(plugin) ? { ...plugin } : {} }
}

/** Whether a value has the shape remark-frontmatter accepts: a preset, a matter, or a list. */
export function isFrontmatterOptions(
  value: unknown,
): value is NonNullable<Extensions["frontmatter"]> {
  return typeof value === "string" || Array.isArray(value) || isObject(value)
}

function isCallTo(call: CallExpression, name: string) {
  return call.callee.type === "Identifier" && call.callee.name === name
}
