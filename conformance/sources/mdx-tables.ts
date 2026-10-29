import { realpathSync, writeFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { pathToFileURL } from "node:url"

// The data tables `textlane-mdx` reads, generated from the packages the pinned @mdx-js/mdx
// depends on, so the Rust tables never drift from what upstream ships
const MODULES = resolve(import.meta.dirname, "..", "node_modules")
const HELPERS = resolve(import.meta.dirname, "..", "..", "rust", "mdx", "src", "helpers")

const mdx = realpathSync(join(MODULES, "@mdx-js", "mdx"))
const rehypeRecma = dependency(mdx, "@mdx-js/mdx", "rehype-recma")
const toEstree = dependency(rehypeRecma, "rehype-recma", "hast-util-to-estree")
const propertyInformation = dependency(
  toEstree,
  "hast-util-to-estree",
  "property-information",
)
const recmaJsx = dependency(mdx, "@mdx-js/mdx", "recma-jsx")
const acornJsx = dependency(recmaJsx, "recma-jsx", "acorn-jsx")

const information: PropertyInformation = await load(join(propertyInformation, "index.js"))
const entities: Record<string, string> = (await load(join(acornJsx, "xhtml.js"))).default

writeEntities()
writeProperties()

function writeEntities() {
  const rows = Object.entries(entities).sort(([a], [b]) => compare(a, b))
  write("entity_table.rs", [
    "// acorn-jsx 5.3.2's XHTML entities, which JSX text and attribute strings decode, sorted",
    `pub(crate) static ENTITIES: [(&str, &str); ${rows.length}] = [`,
    ...rows.map(([name, value]) => `    (${quote(name)}, ${quote(value)}),`),
    "];",
  ])
}

function writeProperties() {
  const schema = (name: string, value: Schema) => {
    const rows = Object.entries(value.normal)
      .flatMap(([normal, property]) => {
        const info = value.property[property]
        return info ? [{ normal, info }] : []
      })
      .sort((left, right) => compare(left.normal, right.normal))
    return [
      `pub(crate) static ${name}: [Info; ${rows.length}] = [`,
      ...rows.map(({ normal, info }) => {
        const kinds = [
          ...(info.boolean ? ["BOOLEAN"] : []),
          ...(info.commaSeparated ? ["COMMA_SEPARATED"] : []),
        ]
        const space = info.space === undefined ? "None" : `Some(${quote(info.space)})`
        return `    info(${quote(normal)}, ${quote(info.property)}, ${quote(info.attribute)}, ${space}, ${kinds.join(" | ") || "0"}),`
      }),
      "];",
    ]
  }
  const react = Object.entries(information.hastToReact).sort(([a], [b]) => compare(a, b))
  write("property_table.rs", [
    "// property-information 7.2.0's `html` and `svg` schemas, keyed by normalized name (an",
    "// attribute or a property, lowercased) and sorted for a binary search, and `hastToReact`",
    "",
    "pub(crate) const BOOLEAN: u8 = 1;",
    "pub(crate) const COMMA_SEPARATED: u8 = 1 << 1;",
    "",
    "/// What property-information knows about one property.",
    "pub(crate) struct Info {",
    "    pub(crate) normal: &'static str,",
    "    pub(crate) property: &'static str,",
    "    pub(crate) attribute: &'static str,",
    "    pub(crate) space: Option<&'static str>,",
    "    pub(crate) kinds: u8,",
    "}",
    "",
    "const fn info(",
    "    normal: &'static str,",
    "    property: &'static str,",
    "    attribute: &'static str,",
    "    space: Option<&'static str>,",
    "    kinds: u8,",
    ") -> Info {",
    "    Info { normal, property, attribute, space, kinds }",
    "}",
    "",
    ...schema("HTML", information.html),
    "",
    ...schema("SVG", information.svg),
    "",
    `pub(crate) static HAST_TO_REACT: [(&str, &str); ${react.length}] = [`,
    ...react.map(([from, to]) => `    (${quote(from)}, ${quote(to)}),`),
    "];",
  ])
}

// pnpm puts a package's dependencies beside it, under its own scope's depth
function dependency(path: string, name: string, dependency: string) {
  return realpathSync(join(path, ...name.split("/").map(() => ".."), dependency))
}

async function load(path: string) {
  return import(pathToFileURL(path).href)
}

// Rust's `str` order: UTF-8 bytes
function compare(left: string, right: string) {
  return Buffer.compare(Buffer.from(left), Buffer.from(right))
}

// A Rust string literal: JSON's escapes, with `\u{…}` for every code point past ASCII, as
// some entities are invisible
function quote(value: string) {
  return JSON.stringify(value)
    .replace(/\\u([0-9a-f]{4})/g, "\\u{$1}")
    .replace(
      /[\u{7f}-\u{10ffff}]/gu,
      character => `\\u{${character.codePointAt(0)?.toString(16)}}`,
    )
}

function write(name: string, lines: string[]) {
  writeFileSync(join(HELPERS, name), `${lines.join("\n")}\n`)
}

type Info = {
  property: string
  attribute: string
  space?: string
  boolean: boolean
  commaSeparated: boolean
}
type Schema = { normal: Record<string, string>; property: Record<string, Info> }
type PropertyInformation = {
  html: Schema
  svg: Schema
  hastToReact: Record<string, string>
}
