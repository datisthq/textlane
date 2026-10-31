import { readFileSync } from "node:fs"
import { stripTypeScriptTypes } from "node:module"
import { join, resolve } from "node:path"
import { parse } from "acorn"
import type { AnyNode } from "acorn"
import { ancestor } from "acorn-walk"
import { nameToEmoji } from "gemoji"
import { get as getEmoji } from "node-emoji"
import rehypeStringify from "rehype-stringify"
import remarkEmoji from "remark-emoji"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import remarkStringify from "remark-stringify"
import { unified } from "unified"
import { writeGroups } from "../actions/group/write.ts"
import { evaluateNode, isObject } from "../actions/node/evaluate.ts"
import { resolveConstants } from "../actions/node/scope.ts"
import { fetchSource } from "../actions/source/fetch.ts"
import { findTestTitle } from "../actions/test/find.ts"

const OUTPUT = join(resolve(import.meta.dirname, "..", "tests", "md"), "gemoji.json")
const EMOJI = {
  url: "https://github.com/rhysd/remark-emoji",
  commit: "2373e26b06f5d446435f64e42760454a7b662d22",
}
// remark-emoji's compilers, as `md.gemoji` options and the output each test checks; the
// ones adding remark-github or rehype-slug test other plugins
const COMPILERS: Record<
  string,
  { options: Record<string, boolean>; output: "md" | "html" }
> = {
  compiler: { options: {}, output: "md" },
  padded: { options: { padSpaceAfter: true }, output: "md" },
  emoticon: { options: { emoticon: true }, output: "md" },
  padAndEmoticon: { options: { padSpaceAfter: true, emoticon: true }, output: "md" },
  ariaHtml: { options: { emoticon: true, accessible: true }, output: "html" },
}
// textlane's own inputs: emoticons right after a shortcode, where find-and-replace starts a
// new text, Unicode white space before one, refused matches retried a character on
const OWN: [string, Record<string, boolean>][] = [
  [":dog::) and :cat: :-(", { emoticon: true }],
  ["a　:) b ;) c", { emoticon: true }],
  [":):) x :-)) (:", { emoticon: true }],
  ["**bold :) text** and [link :D](#)", { emoticon: true, accessible: true }],
  [":+1: :heavy_check_mark: :t-rex:", { accessible: true, padSpaceAfter: true }],
]

const skipped = new Map<string, number>()
const groups = new Map<
  string,
  { options: Record<string, boolean>; output: "md" | "html"; tests: object[] }
>()
let written = 0
for (const { title, input, compiler } of readTests())
  addCase(title, input, compiler.options, compiler.output)
for (const [input, options] of OWN) {
  addCase(JSON.stringify(input), input, options, "md")
  addCase(JSON.stringify(input), input, options, "html")
}
writeGroups({
  path: OUTPUT,
  repository: EMOJI.url,
  groups: [...groups].map(([key, group]) => ({
    description: `remark-emoji ${key}`,
    comment:
      "md.gemoji keeps gemoji's names where remark-emoji reads node-emoji's; cases whose shortcodes the two name differently are left out",
    source: {
      repository: EMOJI.url,
      commit: EMOJI.commit,
      path: "test.ts",
      license: "MIT",
    },
    pipeline: [
      ["md.parse"],
      ["md.gemoji", group.options],
      ...(group.output === "md"
        ? [["md.stringify"]]
        : [["md.toHtml"], ["html.stringify"]]),
    ],
    tests: group.tests,
  })),
})
console.log(`emoji: ${written} cases`)
for (const [reason, count] of skipped) console.log(`  skipped ${count}: ${reason}`)

function addCase(
  title: string,
  input: string,
  options: Record<string, boolean>,
  output: "md" | "html",
) {
  const names = input.match(/:\+1:|:-1:|:[\w-]+:/g) ?? []
  if (names.some(name => getEmoji(name) !== nameToEmoji[name.slice(1, -1)])) {
    skipped.set(
      "a shortcode node-emoji and gemoji name differently",
      (skipped.get("a shortcode node-emoji and gemoji name differently") ?? 0) + 1,
    )
    return
  }
  const processor = unified().use(remarkParse).use(remarkEmoji, options)
  const value =
    output === "md"
      ? String(processor.use(remarkStringify).processSync(input))
      : String(processor.use(remarkRehype).use(rehypeStringify).processSync(input))
  const key = `${output} ${JSON.stringify(options)}`
  const group = groups.get(key) ?? { options, output, tests: [] }
  if (group.tests.some(test => isObject(test) && "input" in test && test.input === input))
    return
  group.tests.push({ description: title, input, expected: { [output]: value } })
  groups.set(key, group)
  written += 1
}

// `<compiler>.process(input)`: a literal or constant, or each key of the `tests` table its
// loop walks
function readTests() {
  const root = fetchSource({ repository: EMOJI.url, commit: EMOJI.commit })
  const source = stripTypeScriptTypes(readFileSync(join(root, "test.ts"), "utf8"))
  const program = parse(source, { ecmaVersion: "latest", sourceType: "module" })
  const tests: { title: string; input: string; compiler: (typeof COMPILERS)[string] }[] =
    []
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      const callee = node.callee
      if (
        callee.type !== "MemberExpression" ||
        callee.object.type !== "Identifier" ||
        callee.property.type !== "Identifier" ||
        callee.property.name !== "process"
      )
        return
      const compiler = COMPILERS[callee.object.name]
      if (!compiler) return
      const constants = resolveConstants({ program, ancestors })
      const title = findTestTitle(ancestors, constants) ?? callee.object.name
      const table: unknown = evaluateTable(constants.get("tests"), constants)
      const argument = node.arguments[0]
      if (
        argument?.type === "Identifier" &&
        argument.name === "input" &&
        isObject(table)
      ) {
        for (const input of Object.keys(table)) tests.push({ title, input, compiler })
        return
      }
      const input = evaluateNode(argument, constants)
      if (typeof input === "string") tests.push({ title, input, compiler })
    },
  })
  return tests
}

function evaluateTable(node: AnyNode | undefined, constants: Map<string, AnyNode>) {
  return node ? evaluateNode(node, constants) : undefined
}
