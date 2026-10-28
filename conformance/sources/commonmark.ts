import { mkdirSync, readdirSync, rmSync, writeFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { commonmark } from "commonmark.json"
import { fromHtml } from "hast-util-from-html"
import { toHtml } from "hast-util-to-html"
import rehypeStringify from "rehype-stringify"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import { unified } from "unified"

const SPEC_VERSION = "0.31.2"
const SPEC_URL = `https://spec.commonmark.org/${SPEC_VERSION}/`
const OUTPUT = resolve(import.meta.dirname, "..", "tests", "md")
const PREFIX = "commonmark-"

// The same options mdast-util-from-markdown's own CommonMark test renders with
const PIPELINE = [
  ["md.parse"],
  ["md.toHtml", { allowDangerousHtml: true }],
  ["html.stringify", { allowDangerousHtml: true }],
]
const reference = unified()
  .use(remarkParse)
  .use(remarkRehype, { allowDangerousHtml: true })
  .use(rehypeStringify, { allowDangerousHtml: true })

writeSections()

function writeSections() {
  mkdirSync(OUTPUT, { recursive: true })
  for (const name of readdirSync(OUTPUT)) {
    if (name.startsWith(PREFIX)) rmSync(join(OUTPUT, name))
  }
  for (const [section, tests] of groupBySection()) {
    const slug = slugifySection(section)
    const group = {
      description: section,
      specification: [`${SPEC_URL}#${slug}`],
      source: {
        repository: "https://github.com/commonmark/commonmark-spec",
        version: SPEC_VERSION,
        path: "spec.txt",
        license: "CC-BY-SA-4.0",
      },
      pipeline: PIPELINE,
      tests,
    }
    writeFileSync(
      join(OUTPUT, `${PREFIX}${slug}.json`),
      `${JSON.stringify([group], null, 2)}\n`,
    )
  }
}

function groupBySection() {
  const sections = new Map<string, object[]>()
  commonmark.forEach((example, index) => {
    const tests = sections.get(example.section) ?? []
    tests.push(
      createTest({ number: index + 1, markdown: example.markdown, html: example.html }),
    )
    sections.set(example.section, tests)
  })
  return sections
}

// Unified's output is the expectation; the spec's HTML is kept where it means something else
function createTest(props: { number: number; markdown: string; html: string }) {
  const actual = String(reference.processSync(props.markdown))
  const differs = normalizeHtml(actual) !== normalizeHtml(props.html)
  return {
    description: `example ${props.number}`,
    ...(differs ? { comment: `CommonMark ${SPEC_VERSION} expects: ${props.html}` } : {}),
    input: props.markdown,
    expected: { html: actual },
  }
}

// As upstream compares: both sides reparsed, the spec's closing newline dropped
function normalizeHtml(html: string) {
  return toHtml(fromHtml(html.replace(/\n$/, ""), { fragment: true }))
}

function slugifySection(section: string) {
  return section
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "")
}
