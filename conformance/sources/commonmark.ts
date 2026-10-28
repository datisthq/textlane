import { join, resolve } from "node:path"
import { commonmark } from "commonmark.json"
import { writeGroups } from "../actions/group/write.ts"
import { normalizeHtml } from "../actions/html/normalize.ts"
import {
  createPipeline,
  parseMarkdown,
  renderMarkdown,
  stringifyTree,
} from "../actions/markdown/render.ts"

const SPEC_VERSION = "0.31.2"
const SPEC_URL = `https://spec.commonmark.org/${SPEC_VERSION}/`
const REPOSITORY = "https://github.com/commonmark/commonmark-spec"
const OUTPUT = resolve(import.meta.dirname, "..", "tests", "md")

// Spec sections onto construct files, named as micromark names its test files
const CONSTRUCTS: Record<string, string> = {
  Tabs: "tab",
  "Backslash escapes": "character-escape",
  "Entity and numeric character references": "character-reference",
  Precedence: "precedence",
  "Thematic breaks": "thematic-break",
  "ATX headings": "heading-atx",
  "Setext headings": "heading-setext",
  "Indented code blocks": "code-indented",
  "Fenced code blocks": "code-fenced",
  "HTML blocks": "html-flow",
  "Link reference definitions": "definition",
  Paragraphs: "paragraph",
  "Blank lines": "blank-line",
  "Block quotes": "block-quote",
  "List items": "list-item",
  Lists: "list",
  Inlines: "inline",
  "Code spans": "code-text",
  "Emphasis and strong emphasis": "emphasis",
  Links: "link",
  Images: "image",
  Autolinks: "autolink",
  "Raw HTML": "html-text",
  "Hard line breaks": "hard-break",
  "Soft line breaks": "soft-break",
  "Textual content": "text",
}

writeSections()

// Each section yields an HTML group and a tree group over the same examples
function writeSections() {
  for (const [section, examples] of groupBySection()) {
    const construct = CONSTRUCTS[section]
    if (!construct) throw new Error(`no construct file for the spec section ${section}`)
    const slug = section.toLowerCase().replace(/[^a-z0-9]+/g, "-")
    const shared = {
      specification: [`${SPEC_URL}#${slug}`],
      source: {
        repository: REPOSITORY,
        version: SPEC_VERSION,
        path: "spec.txt",
        license: "CC-BY-SA-4.0",
      },
    }
    const html = {
      description: section,
      ...shared,
      pipeline: createPipeline({ allowDangerousHtml: true }),
      tests: examples.map(example => createHtmlTest(example)),
    }
    const tree = {
      description: `${section} (tree)`,
      ...shared,
      pipeline: [["md.parse"]],
      tests: examples.map(example => createTreeTest(example)),
    }
    const roundTrip = {
      description: `${section} (round trip)`,
      ...shared,
      pipeline: [["md.parse"], ["md.stringify"]],
      tests: examples.map(example => createRoundTripTest(example)),
    }
    const path = join(OUTPUT, `${construct}.json`)
    writeGroups({ path, repository: REPOSITORY, groups: [html, tree, roundTrip] })
  }
}

type Example = { number: number; markdown: string; html: string }

function groupBySection() {
  const sections = new Map<string, Example[]>()
  commonmark.forEach((example, index) => {
    const examples = sections.get(example.section) ?? []
    examples.push({ number: index + 1, markdown: example.markdown, html: example.html })
    sections.set(example.section, examples)
  })
  return sections
}

// The same options mdast-util-from-markdown's own CommonMark test renders with
function createHtmlTest(example: Example) {
  const actual = renderMarkdown({ markdown: example.markdown, allowDangerousHtml: true })
  const differs = normalizeHtml(actual) !== normalizeHtml(example.html)
  return {
    description: `example ${example.number}`,
    ...(differs
      ? { comment: `CommonMark ${SPEC_VERSION} expects: ${example.html}` }
      : {}),
    input: example.markdown,
    expected: { html: actual },
  }
}

function createTreeTest(example: Example) {
  return {
    description: `example ${example.number}`,
    input: example.markdown,
    expected: { tree: parseMarkdown(example.markdown) },
  }
}

// The serializer over every spec construct, as parsed: what the pinned remark-stringify writes
function createRoundTripTest(example: Example) {
  return {
    description: `example ${example.number}`,
    input: example.markdown,
    expected: { md: stringifyTree({ tree: parseMarkdown(example.markdown) }) },
  }
}
