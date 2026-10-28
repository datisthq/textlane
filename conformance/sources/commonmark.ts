import { join, resolve } from "node:path"
import { commonmark } from "commonmark.json"
import { writeGroups } from "../actions/group/write.ts"
import { normalizeHtml } from "../actions/html/normalize.ts"
import { createPipeline, renderMarkdown } from "../actions/markdown/render.ts"

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

function writeSections() {
  for (const [section, tests] of groupBySection()) {
    const construct = CONSTRUCTS[section]
    if (!construct) throw new Error(`no construct file for the spec section ${section}`)
    const slug = section.toLowerCase().replace(/[^a-z0-9]+/g, "-")
    const group = {
      description: section,
      specification: [`${SPEC_URL}#${slug}`],
      source: {
        repository: REPOSITORY,
        version: SPEC_VERSION,
        path: "spec.txt",
        license: "CC-BY-SA-4.0",
      },
      pipeline: createPipeline({ allowDangerousHtml: true }),
      tests,
    }
    writeGroups({
      path: join(OUTPUT, `${construct}.json`),
      repository: REPOSITORY,
      groups: [group],
    })
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

// The same options mdast-util-from-markdown's own CommonMark test renders with
function createTest(props: { number: number; markdown: string; html: string }) {
  const actual = renderMarkdown({ markdown: props.markdown, allowDangerousHtml: true })
  const differs = normalizeHtml(actual) !== normalizeHtml(props.html)
  return {
    description: `example ${props.number}`,
    ...(differs ? { comment: `CommonMark ${SPEC_VERSION} expects: ${props.html}` } : {}),
    input: props.markdown,
    expected: { html: actual },
  }
}
