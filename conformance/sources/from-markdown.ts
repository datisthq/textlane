import { isDeepStrictEqual } from "node:util"
import { readdirSync, readFileSync } from "node:fs"
import { basename, join, resolve } from "node:path"
import { parse } from "acorn"
import { ancestor } from "acorn-walk"
import { commonmark } from "commonmark.json"
import { writeGroups } from "../actions/group/write.ts"
import { isStressInput } from "../actions/input/check.ts"
import { parseMarkdown } from "../actions/markdown/render.ts"
import { evaluateNode } from "../actions/node/evaluate.ts"
import { resolveConstants } from "../actions/node/scope.ts"
import { fetchSource } from "../actions/source/fetch.ts"
import { findTestTitle } from "../actions/test/find.ts"

const REPOSITORY = "https://github.com/syntax-tree/mdast-util-from-markdown"
const COMMIT = "afe9516d97683e51974694f944a799a915a242c0"
const OUTPUT = resolve(import.meta.dirname, "..", "tests", "md")

// Fixture names onto construct files where they differ
const FIXTURE_CONSTRUCTS: Record<string, string> = {
  attention: "emphasis",
  blockquote: "block-quote",
  "character-references-everywhere": "character-reference",
  empty: "document",
  "hard-break-escape": "hard-break",
  "hard-break-prefix": "hard-break",
  "image-reference": "image",
  "image-resource": "image",
  "image-resource-eol": "image",
  "link-reference": "link",
  "link-reference-with-phrasing": "link",
  "link-resource": "link",
  "link-resource-eol": "link",
}
// Inline test titles onto construct files, first match wins
const TITLE_CONSTRUCTS: [RegExp, string][] = [
  [/autolink/, "autolink"],
  [/block quote/, "block-quote"],
  [/character escape/, "character-escape"],
  [/character reference/, "character-reference"],
  [/code \(fenced\)/, "code-fenced"],
  [/code \(indented\)/, "code-indented"],
  [/code \(text\)/, "code-text"],
  [/definition/, "definition"],
  [/emphasis|strong/, "emphasis"],
  [/hard break/, "hard-break"],
  [/heading \(atx\)/, "heading-atx"],
  [/heading \(setext\)/, "heading-setext"],
  [/html \(flow\)/, "html-flow"],
  [/html \(text\)/, "html-text"],
  [/image/, "image"],
  [/link/, "link"],
  [/thematic break/, "thematic-break"],
  [/list/, "list"],
  [/paragraph/, "paragraph"],
]

type Test = { description: string; input: string; expected: { tree: object } }

const skipped = new Map<string, number>()
importFromMarkdown()

function importFromMarkdown() {
  const root = fetchSource({ repository: REPOSITORY, commit: COMMIT })
  const seen = new Set(commonmark.map(example => example.markdown))
  const byConstruct = new Map<string, Test[]>()
  const add = (construct: string, test: Test) => {
    if (seen.has(test.input)) return countSkip("duplicate of an earlier case")
    seen.add(test.input)
    byConstruct.set(construct, [...(byConstruct.get(construct) ?? []), test])
  }
  for (const fixture of readFixtures(join(root, "test", "fixtures"))) {
    add(FIXTURE_CONSTRUCTS[fixture.name] ?? fixture.name, fixture.test)
  }
  const source = readFileSync(join(root, "test", "index.js"), "utf8")
  for (const found of extractInputs(source)) {
    const construct = TITLE_CONSTRUCTS.find(([pattern]) => pattern.test(found.title))?.[1]
    const test = {
      description: found.title,
      input: found.input,
      expected: { tree: parseMarkdown(found.input) },
    }
    add(construct ?? "document", test)
  }
  let written = 0
  for (const [construct, tests] of byConstruct) {
    const group = {
      description: "from-markdown",
      source: { repository: REPOSITORY, commit: COMMIT, path: "test", license: "MIT" },
      pipeline: [["md.parse"]],
      tests,
    }
    writeGroups({
      path: join(OUTPUT, `${construct}.json`),
      repository: REPOSITORY,
      groups: [group],
    })
    written += tests.length
  }
  console.log(`from-markdown: ${written} cases in ${byConstruct.size} files`)
  for (const [reason, count] of skipped) console.log(`  skipped ${count}: ${reason}`)
}

// A fixture is upstream's snapshot of the same parser the reference pins; a drift is reported
function readFixtures(directory: string) {
  return readdirSync(directory)
    .filter(name => name.endsWith(".md"))
    .sort()
    .map(file => {
      const name = basename(file, ".md")
      const input = readFileSync(join(directory, file), "utf8")
      const tree = parseMarkdown(input)
      const snapshot = JSON.parse(readFileSync(join(directory, `${name}.json`), "utf8"))
      if (!isDeepStrictEqual(tree, snapshot))
        console.log(`  fixture ${name} drifts from its snapshot`)
      return { name, test: { description: `fixture ${name}`, input, expected: { tree } } }
    })
}

// Only `fromMarkdown(<literal>)`: a second argument carries extensions or an encoding
function extractInputs(source: string) {
  const program = parse(source, { ecmaVersion: "latest", sourceType: "module" })
  const inputs: { title: string; input: string }[] = []
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      const constants = resolveConstants({ program, ancestors })
      if (node.callee.type !== "Identifier" || node.callee.name !== "fromMarkdown") return
      const title = findTestTitle(ancestors, constants)
      if (!title?.startsWith("should parse")) return
      const input = evaluateNode(node.arguments[0], constants)
      if (node.arguments.length > 1) return countSkip("options or an encoding")
      if (typeof input !== "string") return countSkip("not statically evaluable")
      if (isStressInput(input)) return countSkip("a stress test")
      inputs.push({ title, input })
    },
  })
  return inputs
}

function countSkip(reason: string) {
  skipped.set(reason, (skipped.get(reason) ?? 0) + 1)
}
