import { readFileSync, realpathSync, writeFileSync } from "node:fs"
import { dirname, join, resolve } from "node:path"
import { pathToFileURL } from "node:url"
import { parse } from "acorn"
import type { Expression, SpreadElement } from "acorn"
import { simple } from "acorn-walk"

// The data tables `textlane-prose`'s checks read, generated from the pinned retext packages
// and the packages they depend on, so the Rust tables never drift from what upstream ships
const MODULES = resolve(import.meta.dirname, "..", "node_modules")
const HELPERS = resolve(
  import.meta.dirname,
  "..",
  "..",
  "rust",
  "prose",
  "src",
  "helpers",
)

const equality = locate("retext-equality")
const passive = locate("retext-passive")
const readability = locate("retext-readability")
const syllable = sibling(readability, "syllable")

const patterns: Pattern[] = (await load(join(equality, "lib", "patterns-en.js"))).patterns
const list: string[] = (await load(join(passive, "lib", "list.js"))).list
const daleChall: string[] = (
  await load(join(sibling(readability, "dale-chall"), "index.js"))
).daleChall
const spache: string[] = (await load(join(sibling(readability, "spache"), "index.js")))
  .spache
const problematic: Record<string, number> = (await load(join(syllable, "problematic.js")))
  .problematic
const charmap: Record<string, string> = JSON.parse(
  readFileSync(join(sibling(syllable, "normalize-strings"), "charmap.json"), "utf8"),
)
const pluralize = readPluralize(join(sibling(syllable, "pluralize"), "pluralize.js"))

writeEquality()
writePassive()
writeReadability()

function writeEquality() {
  const rows = patterns.map(pattern => {
    const condition =
      pattern.condition === undefined ? "None" : `Some(${quote(pattern.condition)})`
    const considerate = Object.keys(pattern.considerate ?? {}).map(quote)
    const inconsiderate = Object.entries(pattern.inconsiderate).map(
      ([phrase, kind]) => `(${quote(phrase)}, ${quote(kind)})`,
    )
    return [
      "    EqualityPattern {",
      `        id: ${quote(pattern.id)},`,
      `        or: ${pattern.type === "or"},`,
      `        apostrophe: ${pattern.apostrophe === true},`,
      `        condition: ${condition},`,
      `        considerate: &[${considerate.join(", ")}],`,
      `        inconsiderate: &[${inconsiderate.join(", ")}],`,
      "    },",
    ].join("\n")
  })
  write("equality_table.rs", [
    "// retext-equality 7.1.0's English patterns, in their order, each phrase with its category",
    "",
    "/// One retext-equality pattern.",
    "pub(crate) struct EqualityPattern {",
    "    pub id: &'static str,",
    "    pub or: bool,",
    "    pub apostrophe: bool,",
    "    pub condition: Option<&'static str>,",
    "    pub considerate: &'static [&'static str],",
    "    pub inconsiderate: &'static [(&'static str, &'static str)],",
    "}",
    "",
    `pub(crate) static PATTERNS: [EqualityPattern; ${rows.length}] = [`,
    ...rows,
    "];",
  ])
}

function writePassive() {
  write("passive_table.rs", [
    "// retext-passive 5.0.0's participles, in their order",
    `pub(crate) static PARTICIPLES: [&str; ${list.length}] = [`,
    ...list.map(word => `    ${quote(word)},`),
    "];",
  ])
}

function writeReadability() {
  const words = (name: string, values: string[]) => {
    const sorted = [...new Set(values)].sort(compare)
    return [
      `pub(crate) static ${name}: [&str; ${sorted.length}] = [`,
      ...sorted.map(word => `    ${quote(word)},`),
      "];",
    ]
  }
  const problems = Object.entries(problematic).sort(([a], [b]) => compare(a, b))
  const characters = Object.entries(charmap)
    .map(([code, value]) => ({ code: Number(code), value }))
    .sort((left, right) => left.code - right.code)
  const rules = (rules: [string, string][]) =>
    rules.map(
      ([pattern, replacement]) => `    (${quote(pattern)}, ${quote(replacement)}),`,
    )
  write("readability_table.rs", [
    "// The word lists retext-readability reads, sorted for a binary search: dale-chall 2.0.1 and",
    "// spache 2.0.1, syllable 5.0.1's `problematic`, normalize-strings 1.1.1's charmap and",
    "// pluralize 8.0.0's singular rules in their order (patterns as JavaScript writes them)",
    ...words("DALE_CHALL", daleChall),
    ...words("SPACHE", spache),
    `pub(crate) static PROBLEMATIC: [(&str, usize); ${problems.length}] = [`,
    ...problems.map(([word, count]) => `    (${quote(word)}, ${count}),`),
    "];",
    `pub(crate) static CHARMAP: [(u16, &str); ${characters.length}] = [`,
    ...characters.map(({ code, value }) => `    (${code}, ${quote(value)}),`),
    "];",
    `pub(crate) static IRREGULAR: [(&str, &str); ${pluralize.irregular.length}] = [`,
    ...pluralize.irregular.map(
      ([single, plural]) => `    (${quote(single)}, ${quote(plural)}),`,
    ),
    "];",
    `pub(crate) static UNCOUNTABLE: [&str; ${pluralize.uncountable.length}] = [`,
    ...pluralize.uncountable.sort(compare).map(word => `    ${quote(word)},`),
    "];",
    `pub(crate) static SINGULAR_RULES: [(&str, &str); ${pluralize.singular.length}] = [`,
    ...rules(pluralize.singular),
    "];",
  ])
}

// pluralize keeps its rules private: read the literals its factory registers, then append
// the uncountable expressions as singular rules keeping their match, as it does
function readPluralize(path: string) {
  const program = parse(readFileSync(path, "utf8"), { ecmaVersion: "latest" })
  const lists = new Map<string, (Expression | SpreadElement | null)[]>()
  simple(program, {
    CallExpression(node) {
      if (
        node.callee.type !== "MemberExpression" ||
        node.callee.object.type !== "ArrayExpression" ||
        node.callee.property.type !== "Identifier" ||
        node.callee.property.name !== "forEach"
      )
        return
      const source = readFileSync(path, "utf8").slice(
        node.arguments[0]?.start,
        node.arguments[0]?.end,
      )
      const name = /add(\w+)Rule/.exec(source)?.[1]
      if (name) lists.set(name, node.callee.object.elements)
    },
  })
  const literal = (node: Expression | SpreadElement | null) => {
    if (node?.type !== "Literal") throw new Error("pluralize: expected a literal")
    const regex = "regex" in node ? node.regex?.pattern : undefined
    return { regex, text: String(node.value) }
  }
  const pairs = (name: string) =>
    (lists.get(name) ?? []).map(element => {
      if (element?.type !== "ArrayExpression")
        throw new Error("pluralize: expected a pair")
      const [rule, replacement] = element.elements.map(literal)
      const pair: [string, string] = [
        rule?.regex ?? `^${rule?.text}$`,
        replacement?.text ?? "",
      ]
      return pair
    })
  const irregular = (lists.get("Irregular") ?? []).map(element => {
    if (element?.type !== "ArrayExpression") throw new Error("pluralize: expected a pair")
    const [single, plural] = element.elements.map(item => literal(item).text)
    const pair: [string, string] = [
      single?.toLowerCase() ?? "",
      plural?.toLowerCase() ?? "",
    ]
    return pair
  })
  const uncountable: string[] = []
  const singular = pairs("Singular")
  for (const element of lists.get("Uncountable") ?? []) {
    const item = literal(element)
    if (item.regex) singular.push([item.regex, "$0"])
    else uncountable.push(item.text.toLowerCase())
  }
  if (irregular.length === 0 || singular.length === 0 || uncountable.length === 0)
    throw new Error("pluralize: rules not found")
  return { irregular, singular, uncountable }
}

function locate(name: string) {
  return realpathSync(join(MODULES, name))
}

// pnpm puts a package's dependencies beside it
function sibling(path: string, name: string) {
  return realpathSync(join(dirname(path), name))
}

async function load(path: string) {
  return import(pathToFileURL(path).href)
}

// Rust's `str` order: UTF-8 bytes
function compare(left: string, right: string) {
  return Buffer.compare(Buffer.from(left), Buffer.from(right))
}

// A Rust string literal: JSON's escapes, with `\u{…}` for the code points JSON writes `\u…`
function quote(value: string) {
  return JSON.stringify(value).replace(/\\u([0-9a-f]{4})/g, "\\u{$1}")
}

function write(name: string, lines: string[]) {
  writeFileSync(join(HELPERS, name), `${lines.join("\n")}\n`)
}

type Pattern = {
  id: string
  type: "basic" | "or"
  apostrophe?: boolean
  condition?: string
  considerate?: Record<string, string>
  inconsiderate: Record<string, string>
}
